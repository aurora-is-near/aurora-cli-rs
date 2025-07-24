use std::path::Path;

use aurora_sdk_rs::{
    aurora::parameters::connector::{FungibleReferenceHash, FungibleTokenMetadata},
    near::{
        crypto::{InMemorySigner, PublicKey, SecretKey},
        primitives::{borsh::BorshDeserialize, types::AccountId},
    },
};
use base64::Engine;
use serde::{Deserialize, Serialize};

pub mod output;

pub fn parse_ft_metadata(input: Option<String>) -> anyhow::Result<FungibleTokenMetadata> {
    let Some(input) = input else {
        return Ok(default_ft_metadata());
    };
    let json: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&input)?;

    Ok(FungibleTokenMetadata {
        spec: json
            .get("spec")
            .ok_or_else(|| anyhow::anyhow!("Missing spec field"))?
            .to_string(),
        name: json
            .get("name")
            .ok_or_else(|| anyhow::anyhow!("Missing name field"))?
            .to_string(),
        symbol: json
            .get("symbol")
            .ok_or_else(|| anyhow::anyhow!("Missing symbol field"))?
            .to_string(),
        icon: json.get("icon").map(ToString::to_string),
        reference: json.get("reference").map(ToString::to_string),
        reference_hash: json
            .get("reference_hash")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("reference_hash must be a string"))
            .and_then(|s| {
                base64::engine::general_purpose::STANDARD
                    .decode(s)
                    .map_err(Into::into)
            })
            .and_then(|bytes| FungibleReferenceHash::try_from_slice(&bytes).map_err(Into::into))
            .ok(),
        decimals: serde_json::from_value(
            json.get("decimals")
                .ok_or_else(|| anyhow::anyhow!("Missing decimals field"))?
                .clone(),
        )?,
    })
}

fn default_ft_metadata() -> FungibleTokenMetadata {
    FungibleTokenMetadata {
        spec: "ft-1.0.0".to_string(),
        name: "localETH".to_string(),
        symbol: "localETH".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 18,
    }
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
    pub account_id: AccountId,
    pub public_key: PublicKey,
    // Credential files generated which near cli works with have private_key
    // rather than secret_key field. To make it possible to read those from
    // neard add private_key as an alias to this field so either will work.
    #[serde(alias = "private_key")]
    pub secret_key: SecretKey,
}

#[derive(Serialize, Deserialize)]
struct KeyFileWithoutPublicKey {
    pub account_id: AccountId,
    // Credential files generated which near cli works with have private_key
    // rather than secret_key field. To make it possible to read those from
    // neard add private_key as an alias to this field so either will work.
    #[serde(alias = "private_key")]
    pub secret_key: SecretKey,
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
