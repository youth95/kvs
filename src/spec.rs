use serde::{Deserialize, Serialize};

use crate::errors::KVSResult;

pub trait Session {
    fn read_vec(&mut self) -> KVSResult<Vec<u8>>;

    fn write_vec(&mut self, payload: &[u8]) -> KVSResult<()>;

    fn write<T: ?Sized>(&mut self, payload: &T) -> KVSResult<()>
    where
        T: serde::Serialize;
}
pub trait KVSAction<R: serde::Serialize> {
    fn serve(&mut self, session: &mut impl Session) -> KVSResult<R>;
    fn request(&mut self, session: &mut impl Session) -> KVSResult<R>;

    fn serve_serialize(&mut self, session: &mut impl Session) -> KVSResult<Vec<u8>> {
        let data = bincode::serialize(&KVPayloadResult::Ok(self.serve(session)?))?;
        Ok(data)
    }

    fn request_serialize(&mut self, session: &mut impl Session) -> KVSResult<Vec<u8>> {
        let data = bincode::serialize(&KVPayloadResult::Ok(self.request(session)?))?;
        Ok(data)
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub enum KVPayloadResult<T> {
//     Err(String),
//     Ok(T),
// }

pub type KVPayloadResult<T> = Result<T, String>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[repr(u8)]
pub enum ReplyCode {
    Ok = 0,
}
