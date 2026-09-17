// Pins which handshake each multistream id selects, observed on the wire.
//
// `Config` falls back to classical XX for any protocol id it does not
// recognise, on both peers, so a handshake between two peers using a stale
// id still succeeds. The only observable difference is the size of message A:
// 32 bytes (e) for XX, 32 + 1184 bytes (e, e1) for XXhfs.
#![cfg(feature = "mlkem-hfs")]

use futures::{future, prelude::*};
use libp2p_core::upgrade::OutboundConnectionUpgrade;
use libp2p_identity as identity;
use libp2p_noise as noise;

const HFS: &str = "/noise-mlkem768-hfs/0.2.0";
const STALE_HFS: &str = "/noise-mlkem768-hfs/0.1.0";

/// e (32) + e1, the ML-KEM-768 encapsulation key (1184).
const HFS_MSG_A_LEN: u16 = 1216;
/// e (32).
const XX_MSG_A_LEN: u16 = 32;

/// Starts an outbound upgrade with `protocol` and returns the length prefix of
/// the first frame it writes. The handshake itself is abandoned.
fn first_frame_len(protocol: &'static str) -> u16 {
    let id = identity::Keypair::generate_ed25519();
    let (client, mut server) = futures_ringbuf::Endpoint::pair(4096, 4096);

    futures::executor::block_on(async move {
        let upgrade = noise::Config::new(&id)
            .unwrap()
            .upgrade_outbound(client, protocol);
        let read_len = async move {
            let mut len = [0u8; 2];
            server
                .read_exact(&mut len)
                .await
                .expect("read length prefix");
            u16::from_be_bytes(len)
        };

        match future::select(Box::pin(read_len), upgrade).await {
            future::Either::Left((len, _abandoned_upgrade)) => len,
            future::Either::Right((result, _)) => {
                panic!(
                    "upgrade finished before message A was read: {:?}",
                    result.err()
                )
            }
        }
    })
}

#[test]
fn hybrid_id_sends_hybrid_message_a() {
    assert_eq!(first_frame_len(HFS), HFS_MSG_A_LEN);
}

#[test]
fn classical_id_sends_classical_message_a() {
    assert_eq!(first_frame_len("/noise"), XX_MSG_A_LEN);
}

#[test]
fn stale_hybrid_id_falls_back_to_classical() {
    assert_ne!(first_frame_len(STALE_HFS), HFS_MSG_A_LEN);
    assert_eq!(first_frame_len(STALE_HFS), XX_MSG_A_LEN);
}
