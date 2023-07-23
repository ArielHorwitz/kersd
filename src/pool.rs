use ethers::prelude::{abigen, Address, U512};
use std::str::FromStr;
use eyre::{eyre, Result};
use std::collections::HashMap;
use crate::api::Client;

// Tokens
const WETH: &str = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
const USDT: &str = "0xdac17f958d2ee523a2206206994597c13d831ec7";
const USDC: &str = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
const BNB: &str = "0xB8c77482e45F1F44dE1745F52C74426C631bDD52";
const DAI: &str = "0x6b175474e89094c44da98b954eedeac495271d0f";

// Smart Contract Addresses
const FACTORY_CONTRACT: &str = "0x1c758aF0688502e49140230F6b0EBd376d429be5";
const POOL_REQ_ETH: &str = "0xa97642500517c728ce1339a466de0f10c19034cd";

// Smart Contract ABIs
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

pub fn get_example_pool() -> Result<Address> {
    Ok(Address::from_str(POOL_REQ_ETH)?)
}

pub fn get_example_tokens() -> Result<HashMap<String, Address>> {
    let mut result: HashMap<String, Address> = HashMap::new();
    result.insert("WETH".to_owned(), Address::from_str(WETH)?);
    result.insert("USDT".to_owned(), Address::from_str(USDT)?);
    result.insert("USDC".to_owned(), Address::from_str(USDC)?);
    result.insert("BNB".to_owned(), Address::from_str(BNB)?);
    result.insert("DAI".to_owned(), Address::from_str(DAI)?);
    Ok(result)
}

// TODO FIX this returns an empty vector
pub async fn get_pools(client: Client, token0: Address, token1: Address) -> Result<Vec<Address>> {
    let address: Address = FACTORY_CONTRACT.parse()?;
    let factory = KSFactory::new(address, client.clone());
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

// TODO see if possible to get trade info from a particular block
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
// The following function is a translation of:
// https://github.com/KyberNetwork/ks-classic-sc/blob/e557b57d7e4ead84caa2ec039aef280584148116/test/ksHelper.js#L16
// Whose purpose is to calculate the token amount required to sell in exchange
// for a given token amount to buy
pub fn get_exchange_rate(ti: TradeInfo, amount_out: U512) -> Result<U512> {
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

