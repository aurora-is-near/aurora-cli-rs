use aurora_sdk_rs::{
    aurora::parameters::connector::{FungibleReferenceHash, FungibleTokenMetadata},
    near::primitives::borsh::BorshDeserialize,
};
use base64::Engine;

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
