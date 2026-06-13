# Base58 Check

**A Rust library for Base58 and Base58Check encoding/decoding** — the Bitcoin address format that produces compact, human-readable, typo-resistant strings with built-in error detection.

## Why It Matters

Base58 was invented by Satoshi Nakamoto for Bitcoin addresses to solve a specific UX problem: Base64 and hex contain characters that look similar (0/O, I/l, +, /) and are hard to read aloud or transcribe. Base58 removes these ambiguous characters, using a 58-symbol alphabet from the Bitcoin character set.

Base58Check extends Base58 with a 4-byte checksum (double SHA-256 of the version + payload), so transcription errors are detected at decode time with ~99.9996% probability. This is why Bitcoin addresses start with `1` (mainnet P2PKH), `3` (P2SH), `bc1` (Bech32 SegWit), or `5`/`K`/`L` (WIF private keys).

The encoding converts raw bytes from base 256 to base 58 using big-integer arithmetic. Leading zero bytes map to leading `1` characters, preserving the "zero prefix" identity.

## How It Works

**Base58 Encoding**: The input byte array is treated as a big-endian big integer. Leading 0x00 bytes are counted (each becomes a leading `1` in the output). The remaining bytes are repeatedly divided by 58, collecting remainders that index into the alphabet `123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz` (note: no 0, O, I, l).

**Base58Check**: Prepend a version byte (e.g., `0x00` for mainnet P2PKH), compute `SHA256(SHA256(version || payload))`, append the first 4 bytes as a checksum, then Base58-encode the whole thing.

**Decoding reverses the process**: convert base-58 digits back to base-256 bytes, verify the checksum, then extract the version and payload.

## Quick Start

```rust
use base58_check::{base58check_encode, base58check_decode, base58_encode, base58_decode, version};

// Encode a Bitcoin P2PKH address (version 0x00 + 20-byte pubkey hash)
let pubkey_hash = [0x89u8; 20];
let address = base58check_encode(version::P2PKH_MAINNET, &pubkey_hash);
println!("Address: {}", address);

// Round-trip decode
let (version_byte, payload) = base58check_decode(&address).unwrap();
assert_eq!(version_byte, version::P2PKH_MAINNET);
assert_eq!(payload, pubkey_hash.to_vec());

// Pure Base58 (no checksum)
let encoded = base58_encode(b"Hello, Base58!");
let decoded = base58_decode(&encoded).unwrap();
assert_eq!(decoded, b"Hello, Base58!");
```

## API

- **`base58_encode(bytes)` → `String`** — Raw Base58 encoding
- **`base58_decode(str)` → `Result<Vec<u8>, &str>`** — Raw Base58 decoding
- **`base58check_encode(version, payload)` → `String`** — Base58Check with version byte + checksum
- **`base58check_decode(str)` → `Result<(u8, Vec<u8>), &str>`** — Decode + checksum verification
- **`version` module** — Constants: `P2PKH_MAINNET` (0x00), `P2SH_MAINNET` (0x05), `WIF_MAINNET` (0x80), testnet variants

## Architecture Notes

Provides cryptocurrency address encoding for SuperInstance blockchain tooling. Implements the full Bitcoin address format family (P2PKH, P2SH, WIF) with network-specific version bytes. See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
