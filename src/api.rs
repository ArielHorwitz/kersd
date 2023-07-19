use ethers::prelude::{Provider, Http, Middleware, abigen, Address};
use std::sync::Arc;
use std::str::FromStr;
use eyre::Result;


pub type Client = Arc<Provider<Http>>;


const FACTORY_CONTRACT: &str = "0x1c758aF0688502e49140230F6b0EBd376d429be5";


abigen!(
    IERC20,
    r#"[
        function name() public view virtual returns (string memory)
        function totalSupply() external view returns (uint256)
    ]"#,
);

abigen!(
    KSFactory,
    r#"[
        function getPools(address token0, address token1) external override view returns (address[] memory _tokenPools)
    ]"#,
);

abigen!(
    KSPool,
    r#"[
        function getReserves() external override view returns (uint112 _reserve0, uint112 _reserve1)
    ]"#,
);

pub fn get_client(api_key: &str) -> Result<Client> {
    let url = format!("https://mainnet.infura.io/v3/{api_key}");
    let provider = Provider::<Http>::try_from(url)?;
    Ok(Arc::new(provider))
}

#[allow(dead_code)]
pub async fn print_block(client: Client) -> Result<()> {
    let block_number = client.clone().get_block_number().await?;
    println!("Block no. {block_number}");

    let block = client.clone().get_block(block_number).await?;
    let block_data = serde_json::to_string(&block)?;
    println!("Block data: {block_data}");

    Ok(())
}

#[allow(dead_code)]
pub async fn get_name(client: Client, token: &str) -> Result<String> {
    let address: Address = token.parse()?;
    Ok(IERC20::new(address, client).name().call().await?)
}

#[allow(dead_code)]
pub async fn get_total_supply(client: Client, token: &str) -> Result<u128> {
    let address: Address = token.parse()?;
    Ok(IERC20::new(address, client).total_supply().call().await?.as_u128())
}

pub async fn get_pools(client: Client, token0: &str, token1: &str) -> Result<Vec<Address>> {
    let address: Address = FACTORY_CONTRACT.parse()?;
    let factory = KSFactory::new(address, client.clone());
    let token0 = Address::from_str(token0)?;
    let token1 = Address::from_str(token1)?;
    let result = factory.get_pools(token0, token1).call().await?;

    Ok(result)
}

pub async fn get_reserves(client: Client, pool_address: Address) -> Result<(u128, u128)>
{
    let contract = KSPool::new(pool_address, client);
    let (reserve0, reserve1): (u128, u128) = contract.get_reserves().call().await?;

    Ok((reserve0, reserve1))
}

