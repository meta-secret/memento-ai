use sha2::{Digest, Sha256};
use std::fmt::Display;
use rand::distributions::Alphanumeric;
use rand::Rng;
use uuid::Uuid;
use crate::common::encoding::base64::Base64Text;

const SEED_LENGTH: usize = 64;

pub struct Sha256Generator {}

impl Sha256Generator {
    pub fn generate_hex_str() -> String {
        let seed: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(SEED_LENGTH)
            .map(char::from)
            .collect();

        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());

        hex::encode(hasher.finalize())
    }
}

pub struct U64Generator {
    
}

impl U64Generator {
    pub fn generate_u64() -> u64 {
        let uuid = Uuid::new_v4();
        let (high, _) = uuid.as_u64_pair();
        high
    }
}

pub struct UuidGenerator {
    uuid: Uuid,
}

impl UuidGenerator {
    /// Generate random uuid encoded with base64 url encoding
    pub fn rand_uuid_b64_url_enc() -> Base64Text {
        let uuid = Uuid::new_v4();
        let uuid_bytes = uuid.as_bytes().as_slice();
        Base64Text::from(uuid_bytes)
    }

    pub fn generate_uuid_b64_url_enc(value: String) -> String {
        let hash = Sha256::digest(value.as_bytes());
        let uuid = Uuid::from_slice(&hash.as_slice()[..16]).unwrap();
        let Base64Text(base64_text) = Base64Text::from(uuid.as_bytes().as_slice());
        base64_text
    }

    pub fn generate_uuid(text: &str) -> Uuid {
        let hash = Sha256::digest(text.as_bytes());
        let uuid = Uuid::from_slice(&hash.as_slice()[..16]).expect("Critical error");
        uuid
    }
}

impl From<&str> for UuidGenerator {
    fn from(text: &str) -> Self {
        UuidGenerator {
            uuid: UuidGenerator::generate_uuid(text),
        }
    }
}

impl From<UuidGenerator> for String {
    fn from(generator: UuidGenerator) -> Self {
        generator.uuid.to_string()
    }
}

impl Display for UuidGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uuid.to_string())
    }
}
