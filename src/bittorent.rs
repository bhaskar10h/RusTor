use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;

#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub name: String,
    pub length: u64,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    pub pieces: Vec<u8>, // Using Vec<u8> for byte array as per previous fix
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Torrent {
    pub announce: String,
    pub info: TorrentInfo,
}

pub async fn connect_to_peer(
    addr: std::net::SocketAddrV4,
) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(addr).await?;
    Ok(stream)
}
