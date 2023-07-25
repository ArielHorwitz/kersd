use ethers::prelude::{abigen, Address, Http, Middleware, Provider, U256, U512};
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, sync::Arc};

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

// Token TokenTrade Rates
#[derive(Debug, Clone)]
pub struct PoolSnapshot {
    pub token0: Address,
    pub token1: Address,
    pub reserve0: U512,
    pub reserve1: U512,
    pub vreserve0: U512,
    pub vreserve1: U512,
    pub fee_in_precision: U512,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRates {
    block_number: u64,
    pool: Address,
    token0: Address,
    token1: Address,
    sell0_buy1: Option<TokenTrade>,
    sell1_buy0: Option<TokenTrade>,
}

impl ExchangeRates {
    pub fn new(block_number: u64, pool: Address, ti: &PoolSnapshot) -> Self {
        Self {
            block_number,
            pool,
            token0: ti.token0,
            token1: ti.token1,
            sell0_buy1: get_best_trade(ti, true),
            sell1_buy0: get_best_trade(ti, false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTrade {
    pub sell_amount: U512,
    pub buy_amount: U512,
    pub exchange_rate: f64,
}

impl TokenTrade {
    pub fn new(sell_amount: U512, buy_amount: U512) -> Self {
        let fsell: f64 = sell_amount.to_string().parse().unwrap_or(0.0);
        let fbuy: f64 = buy_amount.to_string().parse().unwrap_or(0.0);
        let exchange_rate: f64 = match fbuy != 0.0 && fsell != 0.0 {
            true => fbuy / fsell,
            false => 0.0,
        };
        Self {
            sell_amount,
            buy_amount,
            exchange_rate,
        }
    }
}

impl PartialOrd for TokenTrade {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.exchange_rate.partial_cmp(&other.exchange_rate)
    }
}

impl PartialEq for TokenTrade {
    fn eq(&self, other: &Self) -> bool {
        self.exchange_rate == other.exchange_rate
    }
}

pub async fn get_pool_snapshot(client: &Client, pool_address: &Address) -> Result<PoolSnapshot> {
    let contract = KSPool::new(*pool_address, client.clone());
    let (reserve0, reserve1, vreserve0, vreserve1, fee_in_precision) =
        contract.get_trade_info().call().await?;

    let token0: Address = contract.token_0().call().await?;
    let token1: Address = contract.token_1().call().await?;

    Ok(PoolSnapshot {
        token0,
        token1,
        reserve0: reserve0.into(),
        reserve1: reserve1.into(),
        vreserve0: vreserve0.into(),
        vreserve1: vreserve1.into(),
        fee_in_precision: fee_in_precision.into(),
    })
}

fn get_best_trade(ti: &PoolSnapshot, sell0: bool) -> Option<TokenTrade> {
    let mut best_trade = None;
    for buy_amount_exp in 0..15 {
        let buy_amount = U512::from(10).pow(buy_amount_exp.into());
        if let Ok(sell_amount) = calc_sell_amount(ti, buy_amount, sell0) {
            let trade = TokenTrade::new(sell_amount, buy_amount);
            if let Some(best) = &best_trade {
                if &trade > best {
                    best_trade = Some(trade);
                }
            } else if best_trade.is_none() && trade.exchange_rate > 0. {
                best_trade = Some(trade);
            }
        }
    }
    best_trade
}

/// Calculate the token amount required to sell for a given amount to buy
/// A translation of:
/// https://github.com/KyberNetwork/ks-classic-sc/blob/e557b57d7e4ead84caa2ec039aef280584148116/test/ksHelper.js#L16
fn calc_sell_amount(ti: &PoolSnapshot, buy_amount: U512, sell0: bool) -> Result<U512> {
    // sell = in, buy = out
    let prec = U512::exp10(18);
    let (reserve_in, reserve_out) = match sell0 {
        true => (ti.vreserve1, ti.vreserve0),
        false => (ti.vreserve0, ti.vreserve1),
    };
    // amount_in = reserveIn * amountOut / (reserveOut - amountOut)
    let nom = U512::checked_mul(reserve_in, buy_amount).ok_or(eyre!("math fail"))?;
    let denom = U512::checked_sub(reserve_out, buy_amount).ok_or(eyre!("math fail"))?;
    let amount_in = U512::checked_div(nom, denom).ok_or(eyre!("math fail"))?;
    // amountIn = floor(amount_in * precision / (precision - feeInPrecision))
    let nom = U512::checked_mul(amount_in, prec).ok_or(eyre!("math fail"))?;
    let denom = U512::checked_sub(prec, ti.fee_in_precision).ok_or(eyre!("math fail"))?;
    let amount_in = U512::checked_div(nom, denom).ok_or(eyre!("math fail"))?;
    Ok(amount_in)
}
