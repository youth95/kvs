use serde::{Deserialize, Serialize};

use crate::{
    errors::KVSError,
    kv_session::KVSSession,
    spec::{KVPayloadResult, KVSAction, ReplyCode},
    utils::{sha256, to_u8str},
};

use super::{Actions, KVSToken};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteAction {
    pub token: KVSToken,
    pub key: String,
}

impl KVSAction<ReplyCode> for DeleteAction {
    fn serve(&mut self, _: &mut impl crate::spec::Session) -> crate::errors::KVSResult<ReplyCode> {
        let DeleteAction { key, token } = self;
        let KVSToken { id, .. } = token;
        let id_str = ["0x", &to_u8str(&id)].concat();
        let o_key = key.clone();
        let key = to_u8str(&sha256(key.as_bytes()));

        let data_dir_path = std::path::Path::new("data");
        let data_user_dir_path = data_dir_path.clone().join(&id_str);
        let kv_path = data_user_dir_path.clone().join(&key);
        if !kv_path.exists() {
            Err(KVSError::LogicError(format!(
                "The key: `{}` is not exists.",
                o_key
            )))
        } else {
            tracing::info!("[{}] Delete File Value: {} ({})", id_str, key, o_key);
            std::fs::remove_dir_all(kv_path)?;
            Ok(ReplyCode::Ok)
        }
    }

    fn request(
        &mut self,
        session: &mut impl crate::spec::Session,
    ) -> crate::errors::KVSResult<ReplyCode> {
        session.write(&Actions::DeleteAction(self.clone()))?;
        let bytes = session.read_vec()?;
        let reply = KVSSession::to::<KVPayloadResult<ReplyCode>>(&bytes)?;
        match reply {
            KVPayloadResult::Err(error) => Err(KVSError::LogicError(error.to_string())),
            KVPayloadResult::Ok(_) => Ok(ReplyCode::Ok),
        }
    }
}
