# Cross-language interop + vectors + benchmarks added

We've added three contributions to this branch that address spec and testing blockers:

## 1. Interop listener binary (`examples/noise_hfs_listener.rs`)

Standalone TCP binary that completes one XXhfs handshake and prints the peer ID. Run it alongside the Python (py-libp2p PR #1310) or JavaScript (js-libp2p-noise PR #665) dialers to verify cross-language compatibility.

Usage pattern:
- Start: `cargo run --example noise_hfs_listener --features mlkem-hfs`
- Dial from another language implementation
- Verify handshake completes and hash matches

## 2. Deterministic test vectors (`tests/interop_hfs.rs`)

Uses a seeded `CryptoResolver` to pin ephemeral keys. The resulting `HANDSHAKE_HASH` constant provides evidence for `libp2p/specs#723` that the KDF mixing order is correct — if JS or Python produce the same hash for the same static keys and seed, the mixing order is confirmed across all three implementations.

Also tests structural compliance:
- Correct handshake state machine transitions
- Serialization/deserialization of payloads
- Forward secrecy of ephemeral keys

## 3. Criterion benchmarks (`benches/noise_hfs.rs`)

Classical XX vs XXhfs latency and 1 KB transport throughput. Provides real-world performance data for the hybrid handshake:
- Baseline (classical XX) established
- XXhfs hybrid overhead measured
- Useful for production planning and documentation

---

## Follow-up proposal: RustCrypto `ml-kem` swap

`Resolver::resolve_kem()` currently delegates to `snow::DefaultResolver`, whose ML-KEM implementation is unaudited. The RustCrypto `ml-kem` crate (FIPS 203 compliant, audited by NCC Group) can replace it by implementing `snow::types::Kem` directly for a wrapper type — the same pattern as how `Keypair` already implements `snow::types::Dh` using `x25519-dalek`.

**Benefits:**
- Auditability and FIPS compliance
- Consistent with py-libp2p and js-libp2p-noise (both use RustCrypto-equivalent audited backends)
- Independence from snow's internal resolver for the KEM path
- Future-proofs against snow implementation changes

We're happy to write that swap as a follow-up PR if you're open to it — wanted to surface the proposal here before doing any work.

---

Thanks for shepherding the XXhfs spec forward!
