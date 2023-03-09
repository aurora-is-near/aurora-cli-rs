use aurora_engine_transactions::legacy::{LegacyEthSignedTransaction, TransactionLegacy};
use aurora_engine_types::{types::Address, U256};
use libsecp256k1::{Message, PublicKey, SecretKey};
use near_crypto::{InMemorySigner, KeyFile};
use rlp::RlpStream;
use std::{io, path::Path};

pub fn hex_to_arr<const N: usize>(h: &str) -> Result<[u8; N], hex::FromHexError> {
    let mut output = [0u8; N];
    hex::decode_to_slice(h.strip_prefix("0x").unwrap_or(h), &mut output)?;
    Ok(output)
}

pub fn hex_to_address(h: &str) -> Result<Address, hex::FromHexError> {
    hex_to_arr(h).map(Address::from_array)
}

pub fn hex_to_vec(h: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(h.strip_prefix("0x").unwrap_or(h))
}

pub fn address_from_secret_key(sk: &SecretKey) -> Address {
    let pk = PublicKey::from_secret_key(sk);
    let hash = aurora_engine_sdk::keccak(&pk.serialize()[1..]);
    Address::try_from_slice(&hash[12..]).unwrap()
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

pub fn read_key_file<P: AsRef<Path>>(path: P) -> io::Result<InMemorySigner> {
    let content = std::fs::read_to_string(path)?;
    let key: KeyFile = serde_json::from_str(&content)?;
    Ok(InMemorySigner {
        account_id: key.account_id,
        public_key: key.public_key,
        secret_key: key.secret_key,
    })
}
