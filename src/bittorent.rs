use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub length: u64,
    pub path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TorrentInfo {
    pub name: String,
    #[serde(default)]
    pub length: u64,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    pub pieces: Vec<u8>, // Raw concatenated 20-byte SHA-1 hashes
    #[serde(default)]
    pub files: Option<Vec<FileEntry>>,
}

#[allow(dead_code)]
impl TorrentInfo {
    // If not using this method yet, suppress warning
    #[allow(dead_code)]
    pub fn piece_hashes(&self) -> Result<Vec<[u8; 20]>, Box<dyn std::error::Error>> {
        if self.pieces.len() % 20 != 0 {
            return Err("Invalid pieces length: not divisible by 20".into());
        }
        let hashes: Vec<[u8; 20]> = self
            .pieces
            .chunks_exact(20)
            .map(|chunk| {
                let mut hash = [0u8; 20];
                hash.copy_from_slice(chunk);
                hash
            })
            .collect();
        Ok(hashes)
    }

    pub fn total_length(&self) -> u64 {
        if let Some(len) = Some(self.length) {
            len
        } else {
            self.files
                .as_ref()
                .map(|v| v.iter().map(|f| f.length).sum())
                .unwrap_or(0)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Torrent {
    pub announce: String,
    #[serde(default)]
    pub announce_list: Option<Vec<Vec<String>>>, // Handle announce-list
    pub info: TorrentInfo,
}

pub async fn connect_to_peer(addr: std::net::SocketAddrV4) -> Result<TcpStream> {
    let stream = TcpStream::connect(addr).await?;
    Ok(stream)
}
