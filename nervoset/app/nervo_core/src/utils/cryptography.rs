use std::fmt::Display;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct UuidGenerator {
    uuid: Uuid
}

impl From<&str> for UuidGenerator {
    fn from(text: &str) -> Self {
        UuidGenerator {
            uuid: generate_uuid(text),
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

pub fn generate_uuid(text: &str) -> Uuid {
    let hash = Sha256::digest(text.as_bytes());
    let uuid = Uuid::from_slice(&hash.as_slice()[..16]).expect("Critical error");
    uuid
}
