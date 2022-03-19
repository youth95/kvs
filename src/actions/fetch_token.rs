use serde::{Deserialize, Serialize};

use crate::{
    config::{get_or_create_jwt_secret, get_or_create_secret},
    errors::{KVSError, KVSResult},
    kv_session::KVSSession,
    secret::Secret,
    spec::{KVPayloadResult, KVSAction, Session},
    utils::{sgin, to_u8str},
};

use super::Actions;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchTokenAction {
    pub pub_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KVSToken {
    pub id: Vec<u8>, // len 20
    pub time_stamp: i64,
    pub sign: Vec<u8>, // len 32
}

impl KVSAction<KVSToken> for FetchTokenAction {
    fn serve(&mut self, session: &mut impl Session) -> KVSResult<KVSToken> {
        let jwt_secret = get_or_create_jwt_secret(false)?;
        if self.pub_key.len() != 162 {
            return Err(KVSError::LogicError("Illegal public key".to_string()));
        }
        let addr = sgin(&self.pub_key);
        let addr_str = to_u8str(&addr);
        tracing::info!("[0x{}] fetch_token", addr_str);
        let nonce = (0..32).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
        let nonce_encrypt = Secret::encrypt_with_pub_key_bits(&self.pub_key, &nonce);
        session.write_vec(&nonce_encrypt)?;
        let c_nonce = session.read_vec()?;
        if c_nonce != nonce {
            return Err(KVSError::LogicError(format!(
                "Illegal NONCE reply with: {:x?}",
                c_nonce
            )));
        }

        let time_stamp = chrono::Local::now().timestamp_millis();

        let token = KVSToken {
            id: addr.clone(),
            time_stamp,
            sign: sgin(
                &[
                    addr.clone(),
                    time_stamp.to_be_bytes().to_vec(),
                    jwt_secret.to_vec(),
                ]
                .concat(),
            ),
        };
        Ok(token)
    }

    fn request(&mut self, session: &mut impl Session) -> KVSResult<KVSToken> {
        let secret = get_or_create_secret()?;
        // 1. c -> s [fetch_token,public_key]
        let fetch_token_payload = Actions::FetchToken(self.clone());

        session.write(&fetch_token_payload)?;
        // 2. s -> c [random_nonce]
        let random_nonce = session.read_vec()?;
        tracing::debug!("random_nonce {:x?}", random_nonce);
        let c_nonce = Secret::decrypt_width_priv_key_bits(&secret.priv_key_bits, &random_nonce)?;
        // 3. c -> s [random_sign]
        session.write_vec(&c_nonce)?;
        // 4. s -> c [jwt_token, addr,time_stamp,sign]
        let token_bytes = session.read_vec()?;
        match KVSSession::to::<KVPayloadResult<KVSToken>>(&token_bytes)? {
            KVPayloadResult::Err(error) => Err(KVSError::LogicError(error)),
            KVPayloadResult::Ok(token) => Ok(token),
        }
    }
}
