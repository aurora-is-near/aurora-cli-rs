use aurora_engine_transactions::legacy::{LegacyEthSignedTransaction, TransactionLegacy};
use aurora_engine_types::{types::Address, U256};
use libsecp256k1::{Message, PublicKey, SecretKey};
use near_crypto::InMemorySigner;
use rlp::RlpStream;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(not(feature = "advanced"))]
pub fn secret_key_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<SecretKey> {
    std::fs::read_to_string(path)
        .map_err(Into::into)
        .and_then(|key| secret_key_from_hex(&key))
}

#[cfg(feature = "advanced")]
pub fn hex_to_address(h: &str) -> anyhow::Result<Address> {
    hex_to_arr(h).map(Address::from_array)
}

#[cfg(feature = "advanced")]
pub fn hex_to_vec(h: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(h.strip_prefix("0x").unwrap_or(h))
}

pub fn address_from_secret_key(sk: &SecretKey) -> anyhow::Result<Address> {
    let pk = PublicKey::from_secret_key(sk);
    let hash = aurora_engine_sdk::keccak(&pk.serialize()[1..]);
    Address::try_from_slice(&hash[12..])
        .map_err(|e| anyhow::anyhow!("Couldn't create address from secret key: {e}"))
}

pub fn secret_key_from_hex(key: &str) -> anyhow::Result<SecretKey> {
    hex_to_arr(key.trim())
        .and_then(|bytes| SecretKey::parse(&bytes).map_err(Into::into))
        .map_err(|e| anyhow::anyhow!("Couldn't create secret key from hex: {e}"))
}

#[cfg(not(feature = "advanced"))]
pub fn address_from_hex(address: &str) -> anyhow::Result<Address> {
    Address::decode(address.trim_start_matches("0x"))
        .map_err(|e| anyhow::anyhow!("Error decoding address: {e}"))
}

pub fn sign_transaction(
    tx: TransactionLegacy,
    chain_id: u64,
    secret_key: &SecretKey,
) -> LegacyEthSignedTransaction {
    let mut rlp_stream = RlpStream::new();
    tx.rlp_append_unsigned(&mut rlp_stream, Some(chain_id));
    let message_hash = aurora_engine_sdk::keccak(rlp_stream.as_raw());
    let message = Message::parse_slice(message_hash.as_bytes()).unwrap();

    let (signature, recovery_id) = libsecp256k1::sign(&message, secret_key);
    let v: u64 = (u64::from(recovery_id.serialize())) + 2 * chain_id + 35;
    let r = U256::from_big_endian(&signature.r.b32());
    let s = U256::from_big_endian(&signature.s.b32());
    LegacyEthSignedTransaction {
        transaction: tx,
        v,
        r,
        s,
    }
}

pub fn read_key_file<P: AsRef<Path>>(path: P) -> anyhow::Result<InMemorySigner> {
    let content = std::fs::read_to_string(path)?;
    let key: KeyFile = serde_json::from_str(&content)?;

    match key {
        KeyFile::WithPublicKey(key) => Ok(InMemorySigner {
            account_id: key.account_id,
            public_key: key.public_key,
            secret_key: key.secret_key,
        }),
        KeyFile::WithoutPublicKey(key) => Ok(InMemorySigner {
            account_id: key.account_id,
            public_key: key.secret_key.public_key(),
            secret_key: key.secret_key,
        }),
    }
}

pub fn hex_to_arr<const SIZE: usize>(hex: &str) -> anyhow::Result<[u8; SIZE]> {
    let mut output = [0u8; SIZE];

    hex::decode_to_slice(hex, &mut output)
        .map(|_| output)
        .map_err(Into::into)
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum KeyFile {
    WithPublicKey(KeyFileWithPublicKey),
    WithoutPublicKey(KeyFileWithoutPublicKey),
}

/// This is copied from the nearcore repo
/// `https://github.com/near/nearcore/blob/5252ba65ce81e187a3ba76dc3db754a596bc16d1/core/crypto/src/key_file.rs#L12`
/// for the purpose of having the `private_key` serde alias because that change has not yet
/// been released (as of v0.14.0). We should delete this and use near's type once the new
/// version is released.
#[derive(Serialize, Deserialize)]
struct KeyFileWithPublicKey {
    pub account_id: near_primitives::types::AccountId,
    pub public_key: near_crypto::PublicKey,
    // Credential files generated which near cli works with have private_key
    // rather than secret_key field. To make it possible to read those from
    // neard add private_key as an alias to this field so either will work.
    #[serde(alias = "private_key")]
    pub secret_key: near_crypto::SecretKey,
}

#[derive(Serialize, Deserialize)]
struct KeyFileWithoutPublicKey {
    pub account_id: near_primitives::types::AccountId,
    // Credential files generated which near cli works with have private_key
    // rather than secret_key field. To make it possible to read those from
    // neard add private_key as an alias to this field so either will work.
    #[serde(alias = "private_key")]
    pub secret_key: near_crypto::SecretKey,
}

/// Converts NEAR into yocto. 1NEAR == 10^24 yocto.
#[cfg(not(feature = "advanced"))]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn near_to_yocto(near: f64) -> u128 {
    (near * 1_000_000.0) as u128 * 1_000_000_000_000_000_000
}

#[test]
#[cfg(not(feature = "advanced"))]
fn test_address_from_hex() {
    assert!(address_from_hex("0x1C16948F011686AE74BB2Ba0477aeFA2Ea97084D").is_ok());
    assert!(address_from_hex("1C16948F011686AE74BB2Ba0477aeFA2Ea97084D").is_ok());
    assert!(address_from_hex("some_address").is_err());
}

#[test]
fn test_parsing_key_file() {
    let file = std::env::temp_dir().join("key_file.json");
    let json = r#"{
      "account_id": "user.testnet",
      "public_key": "ed25519:7zLQYrrHBEcfhUGEVtMAgqZXSASgUvxWmXbJwX8rVRYu",
      "secret_key": "ed25519:4DUZ4Wcq5ihwWLLezuxrUgvfLAM3gWeyAnpdLuoNyNaZD8bGkbTmupYYYqQZVkhheoxJ1qcPu52o2JfXwKVG9Xso"
    }"#;
    std::fs::write(&file, json).unwrap();

    let signer: InMemorySigner = read_key_file(&file).unwrap();
    assert_eq!(signer.account_id, "user.testnet".parse().unwrap());

    let json = r#"{
      "account_id": "user.testnet",
      "secret_key": "ed25519:4DUZ4Wcq5ihwWLLezuxrUgvfLAM3gWeyAnpdLuoNyNaZD8bGkbTmupYYYqQZVkhheoxJ1qcPu52o2JfXwKVG9Xso"
    }"#;
    std::fs::write(&file, json).unwrap();

    let signer2: InMemorySigner = read_key_file(&file).unwrap();
    assert_eq!(signer.account_id, "user.testnet".parse().unwrap());
    assert_eq!(signer.public_key, signer2.public_key);
    assert_eq!(signer.secret_key, signer2.secret_key);
}

#[test]
#[cfg(not(feature = "advanced"))]
fn test_convert_near_to_yocto() {
    assert_eq!(near_to_yocto(1.0), 10_u128.pow(24));
    assert_eq!(near_to_yocto(1.125), 1125 * 10_u128.pow(21));
}
