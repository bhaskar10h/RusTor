use crate::bittorent::{Torrent, TorrentInfo};
use serde::{Deserialize, Serialize};
use serde_bencode::{self, value::Value};
use std::fs;

pub fn decode_torrent_file(path: &str) -> Result<Torrent, Box<dyn std::error::Error>> {
    let content = fs::read(path)?;
    let torrent: Torrent = serde_bencode::from_bytes(&content)?;
    Ok(torrent)
}

pub fn decode_bencode(data: &[u8]) -> Result<Value, Box<dyn std::error::Error>> {
    let value = serde_bencode::from_bytes(data)?;
    Ok(value)
}
