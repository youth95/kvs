use serde::{Deserialize, Serialize};

use crate::{
    spec::{KVPayloadResult, KVSRequest, KVSServe},
    utils::{sgin, to_u8str},
    KVSError, KVSResult, KVSSession, Secret,
};

use super::Actions;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchToken {
    pub pub_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KVSToken {
    pub id: Vec<u8>, // len 20
    pub time_stamp: i64,
    pub sign: Vec<u8>, // len 32
}

impl KVSServe<Vec<u8>> for FetchToken {
    fn serve(&mut self, session: &mut KVSSession, jwt_secret: Option<Vec<u8>>) -> KVSResult<()> {
        match jwt_secret {
            None => Err(KVSError::LogicError("jwt_secret is required".to_string())),
            Some(jwt_secret) => {
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
                session.write(&KVPayloadResult::Ok(token))?;

                Ok(())
            }
        }
    }
}

impl KVSRequest<&Secret, KVSToken> for FetchToken {
    fn request(&mut self, session: &mut KVSSession, secret: Option<&Secret>) -> KVSResult<KVSToken> {
        match secret {
            None => Err(KVSError::LogicError("secret is requred".to_string())),
            Some(secret) => {
                // 1. c -> s [fetch_token,public_key]
                let fetch_token_payload = Actions::FetchToken(self.clone());

                session.write(&fetch_token_payload)?;
                // 2. s -> c [random_nonce]
                let random_nonce = session.read_vec()?;
                tracing::debug!("random_nonce {:x?}", random_nonce);
                let c_nonce =
                    Secret::decrypt_width_priv_key_bits(&secret.priv_key_bits, &random_nonce)?;
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
    }
}
