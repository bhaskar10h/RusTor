use std::net::{Ipv4Addr, SocketAddrV4};

// use crate::Tracker::tracker::TrackerResponse;
use anyhow::{Result, anyhow};
use rand;
use tokio::net::UdpSocket;

#[derive(Debug)]
pub struct TrackerResponse {
    pub peers: Vec<SocketAddrV4>,
}

pub async fn query_udp_tracker(
    announce: &str,
    infohash: [u8; 20],
    peer_id: [u8; 20],
    port: u16,
    left: u64,
) -> Result<TrackerResponse> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let url = url::Url::parse(announce)?;
    let addr = format!(
        "{}:{}",
        url.host_str().ok_or_else(|| anyhow!("Invalid host"))?,
        url.port().ok_or_else(|| anyhow!("Missing port"))?
    );

    // Connection request
    let mut connect_request = Vec::with_capacity(16);
    connect_request.extend_from_slice(&0x41727101980u64.to_be_bytes());
    connect_request.extend_from_slice(&0u32.to_be_bytes());
    let txn_id: u32 = rand::random();
    connect_request.extend_from_slice(&txn_id.to_be_bytes());
    socket.send_to(&connect_request, &addr).await?;

    let mut buf = [0u8; 2048];
    let (len, _) = socket.recv_from(&mut buf).await?;
    if len < 16 || &buf[0..4] != &0u32.to_be_bytes() || &buf[4..8] != &txn_id.to_be_bytes() {
        return Err(anyhow!("Invalid connection response!..."));
    }
    let connection_id = &buf[8..16];

    // Announce request
    let mut announce_request = Vec::with_capacity(98);
    announce_request.extend_from_slice(connection_id); // Connection ID
    announce_request.extend_from_slice(&1u32.to_be_bytes()); // Action = 1: Announce
    let txn_id2: u32 = rand::random();
    announce_request.extend_from_slice(&txn_id2.to_be_bytes());
    announce_request.extend_from_slice(&infohash);
    announce_request.extend_from_slice(&peer_id);
    announce_request.extend_from_slice(&0u64.to_be_bytes()); // downloaded
    announce_request.extend_from_slice(&left.to_be_bytes()); // left
    announce_request.extend_from_slice(&0u64.to_be_bytes()); // uploaded
    announce_request.extend_from_slice(&0u32.to_be_bytes()); // event = 0
    announce_request.extend_from_slice(&0u32.to_be_bytes()); // IP addr = 0
    announce_request.extend_from_slice(&0u32.to_be_bytes()); // key = 0
    announce_request.extend_from_slice(&(!0u32).to_be_bytes()); // num want = -1
    announce_request.extend_from_slice(&port.to_be_bytes());

    socket.send_to(&announce_request, &addr).await?;

    let (alen, _) = socket.recv_from(&mut buf).await?;
    if alen < 20 || &buf[0..4] != &1u32.to_be_bytes() || &buf[4..8] != &txn_id2.to_be_bytes() {
        return Err(anyhow!("Invalid announce response"));
    }

    // Parse peers from response
    let peers_bytes = &buf[20..alen];
    let mut peers = Vec::new();
    for chunk in peers_bytes.chunks_exact(6) {
        let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
        let port = u16::from_be_bytes([chunk[4], chunk[5]]);
        peers.push(SocketAddrV4::new(ip, port));
    }
    Ok(TrackerResponse { peers })
}
