use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TorrentInfo {
    pub name: String,
    pub length: u64,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    pub pieces: Vec<u8>, // Raw concatenated 20-byte SHA-1 hashes
}

impl TorrentInfo {
    // Note: If not using this method yet, suppress warning
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Torrent {
    pub announce: String,
    #[serde(default)]
    pub announce_list: Option<Vec<Vec<String>>>, // Handle announce-list
    pub info: TorrentInfo,
}

pub async fn connect_to_peer(
    addr: std::net::SocketAddrV4,
) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(addr).await?;
    Ok(stream)
}
