use std::{net::TcpStream, time::Duration, io::Write};

use crate::{KVSResult, KVSToken, KVSSession, Secret, fetch_token};

pub fn check_and_auto_fetch_token_to_disk(repository: &String) -> KVSResult<KVSToken> {
    // paths
    let user_kvs_config_dir_path = std::path::Path::new(".kvs");
    let user_token_file_path = user_kvs_config_dir_path.join("token");

    if user_token_file_path.exists() {
        Ok(bincode::deserialize(&std::fs::read(user_token_file_path).unwrap()).unwrap())
    } else {
        let secret = get_or_create_secret();
        let mut session = {
            let stream = TcpStream::connect(repository)?;
            stream.set_read_timeout(Some(Duration::from_millis(1000)))?;
            KVSSession::new(stream)
        }?;
        fetch_token(&mut session, &secret)
    }
}

pub fn get_or_create_secret() -> Secret {
    // paths
    let user_kvs_config_dir_path = std::path::Path::new(".kvs");
    let user_secret_file_path = user_kvs_config_dir_path.join("secret");
    if user_secret_file_path.exists() {
        Secret::from(std::fs::read_to_string(user_secret_file_path).unwrap())
    } else {
        let secret = Secret::default();
        std::fs::create_dir_all(user_kvs_config_dir_path).unwrap();
        let mut file = std::fs::File::create(user_secret_file_path).unwrap();
        file.write_all(&secret.to_string().as_bytes()).unwrap();
        secret
    }
}
