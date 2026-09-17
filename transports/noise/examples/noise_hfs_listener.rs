//! Interop listener (responder) for `Noise_XXhfs_25519+MLKEM768_ChaChaPoly_SHA256`.
//!
//! Accepts one connection, completes the handshake, sends one greeting, checks
//! the reply, prints `INTEROP_OK` and exits 0. Stdout contract: `LOCAL`,
//! `READY`, `PEER`, `SENT`, `RECV`, `INTEROP_OK`; errors go to stderr, exit 1.
//!
//! ```bash
//! cargo run -p libp2p-noise --example noise_hfs_listener --features mlkem-hfs -- 9999
//! ```

#[path = "common/interop.rs"]
mod interop;

use futures::{executor::block_on, io::AllowStdIo};
use libp2p_core::upgrade::InboundConnectionUpgrade;
use libp2p_identity as identity;
use libp2p_noise as noise;
use std::net::TcpListener;

fn main() {
    let port = interop::parse_port(9999);
    let id_keys = identity::Keypair::generate_ed25519();
    println!("LOCAL {}", id_keys.public().to_peer_id());
    let config =
        noise::Config::new(&id_keys).unwrap_or_else(|e| interop::fail(format!("config init: {e}")));

    let listener = TcpListener::bind(("127.0.0.1", port))
        .unwrap_or_else(|e| interop::fail(format!("bind 127.0.0.1:{port}: {e}")));
    println!("READY {port}");
    let (stream, peer_addr) = listener
        .accept()
        .unwrap_or_else(|e| interop::fail(format!("accept: {e}")));
    eprintln!("connection from {peer_addr}");

    block_on(async {
        let (peer_id, mut io) = config
            .upgrade_inbound(AllowStdIo::new(stream), interop::HFS_PROTOCOL)
            .await
            .unwrap_or_else(|e| interop::fail(format!("handshake failed: {e}")));
        println!("PEER {peer_id}");
        interop::send_greeting(&mut io)
            .await
            .unwrap_or_else(|e| interop::fail(e));
        interop::read_greeting(&mut io)
            .await
            .unwrap_or_else(|e| interop::fail(e));
        println!("INTEROP_OK");
    });
}
