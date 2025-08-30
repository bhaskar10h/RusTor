use anyhow::{Result, anyhow};
use sha1::{Digest, Sha1};
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

#[repr(u8)]
pub enum MsgId {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
    Port = 9,
}

// 4-byte big-endian length, then optional 1-byte id, then payload (len==0 => keep-alive)
pub async fn read_msg(stream: &mut TcpStream) -> Result<Option<(u8, Vec<u8>)>> {
    use tokio::io::AsyncReadExt;
    let mut len_buf = [0u8; 4];
    if stream.read_exact(&mut len_buf).await.is_err() {
        return Ok(None);
    }
    let len = u32::from_be_bytes(len_buf);
    if len == 0 {
        return Ok(Some((255, Vec::new())));
    }
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).await?;
    let id = buf.clone();
    Ok(Some((id[0], buf[1..].to_vec())))
}

pub async fn send_interested(stream: &mut TcpStream) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    stream
        .write_all(&[0, 0, 0, 1, MsgId::Interested as u8])
        .await?;
    Ok(())
}

pub async fn send_request(
    stream: &mut TcpStream,
    index: u32,
    begin: u32,
    length: u32,
) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut m = Vec::with_capacity(17);
    m.extend_from_slice(&13u32.to_be_bytes());
    m.push(MsgId::Request as u8);
    m.extend_from_slice(&index.to_be_bytes());
    m.extend_from_slice(&begin.to_be_bytes());
    m.extend_from_slice(&length.to_be_bytes());
    stream.write_all(&m).await?;
    Ok(())
}

pub async fn download_first_piece(
    stream: &mut TcpStream,
    piece_len: u64,
    total_len: u64,
    piece_hash: [u8; 20],
) -> Result<Vec<u8>> {
    let block: u32 = 16 * 1024;
    let mut choked = true;
    let mut have_piece0 = false;

    // Announce interest and wait for Unchoke + Bitfield/Have
    send_interested(stream).await?;
    while choked || !have_piece0 {
        if let Some((id, payload)) = read_msg(stream).await? {
            match id {
                x if x == MsgId::Choke as u8 => {
                    choked = true;
                }
                x if x == MsgId::Unchoke as u8 => {
                    choked = false;
                }
                x if x == MsgId::Have as u8 => {
                    if payload.len() >= 4 {
                        let idx = u32::from_be_bytes(payload[0..4].try_into().unwrap());
                        if idx == 0 {
                            have_piece0 = true;
                        }
                    }
                }
                x if x == MsgId::Bitfield as u8 => {
                    // Piece 0 corresponds to MSB of first byte in bitfield
                    if !payload.is_empty() && (payload[0] & 0b1000_0000) != 0 {
                        have_piece0 = true;
                    }
                }
                255 => {} // keep-alive
                _ => {}
            }
        } else {
            return Err(anyhow!("peer disconnected before unchoke/bitfield"));
        }
    }

    // Piece 0 length (last piece may be shorter; piece 0 is min(piece_len, total_len))
    let this_piece_len = std::cmp::min(piece_len, total_len) as usize;
    let mut buf = vec![0u8; this_piece_len];

    // Pipeline requests for 16 KiB blocks
    let mut off = 0usize;
    while off < this_piece_len {
        let want = std::cmp::min(block as usize, this_piece_len - off) as u32;
        send_request(stream, 0, off as u32, want).await?;
        off += want as usize;
    }

    // Collect Piece messages: payload = index(4) begin(4) block(N)
    let mut received = 0usize;
    while received < this_piece_len {
        if let Some((id, payload)) = read_msg(stream).await? {
            if id == MsgId::Piece as u8 {
                if payload.len() < 8 {
                    continue;
                }
                let index = u32::from_be_bytes(payload[0..4].try_into().unwrap());
                if index != 0 {
                    continue;
                }
                let begin = u32::from_be_bytes(payload[4..8].try_into().unwrap()) as usize;
                if begin >= buf.len() {
                    continue;
                }
                let data = &payload[8..];
                let end = std::cmp::min(begin + data.len(), buf.len());
                let n = end - begin;
                buf[begin..end].copy_from_slice(&data[..n]);
                received += n;
            }
        } else {
            return Err(anyhow!("peer disconnected while downloading"));
        }
    }

    // Verify SHA-1 matches piece 0 hash from metainfo
    let mut h = Sha1::new();
    h.update(&buf);
    let got: [u8; 20] = h.finalize().into();
    if got != piece_hash {
        return Err(anyhow!("piece 0 hash mismatch"));
    }
    Ok(buf)
}
