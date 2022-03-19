use aes_gcm::{aead::Aead, Aes256Gcm, Key, NewAead, Nonce};
use serde::{Deserialize, Serialize};

use crate::{
    actions::KeyMeta,
    kv_stream::NONCE,
    spec::{KVPayloadResult, KVSRequest, KVSServe},
    utils::{sha256, to_u8str},
    KVSError, KVSResult, KVSSession, KVSToken, Secret,
};

use super::Actions;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cat {
    pub token: KVSToken,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CatReply {
    meta: KeyMeta,
    content: Vec<u8>,
}

impl KVSRequest<Secret, Vec<u8>> for Cat {
    fn request(&mut self, session: &mut KVSSession, secret: Option<Secret>) -> KVSResult<Vec<u8>> {
        session.write(&Actions::Cat(self.clone()))?;
        // todo status check
        match KVSSession::to::<KVPayloadResult<CatReply>>(&session.read_vec()?)? {
            KVPayloadResult::Err(error) => Err(KVSError::LogicError(error)),
            KVPayloadResult::Ok(content) => {
                let CatReply { meta, mut content } = content;
                if let Some(rand) = meta.rand {
                    if let Some(secret) = secret {
                        let key = Key::from_slice(&secret.priv_key_bits[..32]);
                        let cipher = Aes256Gcm::new(key);
                        let rand = cipher.decrypt(Nonce::from_slice(NONCE), rand.as_slice())?;
                        let key = Key::from_slice(rand.as_slice());
                        let cipher = Aes256Gcm::new(key);
                        content = cipher.decrypt(Nonce::from_slice(NONCE), &*content)?;
                    } else {
                        return Err(KVSError::LogicError("The secret is required".to_string()));
                    }
                }
                Ok(content)
            }
        }
    }
}

impl KVSServe<()> for Cat {
    fn serve(&mut self, session: &mut KVSSession, _: Option<()>) -> KVSResult<()> {
        let Cat { key, token } = self;
        let KVSToken { id, .. } = token;
        let id_str = ["0x", &to_u8str(&id)].concat();
        let o_key = key.clone();
        let key = to_u8str(&sha256(key.as_bytes()));

        let data_dir_path = std::path::Path::new("data");
        let data_user_dir_path = data_dir_path.clone().join(&id_str);
        let kv_path = data_user_dir_path.clone().join(&key);

        if !kv_path.exists() {
            return Err(KVSError::LogicError(format!(
                "The key: `{}` is not exists.",
                o_key
            )));
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
            session.write(&KVPayloadResult::Ok(send_content))?;
        }
        Ok(())
    }
}
