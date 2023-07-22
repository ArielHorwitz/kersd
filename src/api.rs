use ethers::prelude::{Provider, Http, Middleware, abigen, Address, U512};
use std::sync::Arc;
use std::str::FromStr;
use eyre::{eyre, Result};

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
        function getTradeInfo() external virtual override view returns (uint112 _reserve0, uint112 _reserve1, uint112 _vReserve0, uint112 _vReserve1, uint256 _feeInPrecision)
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

#[derive(Debug, Clone)]
pub struct TradeInfo {
    pub reserve0: U512,
    pub reserve1: U512,
    pub vreserve0: U512,
    pub vreserve1: U512,
    pub fee_in_precision: U512,
}

pub async fn get_trade_info(client: Client, pool_address: Address) -> Result<TradeInfo>
{
    let contract = KSPool::new(pool_address, client);
    let (reserve0, reserve1, vreserve0, vreserve1, fee_in_precision) = contract.get_trade_info().call().await?;

    Ok(TradeInfo {
        reserve0: reserve0.into(),
        reserve1: reserve1.into(),
        vreserve0: vreserve0.into(),
        vreserve1: vreserve1.into(),
        fee_in_precision: fee_in_precision.into(),
    })
}

// TODO write test
// TODO enable passing exchange direction (token_in / token_out)
pub fn get_exchange_rate(ti: TradeInfo, amount_out: U512) -> Result<U512> {
    // https://github.com/KyberNetwork/ks-classic-sc/blob/e557b57d7e4ead84caa2ec039aef280584148116/test/ksHelper.js#L16
    let prec = U512::exp10(18);
    // imprecise_amount_in = reserveIn * amountOut / (reserveOut - amountOut)
    let nom = U512::checked_mul(ti.vreserve0, amount_out).ok_or(eyre!("math fail"))?;
    let denom = U512::checked_sub(ti.vreserve1, amount_out).ok_or(eyre!("math fail"))?;
    let imprecise_amount_in = U512::checked_div(nom, denom).ok_or(eyre!("math fail"))?;
    // amountIn = floor(imprecise_amount_in * precision / (precision - feeInPrecision))
    let nom = U512::checked_mul(imprecise_amount_in, prec).ok_or(eyre!("math fail"))?;
    let denom = U512::checked_sub(prec, ti.fee_in_precision).ok_or(eyre!("math fail"))?;
    let amount_in = U512::checked_div(nom, denom).ok_or(eyre!("math fail"))?;
    Ok(amount_in)
}

