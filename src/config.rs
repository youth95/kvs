use std::{io::Write, net::TcpStream, path::PathBuf, time::Duration};

use crate::{
    actions::{FetchTokenAction, KVSToken},
    errors::KVSResult,
    kv_session::KVSSession,
    secret::Secret,
    spec::KVSAction,
};

pub fn get_or_create_token(
    repository: &String,
    froce_create: bool,
) -> KVSResult<(KVSToken, String)> {
    let user_token_file_path = &get_or_create_user_config_dir()?.join("token");

    if user_token_file_path.exists() && froce_create == false {
        Ok((
            bincode::deserialize(&std::fs::read(user_token_file_path)?)?,
            user_token_file_path.display().to_string(),
        ))
    } else {
        let secret = get_or_create_secret()?;
        let mut session = {
            let stream = TcpStream::connect(repository)?;
            stream.set_read_timeout(Some(Duration::from_millis(1000)))?;
            KVSSession::new(stream)
        }?;
        let token = FetchTokenAction {
            pub_key: secret.pub_key_bits.to_vec(),
        }
        .request(&mut session)?;

        let token_bytes = bincode::serialize(&token)?;
        std::fs::write(&user_token_file_path, &token_bytes)?;
        Ok((token, user_token_file_path.display().to_string()))
    }
}

pub fn get_or_create_secret() -> KVSResult<Secret> {
    let user_kvs_config_dir_path = get_or_create_user_config_dir()?;
    let user_secret_file_path = user_kvs_config_dir_path.join("secret");
    if user_secret_file_path.exists() {
        Ok(Secret::from(std::fs::read_to_string(
            user_secret_file_path,
        )?))
    } else {
        let secret = Secret::default();
        std::fs::create_dir_all(user_kvs_config_dir_path)?;
        let mut file = std::fs::File::create(user_secret_file_path)?;
        file.write_all(&secret.to_string().as_bytes())?;
        Ok(secret)
    }
}

pub fn get_or_create_jwt_secret(froce_create: bool) -> KVSResult<Vec<u8>> {
    let user_kvs_config_dir_path = get_or_create_user_config_dir()?;
    let jwt_secret_file_path = user_kvs_config_dir_path.join("jwt_secret");

    if jwt_secret_file_path.exists() && froce_create == false {
        Ok(std::fs::read(jwt_secret_file_path)?)
    } else {
        let jwt_secret = (0..256).map(|_| rand::random::<u8>()).collect::<Vec<u8>>();
        std::fs::write(jwt_secret_file_path, &jwt_secret)?;
        Ok(jwt_secret)
    }
}

pub fn get_or_create_user_config_dir() -> KVSResult<PathBuf> {
    let user_kvs_config_dir_path = dirs::home_dir().unwrap().join(".kvs");
    if !user_kvs_config_dir_path.exists() {
        std::fs::create_dir_all(&user_kvs_config_dir_path)?;
    }
    Ok(user_kvs_config_dir_path)
}

pub fn get_or_create_data_dir() -> KVSResult<PathBuf> {
    let user_kvs_config_dir_path = dirs::data_dir().unwrap().join(".kvs_data");
    if !user_kvs_config_dir_path.exists() {
        std::fs::create_dir_all(&user_kvs_config_dir_path)?;
    }
    Ok(user_kvs_config_dir_path)
}
