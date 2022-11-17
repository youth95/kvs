use thiserror::Error;

#[derive(Debug, Error)]
pub enum KVSError {
    #[error("IO error: {0}")]
    IOError(#[from]std::io::Error),
    #[error("AESGcmError Error: {0}")]
    AESGcmError(aes_gcm::Error),
    #[error("TryFromSlice Error: {0}")]
    RSAError(#[from] rsa::errors::Error),
    #[error("TryFromSlice Error: {0}")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error("Bincode Error: {0}")]
    LogicError(String),
    #[error("RSA Error: {0}")]
    BincodeError(#[from] Box<bincode::ErrorKind>),
    #[error("MimeFromStr Error: {0}")]
    MimeFromStrError(#[from] mime::FromStrError),
}

pub type KVSResult<T> = Result<T, KVSError>;

impl From<aes_gcm::Error> for KVSError {
    fn from(aes_gcm_error: aes_gcm::Error) -> Self {
        KVSError::AESGcmError(aes_gcm_error)
    }
}
