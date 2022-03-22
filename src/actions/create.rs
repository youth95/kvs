use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    config::{get_or_create_data_dir, get_or_create_secret},
    errors::{KVSError, KVSResult},
    kv_session::{KVSSession, NONCE},
    spec::{KVPayloadResult, KVSAction, ReplyCode, Session},
    utils::{sha256, to_u8str},
};

use aes_gcm::{aead::Aead, Aes256Gcm, Key, NewAead, Nonce};

use super::{Actions, KVSToken};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyMeta {
    pub mime: String,
    pub size: u64,
    pub owner: Vec<u8>,
    pub name: String,
    pub rand: Option<Vec<u8>>,
    pub original_hash: Vec<u8>,
}

impl KeyMeta {
    pub fn from_file<P: AsRef<Path>>(meta_file_path: P) -> KVSResult<KeyMeta> {
        let meta = std::fs::read(meta_file_path)?;
        KVSSession::to::<KeyMeta>(&meta)
    }
}

// impl KeyMeta {
//     pub fn mime(&self) -> KVSResult<mime::Mime> {
//         let result = self.mime.parse()?;
//         Ok(result)
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateAction {
    pub token: KVSToken,
    pub key: String,
    pub meta: KeyMeta,
    pub value: Vec<u8>,
}

impl KVSAction<()> for CreateAction {
    fn serve(&mut self, session: &mut impl Session) -> KVSResult<()> {
        let CreateAction {
            token,
            key,
            value,
            meta,
        } = self;
        let KVSToken { id, .. } = token;
        meta.owner = id.clone();
        let id_str = token.get_addr();
        let o_key = key.clone();
        let key = to_u8str(&sha256(key.as_bytes()));
        let data_dir_path = get_or_create_data_dir()?;
        let data_user_dir_path = data_dir_path.clone().join(&id_str);
        let kv_path = data_user_dir_path.clone().join(&key);
        if kv_path.exists() {
            return Err(KVSError::LogicError(format!(
                "The key: `{}` arealy exists.",
                o_key
            )));
        } else {
            std::fs::create_dir_all(&kv_path)?;
            std::fs::write(kv_path.join("meta"), bincode::serialize(&meta)?)?;
            std::fs::write(kv_path.join("value"), value)?;
            tracing::info!("[{}] Create Key: {} ({})", id_str, key, o_key);
            session.write(&KVPayloadResult::Ok(ReplyCode::Ok))?
        }
        Ok(())
    }

    fn request(&mut self, session: &mut impl Session) -> KVSResult<()> {
        let secret = get_or_create_secret()?;
        if let Some(rand) = &self.meta.rand {
            let key = Key::from_slice(rand.as_slice());
            let cipher = Aes256Gcm::new(key);
            self.value = cipher.encrypt(Nonce::from_slice(NONCE), &*self.value)?;
            let key = Key::from_slice(&secret.priv_key_bits[..32]);
            let cipher = Aes256Gcm::new(key);
            self.meta.rand = Some(cipher.encrypt(Nonce::from_slice(NONCE), rand.as_slice())?);
        }

        session.write(&Actions::CreateKeyValue(self.clone()))?;
        let reply = session.read_vec()?;
        match KVSSession::to::<KVPayloadResult<ReplyCode>>(&reply)? {
            KVPayloadResult::Err(error) => {
                tracing::error!("{}", error);
                Ok(())
            }
            KVPayloadResult::Ok(_) => Ok(()),
        }
    }
}
