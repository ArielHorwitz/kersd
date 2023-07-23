use ethers::prelude::{Provider, Http, Middleware, abigen, Address};
use std::sync::Arc;
use eyre::Result;

pub type Client = Arc<Provider<Http>>;

pub fn get_client(api_key: &str) -> Result<Client> {
    let url = format!("https://mainnet.infura.io/v3/{api_key}");
    let provider = Provider::<Http>::try_from(url)?;
    Ok(Arc::new(provider))
}

pub async fn get_block_number(client: Client) -> Result<u64> {
    Ok(client.get_block_number().await?.as_u64())
}

// IERC20 Tokens
abigen!(
    IERC20,
    r#"[
        function name() public view virtual returns (string memory)
        function totalSupply() external view returns (uint256)
    ]"#,
);

#[allow(dead_code)]
pub async fn get_name(client: Client, token: Address) -> Result<String> {
    Ok(IERC20::new(token, client).name().call().await?)
}

#[allow(dead_code)]
pub async fn get_total_supply(client: Client, token: Address) -> Result<u128> {
    Ok(IERC20::new(token, client).total_supply().call().await?.as_u128())
}

