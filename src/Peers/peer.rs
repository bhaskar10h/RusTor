use anyhow::{Result, anyhow};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub struct Handshake {
    pub length: u8,
    pub protocol: [u8; 19],
    pub reserved: [u8; 8],
    pub infohash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(infohash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            length: 19,
            protocol: *b"BitTorrent protocol",
            reserved: [0; 8],
            infohash,
            peer_id,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.push(self.length);
        bytes.extend_from_slice(&self.protocol);
        bytes.extend_from_slice(&self.reserved);
        bytes.extend_from_slice(&self.infohash);
        bytes.extend_from_slice(&self.peer_id);
        bytes
    }

    pub async fn send_handshake(stream: &mut TcpStream, handshake: &Handshake) -> Result<()> {
        stream.write_all(&handshake.to_bytes()).await?;
        let mut response = [0u8; 68];
        stream.read_exact(&mut response).await?;
        if response[0] != 19 || &response[1..20] != b"BitTorrent protocol" {
            return Err(anyhow!("Invalid handshake response"));
        }
        if &response[28..48] != &handshake.infohash {
            return Err(anyhow!("Mismatched hash in handshake!..."));
        }
        Ok(())
    }

    pub async fn send_interested(stream: &mut TcpStream) -> Result<()> {
        let message = vec![0, 0, 0, 1, 2]; // ID 2: Interested
        stream.write_all(&message).await?;
        Ok(())
    }

    pub async fn request_piece(
        stream: &mut TcpStream,
        index: u32,
        begin: u32,
        length: u32,
    ) -> Result<()> {
        let mut message = Vec::with_capacity(17);
        message.extend_from_slice(&13u32.to_be_bytes());
        message.push(6);
        message.extend_from_slice(&index.to_be_bytes());
        message.extend_from_slice(&begin.to_be_bytes());
        message.extend_from_slice(&length.to_be_bytes());
        stream.write_all(&message).await?;
        Ok(())
    }
}
