use reqwest::blocking::Client;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackerResponse {
    pub peers: Vec<SocketAddrV4>,
}

// Define PeerDict for non-compact format
#[derive(Debug, Deserialize)]
struct PeerDict {
    ip: String,
    port: u16,
}

// Visitor to handle both compact and non-compact peer formats
struct PeerVisitor;

impl<'de> de::Visitor<'de> for PeerVisitor {
    type Value = Vec<SocketAddrV4>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte string (compact) or a list of peer dictionaries (non-compact)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut peers = Vec::new();
        while let Some(peer) = seq.next_element::<PeerDict>()? {
            if let Ok(ip) = peer.ip.parse::<Ipv4Addr>() {
                peers.push(SocketAddrV4::new(ip, peer.port));
            }
        }
        Ok(peers)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() % 6 != 0 {
            return Err(E::custom("Invalid compact peer list length"));
        }
        let peers = v
            .chunks_exact(6)
            .map(|chunk| {
                let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
                let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                SocketAddrV4::new(ip, port)
            })
            .collect();
        Ok(peers)
    }
}

fn deserialize_peers<'de, D>(deserializer: D) -> Result<Vec<SocketAddrV4>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(PeerVisitor)
}

#[derive(Debug, Deserialize)]
struct RawTrackerResponse {
    #[serde(deserialize_with = "deserialize_peers")]
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
    let infohash_encoded = urlencoding::encode_binary(&infohash);
    let peer_id_encoded = urlencoding::encode_binary(&peer_id);

    let url = format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact=1",
        announce, infohash_encoded, peer_id_encoded, port, uploaded, downloaded, left
    );

    let response = client.get(&url).send()?.bytes()?;
    eprintln!("Raw tracker response (hex): {:?}", hex::encode(&response));
    let raw_response: RawTrackerResponse = serde_bencode::from_bytes(&response).map_err(|e| {
        eprintln!("Deserialization error in tracker response: {:?}", e);
        e
    })?;
    Ok(TrackerResponse {
        peers: raw_response.peers,
    })
}
