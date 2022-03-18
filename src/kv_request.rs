use crate::{
    actions::{Cat, CreateKeyValue, FetchToken, KVSToken, Value},
    spec::KVSRequest,
    KVSResult, KVSSession, Secret,
};

pub fn fetch_token(kvs_session: KVSSession, secret: Secret) -> KVSResult<KVSToken> {
    FetchToken {
        pub_key: secret.pub_key_bits.to_vec(),
    }
    .request(kvs_session, Some(&secret))
}

pub fn create_key_value(
    kvs_session: KVSSession,
    token: &KVSToken,
    key: &String,
    value: &String,
) -> KVSResult<()> {
    CreateKeyValue {
        token: token.clone(),
        key: key.to_string(),
        value: Value::Text(value.as_bytes().to_vec()),
    }
    .request(kvs_session, None)
}

pub fn cat(kvs_session: KVSSession, token: &KVSToken, key: &String) -> KVSResult<Vec<u8>> {
    Cat {
        token: token.clone(),
        key: key.to_string(),
    }
    .request(kvs_session, None)
}
