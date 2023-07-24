use ethers::prelude::{abigen, Address, Http, Middleware, Provider, U256, U512};
use eyre::{eyre, Result};
use std::sync::Arc;

pub type Client = Arc<Provider<Http>>;

pub fn get_client(api_key: &str) -> Result<Client> {
    let url = format!("https://mainnet.infura.io/v3/{api_key}");
    let provider = Provider::<Http>::try_from(url)?;
    Ok(Arc::new(provider))
}

pub async fn get_block_number(client: &Client) -> Result<u64> {
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
    Ok(IERC20::new(token, client)
        .total_supply()
        .call()
        .await?
        .as_u128())
}

// Smart Contracts
const FACTORY_CONTRACT: &str = "0x1c758aF0688502e49140230F6b0EBd376d429be5";
abigen!(
    KSFactory,
    r#"[
        function allPoolsLength() external view returns (uint256)
        function allPools(uint256) external view returns (address pool)
    ]"#,
);
abigen!(
    KSPool,
    r#"[
        function token0() external view returns (address)
        function token1() external view returns (address)
        function getTradeInfo() external virtual override view returns (uint112 _reserve0, uint112 _reserve1, uint112 _vReserve0, uint112 _vReserve1, uint256 _feeInPrecision)
    ]"#,
);

pub async fn get_all_pools(client: &Client) -> Result<Vec<Address>> {
    let address: Address = FACTORY_CONTRACT.parse()?;
    let factory = KSFactory::new(address, client.clone());
    let pool_count = factory.all_pools_length().call().await?.as_usize();
    let mut result = Vec::new();
    for i in 0..pool_count {
        let pool = factory.all_pools(U256::from(i)).call().await?;
        result.push(pool);
    }
    Ok(result)
}

#[derive(Debug, Clone)]
pub struct TradeInfo {
    pub token0: Address,
    pub token1: Address,
    pub reserve0: U512,
    pub reserve1: U512,
    pub vreserve0: U512,
    pub vreserve1: U512,
    pub fee_in_precision: U512,
}

// TODO see if possible to get trade info from a particular block
pub async fn get_trade_info(client: &Client, pool_address: &Address) -> Result<TradeInfo> {
    let contract = KSPool::new(*pool_address, client.clone());
    let (reserve0, reserve1, vreserve0, vreserve1, fee_in_precision) =
        contract.get_trade_info().call().await?;

    let token0: Address = contract.token_0().call().await?;
    let token1: Address = contract.token_1().call().await?;

    Ok(TradeInfo {
        token0,
        token1,
        reserve0: reserve0.into(),
        reserve1: reserve1.into(),
        vreserve0: vreserve0.into(),
        vreserve1: vreserve1.into(),
        fee_in_precision: fee_in_precision.into(),
    })
}

// TODO write test
// TODO enable passing exchange direction (token_in / token_out)
// The following function is a translation of:
// https://github.com/KyberNetwork/ks-classic-sc/blob/e557b57d7e4ead84caa2ec039aef280584148116/test/ksHelper.js#L16
// Whose purpose is to calculate the token amount required to sell in exchange
// for a given token amount to buy
pub fn calc_exchange_rate(ti: TradeInfo, amount_out: U512) -> Result<U512> {
    let prec = U512::exp10(18);
    // amount_in = reserveIn * amountOut / (reserveOut - amountOut)
    let nom = U512::checked_mul(ti.vreserve0, amount_out).ok_or(eyre!("math fail"))?;
    let denom = U512::checked_sub(ti.vreserve1, amount_out).ok_or(eyre!("math fail"))?;
    let amount_in = U512::checked_div(nom, denom).ok_or(eyre!("math fail"))?;
    // amountIn = floor(amount_in * precision / (precision - feeInPrecision))
    let nom = U512::checked_mul(amount_in, prec).ok_or(eyre!("math fail"))?;
    let denom = U512::checked_sub(prec, ti.fee_in_precision).ok_or(eyre!("math fail"))?;
    let amount_in = U512::checked_div(nom, denom).ok_or(eyre!("math fail"))?;
    Ok(amount_in)
}
