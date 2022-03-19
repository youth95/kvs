use std::fmt::Display;

use rand::rngs::OsRng;
use rsa::pkcs1::{FromRsaPrivateKey, ToRsaPrivateKey};
use rsa::pkcs8::FromPublicKey;
use rsa::{pkcs8::ToPublicKey, RsaPrivateKey, RsaPublicKey};
use rsa::{PaddingScheme, PublicKey};

use crate::errors::KVSResult;


// pub const PUB_KEY_LENGTH: usize = 162;

pub struct Secret {
    pub pub_key_bits: Vec<u8>, // 162
    pub priv_key_bits: Vec<u8>,
}

impl Default for Secret {
    fn default() -> Self {
        let mut rng = OsRng;
        let bits = 1024;
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a priv key");
        let pub_key = RsaPublicKey::from(&priv_key);
        let pub_key_content = pub_key
            .to_public_key_pem()
            .expect("failed to get public key content");
        let pem = pem::parse(pub_key_content).expect("failed to parse pub key content");
        let pub_key_bits = pem.contents;
        // let pub_key = base64::encode(pub_key_bits.clone());
        // let addr = sha256::digest_bytes(&pub_key_bits);

        let priv_key_content = priv_key
            .to_pkcs1_der()
            .expect("failed to get priv key content");
        let priv_key_bits = priv_key_content.as_der().to_vec();
        // let pwd = base64::encode(priv_key_bits);

        Self {
            pub_key_bits,
            priv_key_bits,
        }
    }
}

impl Display for Secret {
    /**
     * pub_key
     * priv_key
     */
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "{}\n{}",
                base64::encode(self.pub_key_bits.clone()),
                base64::encode(self.priv_key_bits.clone())
            )
            .as_str(),
        )
    }
}

impl From<String> for Secret {
    fn from(content: String) -> Self {
        let mut it = content.split("\n");
        let pub_key_bits = base64::decode(it.next().unwrap()).unwrap();
        let priv_key_bits = base64::decode(it.next().unwrap()).unwrap();
        Secret {
            pub_key_bits,
            priv_key_bits,
        }
    }
}

impl Secret {
    pub fn encrypt_with_pub_key_bits(pub_key_bits: &[u8], message: &[u8]) -> Vec<u8> {
        let mut rng = OsRng;
        let pub_key =
            RsaPublicKey::from_public_key_der(pub_key_bits).expect("failed to parse pub key");
        let enc_data = pub_key
            .encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), message)
            .expect("failed to encrypt");
        enc_data
    }

    pub fn decrypt_width_priv_key_bits(
        priv_key_bits: &[u8],
        enc_data: &[u8],
    ) -> KVSResult<Vec<u8>> {
        let priv_key =
            RsaPrivateKey::from_pkcs1_der(priv_key_bits).expect("failed to parse priv key");
        // Decrypt
        let dec_data = priv_key.decrypt(PaddingScheme::new_pkcs1v15_encrypt(), &enc_data)?;
        Ok(dec_data)
    }
}


use static_dh_ecdh::ecdh::ecdh::{FromBytes, KeyExchange, PkP384, SkP384, ToBytes, ECDHNISTP384};

use crate::{utils::sha256};

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

