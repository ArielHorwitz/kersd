use ethers::prelude::U512;
use std::fs;
use eyre::Result;
mod api;
mod pool;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = fs::read_to_string("APIKEY")?;
    let client = api::get_client(&api_key)?;
    println!("Block number: {}", api::get_block_number(client.clone()).await?);

    let all_pools = pool::get_all_pools(client.clone()).await?;
    println!("Found {} pools", all_pools.len());

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

