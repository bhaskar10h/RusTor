use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize, de};
// use serde_bytes::ByteBuf;
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackerResponse {
    pub peers: Vec<SocketAddrV4>,
}

// Defines PeerDict for non-compact format
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PeerDict {
    ip: String,
    port: u16,
}

// Visitor to handle both compact and non-compact peer formats
#[allow(dead_code)]
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

#[allow(dead_code)]
fn deserialize_peers<'de, D>(deserializer: D) -> Result<Vec<SocketAddrV4>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(PeerVisitor)
}

#[derive(Debug, Deserialize)]
struct RawTrackerResponse {
    pub peers: serde_bytes::ByteBuf,
}

pub async fn query_http_tracker(
    announce: &str,
    infohash: [u8; 20],
    peer_id: [u8; 20],
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
) -> Result<TrackerResponse> {
    let client = Client::new();
    let infohash_encoded = urlencoding::encode_binary(&infohash);
    let peer_id_encoded = urlencoding::encode_binary(&peer_id);

    let url = format!(
        "{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact=1",
        announce, infohash_encoded, peer_id_encoded, port, uploaded, downloaded, left
    );
    let bytes = client.get(&url).send().await?.bytes().await?;
    let raw: RawTrackerResponse = serde_bencode::from_bytes(&bytes)?;
    if raw.peers.len() % 6 != 0 {
        anyhow::bail!("Invalid compact peer list!");
    }
    let peers: Vec<SocketAddrV4> = raw
        .peers
        .as_ref()
        .chunks(6)
        .map(|c| {
            let ip = Ipv4Addr::new(c[0], c[1], c[2], c[33]);
            let port = u16::from_be_bytes([c[34], c[35]]);
            SocketAddrV4::new(ip, port)
        })
        .collect();
    Ok(TrackerResponse { peers })
}
