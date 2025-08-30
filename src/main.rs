use crate::Peers::peer::Handshake;
use crate::Torrentfile::magnet::parse_magnet_link;
use crate::Torrentfile::torrent::TorrentFile;
use crate::Tracker::{tracker::query_http_tracker, udp::query_udp_tracker};
use crate::bittorent::connect_to_peer;
use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};

#[allow(non_snake_case)]
mod Bencode;
#[allow(non_snake_case)]
mod Peers;
#[allow(non_snake_case)]
mod Torrentfile;
#[allow(non_snake_case)]
mod Tracker;
mod bittorent;

#[derive(Parser)]
#[command(name = "minibit", version, about = "Minimal Bittorrent client")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Download { torrent: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Download { torrent } => run_download(&torrent).await?,
    }
    Ok(())
}

async fn run_download(target: &str) -> Result<()> {
    use rand::Rng;

    let mut peer_id = [0u8; 20];

    let (info_hash, m_trackers, left_hints): ([u8; 20], Vec<String>, u64) =
        if target.starts_with("magnet:?") {
            let m = parse_magnet_link(target)?;
            (
                m.infohash,
                if m.trackers.is_empty() {
                    vec![]
                } else {
                    m.trackers
                },
                0,
            )
        } else {
            let tf = TorrentFile::from_file(target)?;
            (
                tf.info_hash,
                vec![tf.torrent.announce.clone()],
                tf.torrent.info.length,
            )
        };

    peer_id[0..8].copy_from_slice(b"-RS0001-");
    rand::thread_rng().fill(&mut peer_id[8..]);

    let announce = m_trackers
        .get(0)
        .ok_or_else(|| anyhow!("No tracker availabl; magnet without trackers rquires DHT/BEP-5"))?
        .clone();

    let port = 6881u16;
    let peers = if announce.starts_with("http") {
        query_http_tracker(&announce, info_hash, peer_id, port, 0, 0, left_hints)
            .await?
            .peers
    } else if announce.starts_with("udp") {
        query_udp_tracker(&announce, info_hash, peer_id, port, left_hints)
            .await?
            .peers
    } else {
        anyhow::bail!("Unsupported tracker protocol: {announce}");
    };
    let Some(peer) = peers.first() else {
        anyhow::bail!("Tracker returned no peers");
    };

    let mut stream = connect_to_peer(*peer).await?;
    let hs = Handshake::new(info_hash, peer_id);
    Handshake::send_handshake(&mut stream, &hs).await?;
    Handshake::send_interested(&mut stream).await?;
    Handshake::request_piece(&mut stream, 0, 0, 16 * 1024).await?;
    Ok(())
}
