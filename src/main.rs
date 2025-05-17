use crate::Peers::peer::Handshake;
use crate::Torrentfile::magnet::parse_magnet_link;
use crate::Torrentfile::torrent::TorrentFile;
use crate::Tracker::tracker::query_http_tracker;
use crate::Tracker::udp::query_udp_tracker;
use crate::bittorent::connect_to_peer;
use rand::Rng;

#[allow(non_snake_case)]
mod Bencode;
#[allow(non_snake_case)]
mod Peers;
#[allow(non_snake_case)]
mod Torrentfile;
#[allow(non_snake_case)]
mod Tracker;
mod bittorent;

#[allow(unused_variables)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let peer_id: [u8; 20] = rand::thread_rng().r#gen();
    let port = 6881;

    let torrent_path = "src/debian.torrent";
    let torrent_file = TorrentFile::from_file(torrent_path).map_err(|e| {
        eprintln!("Torrent file error: {:?}", e);
        panic!("Failed to parse torrent file: {:?}", e);
    })?;
    let info_hash = torrent_file.info_hash;
    let announce = torrent_file.torrent.announce.clone();

    // Example magnet link parsing (optional)
    let magnet = parse_magnet_link(
        "magnet:?xt=urn:btih:1234567890123456789012345678901234567890&tr=udp://tracker.example.com:6969",
    )?;
    let info_hash_magnet = magnet.infohash;
    let trackers = magnet.trackers;

    // Query tracker
    let tracker_response = if announce.starts_with("http") {
        query_http_tracker(
            &announce,
            info_hash,
            peer_id,
            port,
            0,
            torrent_file.torrent.info.length,
            0,
        )
        .map_err(|e| {
            eprintln!("HTTP tracker error: {:?}", e);
            panic!("Failed to query HTTP tracker: {:?}", e);
        })?
    } else if announce.starts_with("udp") {
        query_udp_tracker(&announce, info_hash, peer_id, port)
            .await
            .map_err(|e| {
                eprintln!("UDP tracker error: {:?}", e);
                panic!("Failed to query UDP tracker: {:?}", e);
            })?
    } else {
        return Err("Unsupported tracker protocol".into());
    };

    // Connect to a peer
    if let Some(peer) = tracker_response.peers.first() {
        let mut stream = connect_to_peer(*peer).await?;
        let handshake = Handshake::new(info_hash, peer_id);
        Handshake::send_handshake(&mut stream, &handshake).await?;
        Handshake::send_interested(&mut stream).await?;
        Handshake::request_piece(&mut stream, 0, 0, 16 * 1024).await?;
    }

    Ok(())
}
