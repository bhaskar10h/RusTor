use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackerResponse {
    pub peers: Vec<SocketAddrV4>,
}

pub fn query_http_tracker(
    announce: &str,
    infohash: [u8; 20],
    peer_id: [u8; 20],
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
) -> Result<TrackerResponse, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact=1",
        announce,
        urlencoding::encode(&String::from_utf8(infohash.to_vec())?),
        urlencoding::encode(&String::from_utf8(peer_id.to_vec())?),
        port,
        uploaded,
        downloaded,
        left
    );

    let response = client.get(&url).send()?.bytes()?;
    let tracker_response = serde_bencode::from_bytes(&response)?;
    Ok(tracker_response)
}
