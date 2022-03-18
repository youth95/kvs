use aes_gcm::{aead::Aead, Aes256Gcm, Key, NewAead, Nonce};
use std::net::TcpStream;

use crate::{errors::KVSResult, KVSError, KeyPair};

pub struct KVSSession {
    stream: TcpStream,
    cipher: Aes256Gcm,
}

const NONCE: &[u8] = b"kvskvskvskvs";

impl KVSSession {
    pub fn new(stream: TcpStream) -> KVSResult<Self> {
        let kp = KeyPair::new();
        // 通道建立
        bincode::serialize_into(&stream, &kp.get_pk())?;
        let pk_bytes: Vec<u8> = bincode::deserialize_from(&stream)?;
        let shared_scret = kp.to_shared_secret(&pk_bytes)?;

        let key = Key::from_slice(shared_scret.as_slice());
        let cipher = Aes256Gcm::new(key);
        Ok(KVSSession { stream, cipher })
    }

    pub fn get_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    pub fn read_vec(&mut self) -> KVSResult<Vec<u8>> {
        let payload: Vec<u8> = bincode::deserialize_from(&self.stream)?;
        match self.cipher.decrypt(Nonce::from_slice(NONCE), &*payload) {
            Ok(data) => Ok(data),
            Err(err) => Err(KVSError::AESGcmError(err)),
        }
    }

    pub fn write_vec(&mut self, payload: &[u8]) -> KVSResult<()> {
        let data = self.cipher.encrypt(Nonce::from_slice(NONCE), &*payload)?;
        bincode::serialize_into(&self.stream, &data)?;
        Ok(())
    }

    pub fn write<T: ?Sized>(&mut self, payload: &T) -> KVSResult<()>
    where
        T: serde::Serialize,
    {
        let payload = bincode::serialize(payload)?;
        let data = self.cipher.encrypt(Nonce::from_slice(NONCE), &*payload)?;
        bincode::serialize_into(&self.stream, &data)?;
        Ok(())
    }

    pub fn to<'a, T: serde::de::Deserialize<'a>>(bytes: &'a [u8]) -> KVSResult<T> {
        let data: T = bincode::deserialize(bytes)?;
        Ok(data)
    }
}
