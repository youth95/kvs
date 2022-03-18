use serde::{Deserialize, Serialize};

use crate::{
    spec::{KVSRequest, KVSServe},
    utils::{sha256, to_u8str},
    KVSResult, KVSSession, KVSToken,
};

use super::Actions;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cat {
    pub token: KVSToken,
    pub key: String,
}

impl KVSRequest<(), Vec<u8>> for Cat {
    fn request(&self, mut session: KVSSession, _: Option<()>) -> KVSResult<Vec<u8>> {
        session.write(&Actions::Cat(self.clone()))?;
        // todo status check
        session.read_vec()
    }
}

impl KVSServe<()> for Cat {
    fn serve(&self, mut session: KVSSession, _: Option<()>) -> KVSResult<()> {
        let Cat { key, token } = self;
        let KVSToken { id, .. } = token;
        let id_str = ["0x", &to_u8str(&id)].concat();
        let o_key = key.clone();
        let key = to_u8str(&sha256(key.as_bytes()));

        let data_dir_path = std::path::Path::new("data");
        let data_user_dir_path = data_dir_path.clone().join(&id_str);
        let kv_path = data_user_dir_path.clone().join(&key);
        let content_file_path = kv_path.clone().join("value");
        tracing::debug!("want cat value in: {}", content_file_path.display());
        if !content_file_path.exists() {
            todo!("need check ownner")
        } else {
            tracing::info!("[{}] Cat File Value: {} ({})", id_str, key, o_key);
            session.write_vec(&std::fs::read(content_file_path)?)?;
        }
        Ok(())
    }
}
