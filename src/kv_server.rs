use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use crate::actions::{Actions, Cat, CreateKeyValue};
use crate::spec::KVSServe;
use crate::utils::sgin;
use crate::{KVSError, KVSResult, KVSSession, KVSToken};

fn verify_jwt_token(jwt_secret: &[u8], msg: &Actions) -> KVSResult<()> {
    let token = match &msg {
        Actions::FetchToken(_) => None,
        Actions::CreateKeyValue(CreateKeyValue { token, .. }) => Some(token),
        Actions::Cat(Cat { token, .. }) => Some(token),
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

fn handle_client(stream: TcpStream, jwt_secret: &[u8]) -> KVSResult<()> {
    chrono::Local::now().timestamp_millis();
    stream.set_read_timeout(Some(Duration::from_millis(1000)))?;
    let mut session = KVSSession::new(stream)?;
    let msg = KVSSession::to::<Actions>(&session.read_vec()?)?;
    // token verify
    verify_jwt_token(jwt_secret, &msg)?;

    match &msg {
        Actions::FetchToken(fetch_token) => fetch_token.serve(session, Some(jwt_secret.to_vec())),
        Actions::CreateKeyValue(create_key_value) => create_key_value.serve(session, None),
        Actions::Cat(cat) => cat.serve(session, None),
    }
}

pub fn start_server(addr: &String, jwt_secret: &Vec<u8>) {
    let listener = TcpListener::bind(addr).expect("Could not bind");
    tracing::info!("starting with {} successfully!", addr);
    for stream in listener.incoming() {
        match stream {
            Err(e) => {
                tracing::error!("{}", e)
            }
            Ok(stream) => {
                let jwt_secret = jwt_secret.clone();
                thread::spawn(move || {
                    handle_client(stream, &jwt_secret)
                        .unwrap_or_else(|error| eprintln!("{:?}", error));
                });
            }
        }
    }
}
