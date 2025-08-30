use anyhow::Result;
use serde::de::DeserializeOwned;
use serde_bencode::{self};
use std::fs;

pub fn decode_torrent_file<T: DeserializeOwned>(path: &str) -> Result<T> {
    let content = fs::read(path)?;
    let torrent: T = serde_bencode::from_bytes(&content)?;
    Ok(torrent)
}
