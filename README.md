# base58-check

**Base58Check encoding and decoding for Bitcoin-style addresses, WIF keys, and versioned payloads.**

Base58 is a binary-to-text encoding scheme designed for human-readable representation of byte sequences. Unlike Base64, it omits visually ambiguous characters (0, O, I, l) and symbols (+, /) that are problematic in URLs and printed text. Base58Check extends Base58 with a 4-byte double-SHA-256 checksum for error detection, making it suitable for cryptocurrency addresses where a single transcription error should be caught, not silently processed.

## Why It Matters

Every Bitcoin address, WIF private key, and P2SH script hash is encoded in Base58Check. The encoding solves three practical problems:

1. **Human readability** — `1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa` is easier to transcribe than `0x0009668b8a51b5c8e1c3e9a3d...`
2. **Error detection** — The 4-byte checksum catches ~99.9999% of single-character errors and most multi-character errors. The probability of a random string having a valid checksum is 1/2³² ≈ 2.3 × 10⁻¹⁰.
3. **Version tagging** — The leading version byte identifies the address type (P2PKH, P2SH, WIF, testnet), enabling wallets to prevent cross-network errors.

Beyond Bitcoin, Base58Check is used in IPFS peer IDs, Ripple addresses, and various altcoin systems.

## How It Works

### Base58 Encoding Algorithm

Input: byte array `b[0..m]`. Output: string.

**Step 1 — Count leading zero bytes.** Each leading 0x00 byte maps to a leading '1' character in the output.

**Step 2 — Base conversion (base 256 → base 58).** Treat the input as a big-endian integer. Repeatedly divide by 58, collecting remainders as Base58 digits:

```
digits = []
for byte in b:
    carry = byte
    for d in digits:
        val = d * 256 + carry
        d = val % 58
        carry = val / 58
    while carry > 0:
        digits.push(carry % 58)
        carry /= 58
```

**Step 3 — Map to alphabet.** Reverse digits and map each to `BASE58_ALPHABET[digit]`:

> Alphabet: `123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz`

Note: `0` (zero), `O` (capital o), `I` (capital i), and `l` (lowercase L) are excluded.

**Step 4 — Prepend '1's** for each leading zero byte.

### Base58Check

```
payload = version_byte || data
checksum = SHA256(SHA256(payload))[0:4]
encoded = Base58Encode(payload || checksum)
```

The double SHA-256 provides a **collision-resistant** checksum. Finding two inputs with the same 4-byte checksum requires ~2¹⁶ SHA-256 evaluations on average (birthday bound), but finding one that matches a *specific* checksum requires ~2³² evaluations.

### Size Analysis

| Payload | Base58Check length | Hex length |
|---------|-------------------|------------|
| 20 bytes (pubkey hash) | ~28 chars | 40 chars |
| 32 bytes (priv key) | ~38 chars | 64 chars |
| 25 bytes (version + hash + checksum) | 34 chars (typical P2PKH) | 50 chars |

Base58Check is ~40% shorter than hex for the same data.

### Version Bytes

| Version | Prefix | Network | Address Type |
|---------|--------|---------|-------------|
| 0x00 | `1` | Mainnet | P2PKH |
| 0x6F | `m`/`n` | Testnet | P2PKH |
| 0x05 | `3` | Mainnet | P2SH |
| 0xC4 | `2` | Testnet | P2SH |
| 0x80 | `5`/`K`/`L` | Mainnet | WIF (private key) |
| 0xEF | `9`/`c` | Testnet | WIF |

### Complexity

| Operation | Time | Notes |
|-----------|------|-------|
| `base58_encode(n bytes)` | O(n²) | Schoolbook big-int division |
| `base58_decode(n chars)` | O(n²) | Schoolbook big-int multiplication |
| `base58check_encode` | O(n²) + SHA-256 | SHA-256 is O(n) |
| `base58check_decode` | O(n²) + SHA-256 | Same |

The O(n²) comes from the base-256↔base-58 conversion using schoolbook arithmetic. For the typical 25-byte payload, this is negligible (~625 operations).

## Quick Start

```rust
// Note: source is in src/main.rs
use base58_check::{base58check_encode, base58check_decode, version};

// Encode a P2PKH address (mainnet)
let pubkey_hash = [0x89u8; 20];
let addr = base58check_encode(version::P2PKH_MAINNET, &pubkey_hash);
println!("Address: {}", addr);

// Round-trip decode
let (ver, data) = base58check_decode(&addr).unwrap();
assert_eq!(ver, version::P2PKH_MAINNET);
assert_eq!(data, pubkey_hash.to_vec());

// Encode WIF private key
let privkey = [0x42u8; 32];
let wif = base58check_encode(version::WIF_MAINNET, &privkey);

// Pure Base58 (no checksum)
let original = b"Hello, Base58!";
let encoded = base58_check::base58_encode(original);
let decoded = base58_check::base58_decode(&encoded).unwrap();
assert_eq!(original.to_vec(), decoded);
```

## API

- **`base58_encode(input: &[u8]) → String`** — Raw Base58 (no checksum)
- **`base58_decode(input: &str) → Result<Vec<u8>, &str>`** — Raw Base58 decode
- **`base58check_encode(version: u8, payload: &[u8]) → String`** — Versioned + checksummed
- **`base58check_decode(input: &str) → Result<(u8, Vec<u8>), &str>`** — Verify checksum, return (version, payload)
- **`version` module** — P2PKH_MAINNET (0x00), P2PKH_TESTNET (0x6F), P2SH_MAINNET (0x05), P2SH_TESTNET (0xC4), WIF_MAINNET (0x80), WIF_TESTNET (0xEF)

## Architecture Notes

The γ+η=C identity: γ (generative capacity) is the range of version types the encoding supports — each version byte unlocks a new use case (addresses, keys, scripts). η (evaluative depth) is the checksum verification that catches transcription errors. C = usability: an encoding system that supports diverse payload types (high γ) while catching human errors (high η) achieves high C (production reliability).

## References

1. Nakamoto, S. (2008). *Bitcoin: A Peer-to-Peer Electronic Cash System*. — Original use of Base58Check.
2. Bitcoin Wiki (2019). "Base58Check encoding." — Algorithm specification.
3. Antonopoulos, A. (2014). *Mastering Bitcoin*, Ch. 4. — Address generation pipeline.
4. Eastlake, D. & Hansen, T. (2006). RFC 4634: "US Secure Hash Algorithms (SHA and HMAC-SHA)." — SHA-256 specification.

## License

MIT
