use sha2::{Digest, Sha256};
use uuid::Uuid;

pub fn generate_uuid(text: &str) -> anyhow::Result<Uuid> {
    let hash = Sha256::digest(text.as_bytes());
    let uuid = Uuid::from_slice(&hash.as_slice()[..16])?;
    Ok(uuid)
}
