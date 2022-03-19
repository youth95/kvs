use serde::{Deserialize, Serialize};

use crate::{
    kv_stream::NONCE,
    spec::{KVPayloadResult, KVSRequest, KVSServe, ReplyCode},
    utils::{sha256, to_u8str},
    KVSError, KVSResult, KVSSession, Secret,
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
}
// impl KeyMeta {
//     pub fn mime(&self) -> KVSResult<mime::Mime> {
//         let result = self.mime.parse()?;
//         Ok(result)
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateKeyValue {
    pub token: KVSToken,
    pub key: String,
    pub meta: KeyMeta,
    pub value: Vec<u8>,
}


impl KVSServe<()> for CreateKeyValue {
    fn serve(&mut self, session: &mut KVSSession, _: Option<()>) -> KVSResult<()> {
        let CreateKeyValue {
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
        let data_dir_path = std::path::Path::new("data");
        let data_user_dir_path = data_dir_path.clone().join(&id_str);
        let kv_path = data_user_dir_path.clone().join(&key);
        if kv_path.exists() {
            return Err(KVSError::LogicError(format!(
                "The key: {} arealy exists.",
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
}

impl KVSRequest<Secret, ()> for CreateKeyValue {
    fn request(&mut self, session: &mut KVSSession, secret: Option<Secret>) -> KVSResult<()> {
        if let Some(rand) = &self.meta.rand {
            let key = Key::from_slice(rand.as_slice());
            let cipher = Aes256Gcm::new(key);
            self.value = cipher.encrypt(Nonce::from_slice(NONCE), &*self.value)?;

            if let Some(secret) = secret {
                let key = Key::from_slice(&secret.priv_key_bits[..32]);
                let cipher = Aes256Gcm::new(key);
                self.meta.rand = Some(cipher.encrypt(Nonce::from_slice(NONCE), rand.as_slice())?);
            }
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
