use anyhow::Result;
use serde::Serialize;
use serde_bencode;

pub fn encode_bencode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let encoded = serde_bencode::to_bytes(value)?;
    Ok(encoded)
}
