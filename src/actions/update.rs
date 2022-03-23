use serde::{Deserialize, Serialize};

use crate::{
    config::{get_or_create_data_dir, get_or_create_secret},
    errors::{KVSError, KVSResult},
    kv_session::{KVSSession, NONCE},
    spec::{KVPayloadResult, KVSAction, ReplyCode, Session},
    utils::{sha256, to_u8str},
};

use aes_gcm::{aead::Aead, Aes256Gcm, Key, NewAead, Nonce};

use super::{Actions, KVSToken, KeyMeta};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateAction {
    pub token: KVSToken,
    pub key: String,
    pub meta: KeyMeta,
    pub value: Vec<u8>,
}

impl KVSAction<ReplyCode> for UpdateAction {
    fn serve(&mut self, session: &mut impl Session) -> KVSResult<ReplyCode> {
        let UpdateAction {
            token,
            key,
            value,
            meta,
        } = self;
        let KVSToken { id, .. } = token;
        meta.owner = id.clone();

        let id_str = ["0x", &to_u8str(&token.id)].concat();
        let o_key = key.clone();
        let key = to_u8str(&sha256(key.as_bytes()));
        let data_dir_path = get_or_create_data_dir()?;
        let data_user_dir_path = data_dir_path.clone().join(&id_str);
        let kv_path = data_user_dir_path.clone().join(&key);
        if !kv_path.exists() {
            return Err(KVSError::LogicError(format!(
                "The key: `{}` is not exists.",
                o_key
            )));
        } else {
            std::fs::write(kv_path.join("meta"), bincode::serialize(&meta)?)?;
            std::fs::write(kv_path.join("value"), value)?;
            tracing::info!("[{}] Update Key: {} ({})", id_str, key, o_key);
            session.write(&KVPayloadResult::Ok(ReplyCode::Ok))?
        }
        Ok(ReplyCode::Ok)
    }

    fn request(&mut self, session: &mut impl Session) -> KVSResult<ReplyCode> {
        let secret = get_or_create_secret()?;
        if let Some(rand) = &self.meta.rand {
            let key = Key::from_slice(rand.as_slice());
            let cipher = Aes256Gcm::new(key);
            self.value = cipher.encrypt(Nonce::from_slice(NONCE), &*self.value)?;
            let key = Key::from_slice(&secret.priv_key_bits[..32]);
            let cipher = Aes256Gcm::new(key);
            self.meta.rand = Some(cipher.encrypt(Nonce::from_slice(NONCE), rand.as_slice())?);
        }

        session.write(&Actions::UpdateAction(self.clone()))?;
        let reply = session.read_vec()?;
        match KVSSession::to::<KVPayloadResult<ReplyCode>>(&reply)? {
            KVPayloadResult::Err(error) => {
                tracing::error!("{}", error);
                Ok(ReplyCode::Ok)
            }
            KVPayloadResult::Ok(_) => Ok(ReplyCode::Ok),
        }
    }
}
