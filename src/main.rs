use ethers::prelude::U512;
use std::fs;
use itertools::Itertools;
use eyre::Result;
mod api;
mod pool;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = fs::read_to_string("APIKEY")?;
    let client = api::get_client(&api_key)?;
    println!("Block number: {}", api::get_block_number(client.clone()).await?);

    let tokens = pool::get_example_tokens()?;
    let combinations = tokens.values().combinations(2);
    let mut all_pools = Vec::new();
    for pair in combinations {
        let mut pools = pool::get_pools(client.clone(), *pair[0], *pair[1]).await?;
        all_pools.append(&mut pools);
    }
    println!("Pools: {all_pools:?}");
    all_pools.push(pool::get_example_pool()?);

    for pool in all_pools {
        println!("Pool: {pool:?}");
        let ti = pool::get_trade_info(client.clone(), pool).await?;
        let amount_in = pool::get_exchange_rate(ti.clone(), U512::exp10(10))?;
        println!("{amount_in:?}");
        let amount_in_2 = pool::get_exchange_rate(ti.clone(), U512::exp10(11))?;
        println!("{amount_in_2:?}");
    }
    Ok(())
}

