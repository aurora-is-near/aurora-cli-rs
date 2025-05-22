use aurora_sdk_rs::aurora::ContractMethod;
use helpers::setup_sandbox;
use near_token::NearToken;

mod helpers;

struct FtTotalSupply;

impl ContractMethod for FtTotalSupply {
    type Response = String;

    fn method_name(&self) -> &'static str {
        "ft_total_supply"
    }
}

#[tokio::test]
async fn test_total_supply() -> anyhow::Result<()> {
    let (_, client, contract, _) = setup_sandbox().await?;
    let aurora_client = aurora_sdk_rs::aurora::client::Client::new(client);
    let total_supply = aurora_client.call(contract.id(), FtTotalSupply).await?;

    assert_eq!(
        total_supply,
        NearToken::from_near(1).as_yoctonear().to_string()
    );
    Ok(())
}
