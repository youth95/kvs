use crate::actions::{Actions, CatAction, CreateKeyValueAction, KVSToken};
use crate::errors::{KVSError, KVSResult};
use crate::kv_session::KVSSession;
use crate::spec::{KVPayloadResult, KVSAction, Session};
use crate::utils::sgin;

pub fn verify_jwt_token(jwt_secret: &[u8], msg: &Actions) -> KVSResult<()> {
    let token = match &msg {
        Actions::FetchToken(_) => None,
        Actions::CreateKeyValue(CreateKeyValueAction { token, .. }) => Some(token),
        Actions::CatAction(CatAction { token, .. }) => Some(token),
    };
    if let Some(token) = token {
        let KVSToken {
            id,
            time_stamp,
            sign,
        } = token;
        let s_sign = sgin(
            &[
                id.clone(),
                time_stamp.to_be_bytes().to_vec(),
                jwt_secret.to_vec(),
            ]
            .concat(),
        );

        tracing::debug!("verify_jwt_token");
        tracing::debug!("jwt_secret: {:x?}", jwt_secret);
        tracing::debug!("s_sign: {:x?}", s_sign);
        tracing::debug!("sign: {:x?}", sign);
        if s_sign != *sign {
            return Err(KVSError::LogicError("Illegal Token".to_string()));
        }
    }

    Ok(())
}

pub fn handle_client(session: &mut impl Session, jwt_secret: &[u8]) -> KVSResult<Vec<u8>> {
    let msg = KVSSession::to::<Actions>(&session.read_vec()?)?;
    verify_jwt_token(jwt_secret, &msg)?;
    let reply = match msg {
        Actions::FetchToken(mut fetch_token) => fetch_token.serve_serialize(session),
        Actions::CreateKeyValue(mut create_key_value) => create_key_value.serve_serialize(session),
        Actions::CatAction(mut cat) => cat.serve_serialize(session),
    }?;
    Ok(reply)
}

pub fn service(session: &mut impl Session, jwt_secret: &[u8]) {
    match handle_client(session, jwt_secret) {
        Ok(reply) => session
            .write_vec(&reply)
            .unwrap_or_else(|error| tracing::error!("{}", error)),
        Err(error) => match error {
            KVSError::LogicError(logic_error) => {
                session
                    .write(&KVPayloadResult::<()>::Err(logic_error))
                    .unwrap_or_else(|error| tracing::error!("{}", error));
            }
            _ => tracing::error!("{}", error),
        },
    }
}
