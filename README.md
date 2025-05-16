<p align="center">
  <img src=".github\Image\Crab-torrent.jpg" alt="Crab-Torrent" width="150"/>
</p>

  # RusTor

  RusTor is a Rust-based BitTorrent client designed to handle torrent files and magnet links, supporting
  Bencode encoding/decoding, peer communication, and tracker interactions.
  This project aims to provide a lightweight and efficient implementation of the BitTorrent protocol.

  ## Features

  -[x] **Bencode Parsing**: Encode and decode Bencode data for torrent files.
  -[x] **Torrent File Support**: Parse and process `.torrent` files and magnet links.
  -[x] **Peer Communication**: Connect and interact with peers for data exchange.
  -[x] **Tracker Interaction**: Communicate with HTTP and UDP trackers to retrieve peer information.


  ## Getting Started

  ### Prerequisites

  - Rust (latest stable version)
  - Cargo (included with Rust)

  Install Rust using rustup:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
  ```


  ## Usage

  ```bash
  git clone https://github.com/bhaskar10h/RusTor.git
  cd RusTor
  cargo build
  cargo run  
  ``

  ## TODO
  - [ ] Seeding in OG .torrent protocol
  - [x] Magnet links - might go w/ udp trackers
      - [ ] UDP Trackers ([BEP0015][])
          - can acquire a list of peers (ip & port) from a UDP tracker url
      - [ ] UDP Extensions ([BEP0041][])
      - [x] Extension to download metadata from peers ([BEP0009][])
  - [x] Announce list - i.e. Multitracker Metadata Extension ([BEP0012])
  - [ ] Some pretty terminal visual of pieces being downloaded?
                      
<!-- reference links -->
[jl-blog-post]: https://blog.jse.li/posts/torrent/
[ubuntu-torrent-url]: https://ubuntu.com/download/alternative-downloads
[BEP0003]: http://bittorrent.org/beps/bep_0003.html 'original bittorrent spec'
[BEP0015]: http://bittorrent.org/beps/bep_0015.html 'UDP Trackers'
[BEP0009]:  http://bittorrent.org/beps/bep_0009.html 'Extension for Peers to Send Metadata Files'
[BEP0041]: http://bittorrent.org/beps/bep_0041.html 'UDP Extensions'
[BEP0012]: http://bittorrent.org/beps/bep_0012.html 'Multitracker Metadata Extension'
[nasa-torrents]: https://academictorrents.com/collection/nasa-datasets 'Archives of NASA torrents'
[example-nasa-torrent]: https://academictorrents.com/details/059ed25558b4587143db637ac3ca94bebb57d88d
[BEP0023]: http://bittorrent.org/beps/bep_0023.html 'Compact Peer Lists'
[BEP0006]: http://bittorrent.org/beps/bep_0006.html 'Fast Extension'
[BEP0029]: http://bittorrent.org/beps/bep_0029.html 'uTorrent Transport Protocol'
