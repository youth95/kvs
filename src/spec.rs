use serde::{Deserialize, Serialize};

use crate::{KVSResult, KVSSession};

pub trait KVSServe<C> {
    fn serve(&mut self, session: &mut KVSSession, context: Option<C>) -> KVSResult<()>;
}

pub trait KVSRequest<C, R> {
    fn request(&mut self, session: &mut KVSSession, context: Option<C>) -> KVSResult<R>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KVPayloadResult<T> {
    Err(String),
    Ok(T),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[repr(u8)]
pub enum ReplyCode {
    Ok = 0,
}
