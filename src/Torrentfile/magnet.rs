use anyhow::{Result, anyhow, bail};
use data_encoding::BASE32;
use url::Url;

pub struct MagnetLink {
    pub infohash: [u8; 20],
    pub trackers: Vec<String>,
    pub display_name: Option<String>,
}

pub fn parse_magnet_link(link: &str) -> Result<MagnetLink> {
    let url = Url::parse(link)?;
    if url.scheme() != "magnet" {
        return Err(anyhow!("Invalid magnet link scheme"));
    }

    let xt = url
        .query_pairs()
        .find(|(key, _)| key == "xt")
        .ok_or_else(|| anyhow!("Missing xt parameter"))?
        .1
        .to_string();

    let info_hash_str = if let Some(s) = xt.strip_prefix("urn:btih:") {
        if s.len() == 40 {
            hex::decode(s.to_ascii_lowercase())?
        } else if s.len() == 32 {
            BASE32
                .decode(s.to_ascii_uppercase().as_bytes())
                .map_err(|_| anyhow!("Invalid base32 btih"))?
        } else {
            bail!("btih must be 40 hex or 32 base32 chars");
        }
    } else {
        bail!("xt must start with urn:btih");
    };

    if info_hash_str.len() != 20 {
        bail!("btih must decode to 20 bytes");
    }

    let mut infohash = [0u8; 20];
    infohash.copy_from_slice(&info_hash_str);

    let trackers = url
        .query_pairs()
        .filter(|(k, _)| k == "tr")
        .map(|(_, v)| v.to_string())
        .collect::<Vec<_>>();

    let display_name = url
        .query_pairs()
        .find(|(k, _)| k == "dn")
        .map(|(_, v)| v.to_string());

    Ok(MagnetLink {
        infohash,
        trackers,
        display_name,
    })
}
