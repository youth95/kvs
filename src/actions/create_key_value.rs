use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::{
    spec::{KVSRequest, KVSServe},
    utils::{sha256, to_u8str},
    KVSResult, KVSSession,
};

use super::{Actions, KVSToken, Value};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateKeyValue {
    pub token: KVSToken,
    pub key: String,
    pub value: Value,
}

impl KVSServe<()> for CreateKeyValue {
    fn serve(&self, _: KVSSession, _: Option<()>) -> KVSResult<()> {
        let CreateKeyValue { key, token, value } = self;
        let KVSToken { id, .. } = token;
        let id_str = ["0x", &to_u8str(&token.id)].concat();
        let o_key = key.clone();
        let key = to_u8str(&sha256(key.as_bytes()));

        let data = match &value {
            Value::Text(data) => data,
            Value::Bin(data) => data,
        };

        let data_type = match &value {
            Value::Text(_) => "text",
            Value::Bin(_) => "bin",
        };
        let data_dir_path = std::path::Path::new("data");
        let data_user_dir_path = data_dir_path.clone().join(&id_str);
        let kv_path = data_user_dir_path.clone().join(&key);
        if kv_path.exists() {
            todo!("need check ownner")
        } else {
            std::fs::create_dir_all(&kv_path)?;
            std::fs::File::create(kv_path.join("type"))?.write_all(data_type.as_bytes())?;
            std::fs::File::create(kv_path.join("value"))?.write_all(data)?;
            std::fs::File::create(kv_path.join("owner"))?.write_all(&id)?;
            tracing::info!("[{}] Create Key: {} ({})", id_str, key, o_key);
        }
        Ok(())
    }
}

impl KVSRequest<(), ()> for CreateKeyValue {
    fn request(&self, mut session: KVSSession, _: Option<()>) -> KVSResult<()> {
        session.write(&Actions::CreateKeyValue(self.clone()))
    }
}
