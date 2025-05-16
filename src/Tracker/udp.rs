use crate::Tracker::tracker::TrackerResponse;
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
        "{} : {}",
        url.host_str().ok_or("Invalid Host!")?,
        url.port().ok_or("Missing Port")?
    );
    let connect_request = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    socket.send_to(&connect_request, &addr).await?;
    let mut buf = [0u8; 16];
    socket.recv_from(&mut buf).await?;

    Ok(TrackerResponse { peers: vec![] })
}
