use aes_gcm::{aead::Aead, Aes256Gcm, Key, NewAead, Nonce};
use std::net::TcpStream;

use crate::{
    errors::{KVSError, KVSResult},
    secret::KeyPair,
    spec::Session,
};

pub struct KVSSession {
    stream: TcpStream,
    cipher: Aes256Gcm,
}

pub const NONCE: &[u8] = b"kvskvskvskvs";

impl KVSSession {
    pub fn to<'a, T: serde::de::Deserialize<'a>>(bytes: &'a [u8]) -> KVSResult<T> {
        let data: T = bincode::deserialize(bytes)?;
        Ok(data)
    }
}

impl KVSSession {
    pub fn new(stream: TcpStream) -> KVSResult<Self> {
        let kp = KeyPair::new();
        // 通道建立
        bincode::serialize_into(&stream, &kp.get_pk())?;
        let pk_bytes: Vec<u8> = bincode::deserialize_from(&stream)?;
        let shared_secret = kp.to_shared_secret(&pk_bytes)?;

        let key = Key::from_slice(shared_secret.as_slice());
        let cipher = Aes256Gcm::new(key);
        Ok(KVSSession { stream, cipher })
    }
}

impl Session for KVSSession {
    fn read_vec(&mut self) -> KVSResult<Vec<u8>> {
        let payload: Vec<u8> = bincode::deserialize_from(&self.stream)?;
        match self.cipher.decrypt(Nonce::from_slice(NONCE), &*payload) {
            Ok(data) => Ok(data),
            Err(err) => Err(KVSError::AESGcmError(err)),
        }
    }

    fn write_vec(&mut self, payload: &[u8]) -> KVSResult<()> {
        let data = self.cipher.encrypt(Nonce::from_slice(NONCE), &*payload)?;
        bincode::serialize_into(&self.stream, &data)?;
        Ok(())
    }

    fn write<T: ?Sized>(&mut self, payload: &T) -> KVSResult<()>
    where
        T: serde::Serialize,
    {
        let payload = bincode::serialize(payload)?;
        let data = self.cipher.encrypt(Nonce::from_slice(NONCE), &*payload)?;
        bincode::serialize_into(&self.stream, &data)?;
        Ok(())
    }
}

#[cfg(test)]
pub struct MockSession {
    stream: std::fs::File,
    cipher: Aes256Gcm,
}
#[cfg(test)]
impl MockSession {
    pub fn new() -> KVSResult<Self> {
        if !std::path::Path::new("mock_stream").exists() {
            std::fs::File::create("mock_stream")?;
        }
        let stream = std::fs::File::open("mock_stream")?;
        // 通道建立
        let s = b"12345678900987654321123456789098".to_vec();
        let key = Key::from_slice(s.as_slice());
        let cipher = Aes256Gcm::new(key);
        Ok(MockSession { stream, cipher })
    }
}
#[cfg(test)]
impl Session for MockSession {
    fn read_vec(&mut self) -> KVSResult<Vec<u8>> {
        let payload: Vec<u8> = bincode::deserialize_from(&self.stream)?;
        match self.cipher.decrypt(Nonce::from_slice(NONCE), &*payload) {
            Ok(data) => Ok(data),
            Err(err) => Err(KVSError::AESGcmError(err)),
        }
    }

    fn write_vec(&mut self, payload: &[u8]) -> KVSResult<()> {
        let data = self.cipher.encrypt(Nonce::from_slice(NONCE), &*payload)?;
        bincode::serialize_into(&self.stream, &data)?;
        Ok(())
    }

    fn write<T: ?Sized>(&mut self, payload: &T) -> KVSResult<()>
    where
        T: serde::Serialize,
    {
        let payload = bincode::serialize(payload)?;
        let data = self.cipher.encrypt(Nonce::from_slice(NONCE), &*payload)?;
        bincode::serialize_into(&self.stream, &data)?;
        Ok(())
    }
}
