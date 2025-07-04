use std::str::FromStr;

use aurora_engine_transactions::legacy::{LegacyEthSignedTransaction, TransactionLegacy};
use aurora_engine_types::{
    U256, account_id::AccountId, parameters::connector::Erc20Identifier, types::Address,
};
use libsecp256k1::{Message, SecretKey};
use near_crypto::PublicKey;
use rlp::RlpStream;

pub trait IntoAurora<T> {
    fn into_aurora(self) -> T;
}

impl IntoAurora<AccountId> for crate::near::primitives::types::AccountId {
    fn into_aurora(self) -> AccountId {
        AccountId::new(self.as_ref()).unwrap()
    }
}

impl IntoAurora<aurora_engine_types::public_key::PublicKey> for PublicKey {
    fn into_aurora(self) -> aurora_engine_types::public_key::PublicKey {
        aurora_engine_types::public_key::PublicKey::from_str(self.to_string().as_str())
            .expect("Failed to convert NEAR PublicKey to Aurora PublicKey")
    }
}

pub fn hex_to_arr<const SIZE: usize>(hex: &str) -> anyhow::Result<[u8; SIZE]> {
    let mut output = [0u8; SIZE];

    hex::decode_to_slice(hex.trim_start_matches("0x"), &mut output)
        .map(|()| output)
        .map_err(|e| anyhow::anyhow!("Couldn't create array from the hex: {hex}, {e}"))
}

pub fn hex_to_address(h: &str) -> anyhow::Result<Address> {
    hex_to_arr(h).map(Address::from_array)
}

pub fn hex_to_vec(hex: &str) -> anyhow::Result<Vec<u8>> {
    hex::decode(hex.trim_start_matches("0x"))
        .map_err(|e| anyhow::anyhow!("Couldn't create vector from the hex: {hex}, {e}"))
}

pub fn address_from_secret_key(sk: &SecretKey) -> anyhow::Result<Address> {
    let pk = libsecp256k1::PublicKey::from_secret_key(sk);
    let hash = aurora_engine_sdk::keccak(&pk.serialize()[1..]);
    Address::try_from_slice(&hash[12..])
        .map_err(|e| anyhow::anyhow!("Couldn't create address from secret key: {e}"))
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

pub fn gen_key_pair(random: bool, seed: Option<u64>) -> anyhow::Result<(Address, SecretKey)> {
    use rand::RngCore;

    let sk = if random {
        Ok(SecretKey::random(&mut rand::thread_rng()))
    } else {
        seed.map_or_else(
            || Ok(SecretKey::default()),
            |seed| {
                let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
                let mut buffer = [0; 32];
                rng.fill_bytes(&mut buffer);
                SecretKey::parse(&buffer)
            },
        )
    }?;

    Ok((address_from_secret_key(&sk)?, sk))
}

pub fn str_to_identifier(id: &str) -> anyhow::Result<Erc20Identifier> {
    hex_to_address(id).map(Into::into).or_else(|_| {
        id.parse::<AccountId>()
            .map(Into::into)
            .map_err(|e| anyhow::anyhow!("{e}"))
    })
}
