#[derive(Debug)]
pub enum KVSError {
    IOError(std::io::Error),
    AESGcmError(aes_gcm::Error),
    RSAError(rsa::errors::Error),
    TryFromSliceError(std::array::TryFromSliceError),
    LogicError(String),
    BincodeError(Box<bincode::ErrorKind>),
    MimeFromStrError(mime::FromStrError),
}

pub type KVSResult<T> = Result<T, KVSError>;

impl std::error::Error for KVSError {}

impl std::fmt::Display for KVSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::KVSError::*;
        match &self {
            IOError(ref e) => write!(f, "IO error: {}", e),
            AESGcmError(ref e) => write!(f, "AESGcmError Error: {}", e),
            TryFromSliceError(ref e) => write!(f, "TryFromSlice Error: {}", e),
            LogicError(ref e) => write!(f, "Logic Error: {}", e),
            BincodeError(ref e) => write!(f, "Bincode Error: {}", e),
            RSAError(ref e) => write!(f, "RSA Error: {}", e),
            MimeFromStrError(ref e) => write!(f, "Mime FromStrError: {}", e),
        }
    }
}

impl From<std::io::Error> for KVSError {
    fn from(io_error: std::io::Error) -> Self {
        KVSError::IOError(io_error)
    }
}

impl From<aes_gcm::Error> for KVSError {
    fn from(aes_gcm_error: aes_gcm::Error) -> Self {
        KVSError::AESGcmError(aes_gcm_error)
    }
}

impl From<std::array::TryFromSliceError> for KVSError {
    fn from(try_from_slice_error: std::array::TryFromSliceError) -> Self {
        KVSError::TryFromSliceError(try_from_slice_error)
    }
}

impl From<rsa::errors::Error> for KVSError {
    fn from(rsa_error: rsa::errors::Error) -> Self {
        KVSError::RSAError(rsa_error)
    }
}

impl From<Box<bincode::ErrorKind>> for KVSError {
    fn from(bincode_error: Box<bincode::ErrorKind>) -> Self {
        KVSError::BincodeError(bincode_error)
    }
}

impl From<mime::FromStrError> for KVSError {
    fn from(mime_from_str_error: mime::FromStrError) -> Self {
        KVSError::MimeFromStrError(mime_from_str_error)
    }
}
