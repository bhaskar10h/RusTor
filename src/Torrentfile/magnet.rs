use url::Url;

pub struct MagnetLink {
    pub infohash: [u8; 20],
    pub trackers: Vec<String>,
}

pub fn parse_magnet_link(link: &str) -> Result<MagnetLink, Box<dyn std::error::Error>> {
    let url = Url::parse(link)?;
    if url.scheme() != "magnet" {
        return Err("Invalid magnet link scheme".into());
    }

    let xt = url
        .query_pairs()
        .find(|(key, _)| key == "xt")
        .ok_or("Missing xt parameter")?
        .1;

    let info_hash_str = xt.strip_prefix("urn:btih:").ok_or("Invalid xt format")?;

    let info_hash = hex::decode(info_hash_str)?;
    let infohash: [u8; 20] = info_hash.try_into().map_err(|_| "Invalid hash length")?;

    let trackers = url
        .query_pairs()
        .filter(|(key, _)| key == "tr")
        .map(|(_, value)| value.to_string())
        .collect();

    Ok(MagnetLink { infohash, trackers })
}
