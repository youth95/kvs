use crate::{KVSResult, KVSSession};

pub trait KVSServe<C> {
    fn serve(&self, session: KVSSession, context: Option<C>) -> KVSResult<()>;
}

pub trait KVSRequest<C, R> {
    fn request(&self, session: KVSSession, context: Option<C>) -> KVSResult<R>;
}
