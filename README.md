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
  ```

   ## ✅ TODO

  - [ ] Seeding support for original `.torrent` protocol
  - [x] Magnet links
    - [ ] UDP Trackers  – acquire peers from a UDP tracker
      - [ ] UDP Extensions
      - [x] Metadata download from peers 
  - [x] Announce list / Multitracker support 
  - [ ] Visual terminal progress for downloaded pieces?
      - Consider TUI/terminal graphics for this


  ## Reference

  <!-- Reference Links -->
  [BEP0003]: https://wiki.theory.org/BitTorrentSpecification#Related_Documents "Bittorrent Specifications"
  [BEP0015]: https://bittorrent.org/bittorrentecon.pdf "Incentives Build Robustness in BitTorrent"
  [BEP0041]: https://bittorrent.org/beps/bep_0003.html "BitTorent.org"
  [BEP0009]: http://bittorrent.org/beps/bep_0009.html "Extension for Peers to Send Metadata Files"
  [BEP0012]: http://bittorrent.org/beps/bep_0012.html "Multitracker Metadata Extension"
  
