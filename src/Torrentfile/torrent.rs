use crate::Bencode::decode::decode_torrent_file;
use crate::Bencode::encode::encode_bencode;
use crate::bittorent::Torrent;
use sha1::{Digest, Sha1};

pub struct TorrentFile {
    pub torrent: Torrent,
    pub info_hash: [u8; 20],
}

impl TorrentFile {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read(path)?;
        eprintln!("Raw torrent file contents: {:?}", content);
        let torrent = decode_torrent_file(path)?;

        // Validate pieces length
        if torrent.info.pieces.len() % 20 != 0 {
            return Err("Invalid torrent: pieces length not divisible by 20".into());
        }

        let info_bencoded = encode_bencode(&torrent.info)?;
        let mut hasher = Sha1::new();
        hasher.update(&info_bencoded);
        let info_hash: [u8; 20] = hasher.finalize().into();

        Ok(TorrentFile { torrent, info_hash })
    }
}
