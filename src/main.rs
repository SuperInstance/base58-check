use std::fmt;

/// Base58Check encoding/decoding (Bitcoin alphabet).
const BASE58_ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Base58 alphabet lookup table (255 = invalid).
fn build_decode_table() -> [u8; 256] {
    let mut table = [255u8; 256];
    for (i, &ch) in BASE58_ALPHABET.iter().enumerate() {
        table[ch as usize] = i as u8;
    }
    table
}

/// Encode raw bytes into Base58 (no checksum).
pub fn base58_encode(input: &[u8]) -> String {
    // Count leading zeros
    let leading_zeros = input.iter().take_while(|&&b| b == 0).count();

    // Convert to big integer (base 256 → base 58)
    let mut digits: Vec<u8> = Vec::new();
    for &byte in input {
        let mut carry = byte as u32;
        for digit in digits.iter_mut() {
            let val = (*digit as u32) * 256 + carry;
            *digit = (val % 58) as u8;
            carry = val / 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }

    // Add leading '1's for leading zero bytes
    let mut result = String::new();
    for _ in 0..leading_zeros {
        result.push('1');
    }

    // Reverse and map to alphabet
    for &digit in digits.iter().rev() {
        result.push(BASE58_ALPHABET[digit as usize] as char);
    }

    result
}

/// Decode a Base58 string into raw bytes (no checksum validation).
pub fn base58_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let table = build_decode_table();

    // Count leading '1's
    let leading_ones = input.chars().take_while(|&c| c == '1').count();

    // Convert from base 58 to base 256
    let mut bytes: Vec<u8> = Vec::new();
    for ch in input.chars() {
        let val = table[ch as usize];
        if val == 255 {
            return Err("Invalid Base58 character");
        }
        let mut carry = val as u32;
        for byte in bytes.iter_mut() {
            let val = (*byte as u32) * 58 + carry;
            *byte = (val & 0xFF) as u8;
            carry = val >> 8;
        }
        while carry > 0 {
            bytes.push((carry & 0xFF) as u8);
            carry >>= 8;
        }
    }

    // Add leading zero bytes
    let mut result = vec![0u8; leading_ones];
    result.extend(bytes.iter().rev());
    Ok(result)
}

/// Simple SHA-256 (simplified stub for demonstration).
/// In production, use a real SHA-256 implementation.
fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];
    for (i, &byte) in data.iter().enumerate() {
        hash[i % 32] ^= byte;
        hash[(i + 1) % 32] = hash[(i + 1) % 32].wrapping_add(byte);
    }
    // Mix rounds
    for round in 0..4 {
        for i in 0..32 {
            let j = (i + round + 1) % 32;
            hash[i] = hash[i].wrapping_add(hash[j]).wrapping_mul(31);
        }
    }
    hash
}

/// Encode bytes with a version byte and 4-byte checksum (Base58Check).
pub fn base58check_encode(version: u8, payload: &[u8]) -> String {
    let mut data = vec![version];
    data.extend_from_slice(payload);

    // Compute double SHA-256 checksum
    let first_hash = sha256(&data);
    let second_hash = sha256(&first_hash);
    let checksum = &second_hash[..4];

    data.extend_from_slice(checksum);
    base58_encode(&data)
}

/// Decode a Base58Check string. Returns (version, payload) or error.
pub fn base58check_decode(input: &str) -> Result<(u8, Vec<u8>), &'static str> {
    let decoded = base58_decode(input)?;
    if decoded.len() < 6 {
        return Err("Base58Check data too short");
    }
    let payload_len = decoded.len() - 4;
    let payload = &decoded[..payload_len];
    let checksum = &decoded[payload_len..];

    // Verify checksum
    let first_hash = sha256(payload);
    let second_hash = sha256(&first_hash);
    if &second_hash[..4] != checksum {
        return Err("Checksum mismatch");
    }

    let version = payload[0];
    let data = payload[1..].to_vec();
    Ok((version, data))
}

/// Address version bytes for common networks.
pub mod version {
    pub const P2PKH_MAINNET: u8 = 0x00;
    pub const P2PKH_TESTNET: u8 = 0x6F;
    pub const P2SH_MAINNET: u8 = 0x05;
    pub const P2SH_TESTNET: u8 = 0xC4;
    pub const WIF_MAINNET: u8 = 0x80;
    pub const WIF_TESTNET: u8 = 0xEF;
}



fn main() {
    // Encode a P2PKH address (version 0x00 + 20-byte pubkey hash)
    let pubkey_hash: Vec<u8> = vec![0x89; 20];
    let addr = base58check_encode(version::P2PKH_MAINNET, &pubkey_hash);
    println!("P2PKH address: {}...", &addr[..12]);

    // Encode a P2SH address
    let script_hash: Vec<u8> = vec![0xAB; 20];
    let p2sh_addr = base58check_encode(version::P2SH_MAINNET, &script_hash);
    println!("P2SH address: {}...", &p2sh_addr[..12]);

    // Encode WIF private key
    let privkey: Vec<u8> = vec![0x42; 32];
    let wif = base58check_encode(version::WIF_MAINNET, &privkey);
    println!("WIF key: {}...", &wif[..12]);

    // Round-trip test
    let decoded = base58check_decode(&addr).expect("decode failed");
    println!("Decoded version: 0x{:02X}", decoded.0);
    assert_eq!(decoded.0, version::P2PKH_MAINNET);
    assert_eq!(decoded.1, pubkey_hash);

    // Pure base58 round-trip
    let original = b"Hello, Base58!";
    let encoded = base58_encode(original);
    let decoded58 = base58_decode(&encoded).unwrap();
    assert_eq!(original.to_vec(), decoded58);
    println!("Base58 round-trip OK: {}...", &encoded[..16]);

    // Testnet address
    let test_addr = base58check_encode(version::P2PKH_TESTNET, &pubkey_hash);
    println!("Testnet P2PKH: {}...", &test_addr[..12]);
}
