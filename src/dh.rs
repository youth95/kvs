use static_dh_ecdh::ecdh::ecdh::{FromBytes, KeyExchange, PkP384, SkP384, ToBytes, ECDHNISTP384};

use crate::{utils::sha256, KVSResult};

pub struct KeyPair {
    sk: SkP384,
    pk: PkP384,
}

impl KeyPair {
    pub fn new() -> Self {
        let mut seed = [0u8; 32];
        for v in seed.iter_mut() {
            *v = rand::random();
        }
        let sk = ECDHNISTP384::<48>::generate_private_key(seed);
        let pk = ECDHNISTP384::<48>::generate_public_key(&sk);
        KeyPair { sk, pk }
    }

    pub fn get_pk(&self) -> Vec<u8> {
        Vec::from_iter(self.pk.to_bytes())
    }

    pub fn to_shared_secret(&self, pk_bytes: &[u8]) -> KVSResult<Vec<u8>> {
        let pk = PkP384::from_bytes(&pk_bytes)?;
        let ss =
            Vec::from_iter(ECDHNISTP384::<48>::generate_shared_secret(&self.sk, &pk)?.to_bytes());
        Ok(sha256(&ss))
    }
}

#[cfg(test)]
mod test {
    use super::KeyPair;

    #[test]
    fn test_dh() {
        let a = KeyPair::new();
        let b = KeyPair::new();
        assert_eq!(
            a.to_shared_secret(&b.get_pk()).expect("fail"),
            b.to_shared_secret(&a.get_pk()).expect("fail")
        );
    }
}
