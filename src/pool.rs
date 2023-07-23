use ethers::prelude::{abigen, Address, U512, U256};
use eyre::{eyre, Result};
use crate::api::Client;

// Smart Contract Addresses
const FACTORY_CONTRACT: &str = "0x1c758aF0688502e49140230F6b0EBd376d429be5";

// Smart Contract ABIs
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
        function getTradeInfo() external virtual override view returns (uint112 _reserve0, uint112 _reserve1, uint112 _vReserve0, uint112 _vReserve1, uint256 _feeInPrecision)
    ]"#,
);

pub async fn get_all_pools(client: Client) -> Result<Vec<Address>> {
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

