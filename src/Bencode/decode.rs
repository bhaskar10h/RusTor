use crate::bittorent::Torrent;
use serde_bencode::{self};
use std::fs;

pub fn decode_torrent_file(path: &str) -> Result<Torrent, Box<dyn std::error::Error>> {
    let content = fs::read(path)?;
    let torrent: Torrent = serde_bencode::from_bytes(&content)?;
    Ok(torrent)
}
