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
        .map(|x| format!("{:02x}", x))
        .collect::<Vec<String>>()
        .join("")
}

pub fn to_addr(payload: &[u8]) -> String {
    let data = sgin(payload);
    format!("0x{}", to_u8str(&data))
}

#[cfg(test)]
mod test {
    use super::to_u8str;

    #[test]
    fn test_to_u8str() {
        assert_eq!(to_u8str(&vec![0]), "00");
        assert_eq!(to_u8str(&vec![0xff]), "ff");
        assert_eq!(to_u8str(&vec![1]), "01");
    }
}
