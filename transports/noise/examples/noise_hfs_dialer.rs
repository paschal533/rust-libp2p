//! Interop dialer (initiator) for `Noise_XXhfs_25519+MLKEM768_ChaChaPoly_SHA256`.
//!
//! Reads the listener's greeting, replies, prints `INTEROP_OK`, then waits
//! briefly for the peer to close so the reply is not discarded by an abortive
//! close. Stdout contract as in `noise_hfs_listener`.
//!
//! ```bash
//! cargo run -p libp2p-noise --example noise_hfs_dialer --features mlkem-hfs -- 9999
//! ```

#[path = "common/interop.rs"]
mod interop;

use futures::{executor::block_on, io::AllowStdIo, prelude::*};
use libp2p_core::upgrade::OutboundConnectionUpgrade;
use libp2p_identity as identity;
use libp2p_noise as noise;
use std::{net::TcpStream, time::Duration};

const PEER_CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

fn main() {
    let port = interop::parse_port(9999);
    let id_keys = identity::Keypair::generate_ed25519();
    println!("LOCAL {}", id_keys.public().to_peer_id());
    let config =
        noise::Config::new(&id_keys).unwrap_or_else(|e| interop::fail(format!("config init: {e}")));

    let stream = TcpStream::connect(("127.0.0.1", port))
        .unwrap_or_else(|e| interop::fail(format!("connect 127.0.0.1:{port}: {e}")));
    // A second handle to the same socket, used only to bound the final wait for
    // the peer to close. Setting the timeout up front would also bound the
    // handshake, which is slow against pure-Python ML-KEM.
    let close_probe = stream
        .try_clone()
        .unwrap_or_else(|e| interop::fail(format!("clone socket: {e}")));

    block_on(async {
        let (peer_id, mut io) = config
            .upgrade_outbound(AllowStdIo::new(stream), interop::HFS_PROTOCOL)
            .await
            .unwrap_or_else(|e| interop::fail(format!("handshake failed: {e}")));
        println!("PEER {peer_id}");
        interop::read_greeting(&mut io)
            .await
            .unwrap_or_else(|e| interop::fail(e));
        interop::send_greeting(&mut io)
            .await
            .unwrap_or_else(|e| interop::fail(e));
        println!("INTEROP_OK");
        // Wait for the peer to close so the reply is not lost to an abortive
        // close. EOF, a timeout or a reset are all acceptable outcomes here.
        close_probe
            .set_read_timeout(Some(PEER_CLOSE_TIMEOUT))
            .unwrap_or_else(|e| interop::fail(format!("set_read_timeout: {e}")));
        let _ = io.read(&mut [0u8; 1]).await;
    });
}
