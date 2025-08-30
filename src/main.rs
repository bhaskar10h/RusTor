use crate::Peers::peer::{Handshake, download_first_piece};
use crate::Torrentfile::magnet::parse_magnet_link;
use crate::Torrentfile::torrent::TorrentFile;
use crate::Tracker::{tracker::query_http_tracker, udp::query_udp_tracker};
use crate::bittorent::connect_to_peer;
use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
// use tokio::io::AsyncReadExt;

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
    peer_id[0..8].copy_from_slice(b"-RS0001-");
    rand::thread_rng().fill(&mut peer_id[8..]);

    let (info_hash, trackers, total_len_opt, piece_length_opt, piece0_hash_opt): (
        [u8; 20],
        Vec<String>,
        Option<u64>,
        Option<u64>,
        Option<[u8; 20]>,
    ) = if target.starts_with("magnet:?") {
        let m = parse_magnet_link(target)?;
        (
            m.infohash,
            if m.trackers.is_empty() {
                vec![]
            } else {
                m.trackers
            },
            None,
            None,
            None,
        )
    } else {
        let p0 = [0u8; 20];
        let tf = TorrentFile::from_file(target)?;
        (
            tf.info_hash,
            vec![tf.torrent.announce.clone()],
            Some(tf.torrent.info.length),
            Some(tf.torrent.info.piece_length),
            Some(p0),
        )
    };

    let announce = trackers
        .get(0)
        .ok_or_else(|| {
            anyhow!("No tracker availabl; magnet without trackers rquires DHT (not implemented)")
        })?
        .clone();

    let port = 6881u16;
    let peers = if announce.starts_with("http") {
        let left = total_len_opt.unwrap_or(0);
        query_http_tracker(&announce, info_hash, peer_id, port, 0, 0, left)
            .await?
            .peers
    } else if announce.starts_with("udp") {
        let left = total_len_opt.unwrap_or(0);
        query_udp_tracker(&announce, info_hash, peer_id, port, left)
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

    if let (Some(total_len), Some(piece_len), Some(piece0_hash)) =
        (total_len_opt, piece_length_opt, piece0_hash_opt)
    {
        let data = download_first_piece(&mut stream, piece_len, total_len, piece0_hash).await?;
        std::fs::write("piece0.bin", &data)?;
        println!("Wrote piece0.bin ({} bytes)", data.len());
    } else {
        println!(
            "Connected to peer via magnet; full downloading needs BEP-9/10 metadata exchange."
        );
    }

    Ok(())
}
