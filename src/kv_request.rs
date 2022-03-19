use crate::{
    actions::{Cat, CreateKeyValue, FetchToken, KVSToken, KeyMeta},
    config::get_or_create_secret,
    spec::KVSRequest,
    KVSResult, KVSSession, Secret,
};

pub fn fetch_token(kvs_session: &mut KVSSession, secret: &Secret) -> KVSResult<KVSToken> {
    FetchToken {
        pub_key: secret.pub_key_bits.to_vec(),
    }
    .request(kvs_session, Some(secret))
}

pub fn create_key_value(
    kvs_session: &mut KVSSession,
    token: &KVSToken,
    key: &String,
    value: &String,
    private: &bool,
) -> KVSResult<()> {
    let value = value.as_bytes().to_vec();
    let size = value.len() as u64;
    let owner = token.id.clone();

    let rand = if *private {
        Some((0..32).map(|_| rand::random::<u8>()).collect::<Vec<u8>>())
    } else {
        None
    };
    CreateKeyValue {
        token: token.clone(),
        key: key.to_string(),
        value,
        meta: KeyMeta {
            mime: "text/plain".to_string(),
            size,
            owner,
            name: key.to_string(),
            rand,
        },
    }
    .request(kvs_session, Some(get_or_create_secret()))
}

pub fn cat(kvs_session: &mut KVSSession, token: &KVSToken, key: &String) -> KVSResult<Vec<u8>> {
    Cat {
        token: token.clone(),
        key: key.to_string(),
    }
    .request(kvs_session, Some(get_or_create_secret()))
}
