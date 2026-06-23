# Cross-language interop + structural tests + benchmarks added

We've been working on cross-language PQC interop across the libp2p ecosystem (JS: js-libp2p-noise PR #665, Python: py-libp2p PR #1310) and wanted to contribute the Rust side of that work here.

## 1. Structural compliance tests (`tests/interop_hfs.rs`)

`SeededResolver` pins the X25519 static keys for both sides, but ML-KEM ephemeral keys still use system entropy — the snow fork's `generate()` bypasses the seeded RNG and calls the OS directly. Because of this, the handshake hash changes every run and a pinned `HANDSHAKE_HASH` constant cannot be asserted across runs or machines.

What the tests *can* assert deterministically:

- **Intra-run hash agreement**: both initiator and responder compute the same handshake hash within a single run.
- **Message length geometry**: msg1 = 1216 bytes (e + e1 KEM pubkey), msg2 = 1200 bytes (e + ee + s + es + ekem1 ciphertext + AEAD tags), msg3 = 64 bytes (s + se).

There is also an `#[ignore]` `print_vectors` helper that prints hex-encoded messages for manual cross-language comparison.

## 2. Interop listener binary (`examples/noise_hfs_listener.rs`)

Standalone TCP listener binary for cross-language testing. Prints `READY <port>` before accepting a connection and `PEER <peer_id>` after the handshake completes. Intended to be paired with the Python or JS dialers mentioned above.

```
cargo run --example noise_hfs_listener --features mlkem-hfs -- 9999
```

## 3. Criterion benchmarks (`benches/noise_hfs.rs`)

Compares classical Noise XX vs XXhfs handshake latency, plus 1 KB transport throughput after the hybrid handshake. Useful for production planning documentation.

---

## Finding: snow's ML-KEM entropy bypasses the seeded resolver

The current snow fork's `generate()` in `DefaultResolver` calls system entropy directly, bypassing any seeded `CryptoResolver`. This means cross-implementation deterministic test vectors — needed to close the KDF mixing order question in specs#723 — cannot be produced through `snow::Builder` alone without a custom ML-KEM implementation that accepts an external RNG. Worth considering whether a seeded generation path should be added to the snow fork for interop testing purposes.

## Suggestion: RustCrypto `ml-kem` swap

`Resolver::resolve_kem()` delegates to snow's `DefaultResolver`, whose ML-KEM backend is marked unaudited in the source. The RustCrypto `ml-kem` crate (FIPS 203 compliant, audited by NCC Group) could be used instead by implementing `snow::types::Kem` directly — the same pattern as how `Keypair` already wraps `x25519-dalek` for the DH path. Happy to write that as a follow-up if you're open to it.

---

Thanks for shepherding the XXhfs spec forward!
