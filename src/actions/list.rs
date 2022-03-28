use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use walkdir::WalkDir;

use crate::{
    config::get_or_create_data_dir,
    errors::{KVSError, KVSResult},
    kv_session::KVSSession,
    spec::{KVPayloadResult, KVSAction},
    utils::{sha256, to_u8str},
};

use super::{Actions, KVSToken, KeyMeta};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListAction {
    pub token: KVSToken,
}

impl KVSAction<Vec<KeyMeta>> for ListAction {
    fn serve(
        &mut self,
        _: &mut impl crate::spec::Session,
    ) -> crate::errors::KVSResult<Vec<KeyMeta>> {
        let ListAction { token } = self;
        let addr = token.get_addr();
        let addr_path = get_or_create_data_dir()?.join(addr);
        if !addr_path.exists() {
            return Ok(vec![]);
        }
        let key_files = std::fs::read_dir(addr_path)?
            .filter_map(|p| p.ok())
            .collect::<Vec<_>>();

        Ok(key_files
            .par_iter()
            .map(|file_path| KeyMeta::from_file(file_path.path().join("meta")).unwrap())
            .collect::<Vec<_>>())
    }

    fn request(
        &mut self,
        session: &mut impl crate::spec::Session,
    ) -> crate::errors::KVSResult<Vec<KeyMeta>> {
        session.write(&Actions::ListAction(self.clone()))?;
        let bytes = session.read_vec()?;

        let reply = KVSSession::to::<KVPayloadResult<Vec<KeyMeta>>>(&bytes)?;
        match reply {
            KVPayloadResult::Err(error) => Err(KVSError::LogicError(error.to_string())),
            KVPayloadResult::Ok(reply) => Ok(reply),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LocalFileMeta {
    pub name: String,
    pub path: String,
    pub original_hash: Vec<u8>,
    pub size: u64,
}

impl Debug for LocalFileMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalFileMeta")
            .field("path", &self.path)
            .field("original_hash", &to_u8str(&self.original_hash))
            .field("size", &self.size)
            .finish()
    }
}

impl LocalFileMeta {
    pub fn get_all_files_meta(target_path: &str) -> KVSResult<Vec<LocalFileMeta>> {
        let cwd = std::env::current_dir()?;
        let target_path = relative_path::RelativePath::new(target_path);
        let target_path = target_path.to_logical_path(cwd);
        let files_path = WalkDir::new(&target_path)
            .into_iter()
            .filter(|entry| entry.is_ok())
            .map(|entry| entry.unwrap())
            .filter(|entry| !entry.path().is_dir())
            .map(|entry| {
                let entry_path = entry.path().display().to_string();
                (
                    entry_path.clone(),
                    entry_path
                        .clone()
                        .replace(&target_path.as_path().display().to_string(), ""),
                )
            })
            .collect::<Vec<_>>();
        tracing::info!("analysis local files");
        Ok(files_path
            .par_iter()
            .progress_count(files_path.len() as u64)
            .map(|(entry_path, path)| {
                let bytes = std::fs::read(entry_path).unwrap();
                LocalFileMeta {
                    name: path.clone(),
                    original_hash: sha256(&bytes),
                    size: bytes.len() as u64,
                    path: entry_path.to_string(),
                }
            })
            .collect::<Vec<_>>())
    }
}
