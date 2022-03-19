use std::{io::Write, net::TcpStream, path::PathBuf, time::Duration};

use crate::{fetch_token, KVSSession, KVSToken, Secret};

pub fn get_or_create_token(repository: &String, froce_create: bool) -> (KVSToken, String) {
    let user_token_file_path = &get_or_create_user_config_dir().join("token");

    if user_token_file_path.exists() && froce_create == false {
        (
            bincode::deserialize(&std::fs::read(user_token_file_path).unwrap()).unwrap(),
            user_token_file_path.display().to_string(),
        )
    } else {
        let secret = get_or_create_secret();
        let mut session = {
            let stream = TcpStream::connect(repository).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_millis(1000)))
                .unwrap();
            KVSSession::new(stream)
        }
        .unwrap();
        let token = fetch_token(&mut session, &secret).unwrap();

        let token_bytes = bincode::serialize(&token).unwrap();
        std::fs::write(&user_token_file_path, &token_bytes).unwrap();
        (token, user_token_file_path.display().to_string())
    }
}

pub fn get_or_create_secret() -> Secret {
    let user_kvs_config_dir_path = get_or_create_user_config_dir();
    let user_secret_file_path = user_kvs_config_dir_path.join("secret");
    if user_secret_file_path.exists() {
        Secret::from(std::fs::read_to_string(user_secret_file_path).unwrap())
    } else {
        let secret = Secret::default();
        std::fs::create_dir_all(*user_kvs_config_dir_path).unwrap();
        let mut file = std::fs::File::create(user_secret_file_path).unwrap();
        file.write_all(&secret.to_string().as_bytes()).unwrap();
        secret
    }
}

pub fn get_or_create_jwt_secret(froce_create: bool) -> Vec<u8> {
    let user_kvs_config_dir_path = get_or_create_user_config_dir();
    let jwt_secret_file_path = user_kvs_config_dir_path.join("jwt_secret");

    if jwt_secret_file_path.exists() && froce_create == false {
        std::fs::read(jwt_secret_file_path).unwrap()
    } else {
        let jwt_secret = (0..256).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
        std::fs::write(jwt_secret_file_path, &jwt_secret).unwrap();
        jwt_secret
    }
}

pub fn get_or_create_user_config_dir() -> Box<PathBuf> {
    let user_kvs_config_dir_path = dirs::home_dir().unwrap().join(".kvs");
    if !user_kvs_config_dir_path.exists() {
        std::fs::create_dir_all(&user_kvs_config_dir_path).unwrap();
    }
    Box::new(user_kvs_config_dir_path)
}
