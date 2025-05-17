use crate::Tracker::tracker::TrackerResponse;
use rand::Rng;
use tokio::net::UdpSocket;

#[allow(unused_variables)]
pub async fn query_udp_tracker(
    announce: &str,
    infohash: [u8; 20],
    peer_id: [u8; 20],
    port: u16,
) -> Result<TrackerResponse, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let url = url::Url::parse(announce)?;
    let addr = format!(
        "{}:{}",
        url.host_str().ok_or("Invalid host")?,
        url.port().ok_or("Missing port")?
    );

    // Connection request
    let mut connect_request = vec![0x00, 0x00, 0x04, 0x17, 0x27, 0x10, 0x19, 0x80]; // Protocol ID
    let transaction_id: u32 = rand::thread_rng().r#gen();
    connect_request.extend_from_slice(&transaction_id.to_be_bytes());
    socket.send_to(&connect_request, &addr).await?;

    let mut buf = [0u8; 16];
    let (len, _) = socket.recv_from(&mut buf).await?;

    // Verify connection response
    if len < 16 || buf[0..4] != [0, 0, 0, 0] {
        return Err("Invalid connection response".into());
    }

    // Announce request
    let mut announce_request = Vec::new();
    announce_request.extend_from_slice(&buf[8..16]); // Connection ID
    announce_request.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]); // Action: Announce
    announce_request.extend_from_slice(&transaction_id.to_be_bytes());
    announce_request.extend_from_slice(&infohash);
    announce_request.extend_from_slice(&peer_id);
    announce_request.extend_from_slice(&0u64.to_be_bytes()); // Downloaded
    announce_request.extend_from_slice(&0u64.to_be_bytes()); // Left
    announce_request.extend_from_slice(&0u64.to_be_bytes()); // Uploaded
    announce_request.extend_from_slice(&0u32.to_be_bytes()); // Event
    announce_request.extend_from_slice(&0u32.to_be_bytes()); // IP address
    announce_request.extend_from_slice(&0u32.to_be_bytes()); // Key
    announce_request.extend_from_slice(&(!0u32).to_be_bytes()); // Num want
    announce_request.extend_from_slice(&port.to_be_bytes());

    socket.send_to(&announce_request, &addr).await?;

    let mut response = vec![0u8; 1024];
    let (len, _) = socket.recv_from(&mut response).await?;

    // Parse peers from response
    let mut peers = Vec::new();
    let peer_data = &response[20..len];
    for chunk in peer_data.chunks_exact(6) {
        let ip = std::net::Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
        let port = u16::from_be_bytes([chunk[4], chunk[5]]);
        peers.push(std::net::SocketAddrV4::new(ip, port));
    }

    Ok(TrackerResponse { peers })
}
