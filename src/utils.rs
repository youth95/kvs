use sha2::Digest;

pub fn sha256(payload: &[u8]) -> Vec<u8> {
    let mut sha_256_worker = sha2::Sha256::new();
    sha_256_worker.update(payload);
    sha_256_worker.finalize().as_slice().to_vec()
}

pub fn ripemd_160(payload: &[u8]) -> Vec<u8> {
    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(payload);
    hasher.finalize().to_vec()
}

pub fn sgin(payload: &[u8]) -> Vec<u8> {
    ripemd_160(&sha256(payload))
}

pub fn to_u8str(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|x| format!("{:x}", x))
        .collect::<Vec<String>>()
        .join("")
}

pub fn to_addr(payload: &[u8]) -> String {
    let data = sgin(&ripemd_160(payload));
    format!("0x{}", to_u8str(&data))
}
