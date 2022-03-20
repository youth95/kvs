use aes_gcm::{aead::Aead, Aes256Gcm, Key, NewAead, Nonce};
use serde::{Deserialize, Serialize};

use crate::{
    actions::KeyMeta,
    config::{get_or_create_data_dir, get_or_create_secret},
    errors::{KVSError, KVSResult},
    kv_session::{KVSSession, NONCE},
    spec::{KVPayloadResult, KVSAction, Session},
    utils::{sha256, to_u8str},
};

use super::{Actions, KVSToken};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReadAction {
    pub token: KVSToken,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CatReply {
    meta: KeyMeta,
    content: Vec<u8>,
}

impl CatReply {
    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }
}

impl KVSAction<CatReply> for ReadAction {
    fn serve(&mut self, _: &mut impl Session) -> KVSResult<CatReply> {
        let ReadAction { key, token } = self;
        let KVSToken { id, .. } = token;
        let id_str = ["0x", &to_u8str(&id)].concat();
        let o_key = key.clone();
        let key = to_u8str(&sha256(key.as_bytes()));

        let data_dir_path = get_or_create_data_dir()?;
        let data_user_dir_path = data_dir_path.clone().join(&id_str);
        let kv_path = data_user_dir_path.clone().join(&key);

        if !kv_path.exists() {
            Err(KVSError::LogicError(format!(
                "The key: `{}` is not exists.",
                o_key
            )))
        } else {
            tracing::info!("[{}] Cat File Value: {} ({})", id_str, key, o_key);
            let content_file_path = kv_path.clone().join("value");
            tracing::debug!("want cat value in: {}", content_file_path.display());
            let meta_file_path = kv_path.clone().join("meta");
            let meta = std::fs::read(meta_file_path)?;
            let meta = KVSSession::to::<KeyMeta>(&meta)?;

            // check owner
            if meta.rand.is_some() {
                if meta.owner != token.id {
                    return Err(KVSError::LogicError(format!(
                        "The key: `{}` is private.",
                        o_key
                    )));
                }
            }

            let content = std::fs::read(content_file_path)?;
            let send_content = CatReply { meta, content };
            Ok(send_content)
        }
    }

    fn request(&mut self, session: &mut impl Session) -> KVSResult<CatReply> {
        let secret = get_or_create_secret()?;
        session.write(&Actions::CatAction(self.clone()))?;
        let bytes = session.read_vec()?;

        let reply = KVSSession::to::<KVPayloadResult<CatReply>>(&bytes)?;
        match reply {
            KVPayloadResult::Err(error) => Err(KVSError::LogicError(error.to_string())),
            KVPayloadResult::Ok(mut reply) => {
                if let Some(rand) = reply.meta.rand.clone() {
                    let key = Key::from_slice(&secret.priv_key_bits[..32]);
                    let cipher = Aes256Gcm::new(key);
                    let rand = cipher.decrypt(Nonce::from_slice(NONCE), rand.as_slice())?;
                    let key = Key::from_slice(rand.as_slice());
                    let cipher = Aes256Gcm::new(key);
                    reply.content = cipher.decrypt(Nonce::from_slice(NONCE), &*reply.content)?;
                    Ok(reply)
                } else {
                    Err(KVSError::LogicError("rand is required".to_string()))
                }
            }
        }
    }
}

#[cfg(test)]
mod fetch_token {

    use crate::{config::get_or_create_token, kv_session::MockSession, spec::KVSAction};

    use super::ReadAction;

    #[test]
    fn serve() {
        let (token, _) = get_or_create_token(&"".to_string(), false).unwrap();
        let key = "pr".to_string();
        let mut session = MockSession::new().unwrap();
        let mut cat_action = ReadAction { token, key };
        cat_action.serve(&mut session).unwrap();
    }
}
