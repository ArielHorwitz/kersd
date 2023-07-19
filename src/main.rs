use std::fs;
use itertools::Itertools;
use eyre::Result;
mod api;


const WETH: &str = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
const USDT: &str = "0xdac17f958d2ee523a2206206994597c13d831ec7";
const USDC: &str = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
const BNB: &str = "0xB8c77482e45F1F44dE1745F52C74426C631bDD52";
const DAI: &str = "0x6b175474e89094c44da98b954eedeac495271d0f";
// const POOL_REQ_ETH: &str = "0xa97642500517c728ce1339a466de0f10c19034cd";


#[tokio::main]
async fn main() -> Result<()> {
    let api_key = fs::read_to_string("APIKEY")?;
    let client = api::get_client(&api_key)?;

    let combinations = vec![WETH, USDT, USDC, BNB, DAI].into_iter().combinations(2);
    let mut all_pools = Vec::new();
    for pair in combinations {
        let mut pools = api::get_pools(client.clone(), pair[0], pair[1]).await?;
        all_pools.append(&mut pools);
    }
    println!("Pools: {all_pools:?}");
    // pools.push(Address::from_str(POOL_REQ_ETH)?);

    for pool in all_pools {
        println!("Pool: {pool:?}");
        let (reserve0, reserve1) = api::get_reserves(client.clone(), pool.clone()).await?;
        println!("Reserves: {reserve0} , {reserve1}");
    }

    Ok(())
}

