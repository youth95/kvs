use serde::{Deserialize, Serialize};

use crate::{
    errors::KVSError,
    kv_session::KVSSession,
    spec::{KVPayloadResult, KVSAction},
};

use super::Actions;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteVersionAction;

impl KVSAction<String> for RemoteVersionAction {
    fn serve(&mut self, _: &mut impl crate::spec::Session) -> crate::errors::KVSResult<String> {
        Ok(version!().to_string())
    }

    fn request(
        &mut self,
        session: &mut impl crate::spec::Session,
    ) -> crate::errors::KVSResult<String> {
        session.write(&Actions::RemoteVersionAction(self.clone()))?;
        let bytes = session.read_vec()?;

        let reply = KVSSession::to::<KVPayloadResult<String>>(&bytes)?;

        match reply {
            KVPayloadResult::Err(error) => Err(KVSError::LogicError(error)),
            KVPayloadResult::Ok(version) => Ok(version),
        }
    }
}
