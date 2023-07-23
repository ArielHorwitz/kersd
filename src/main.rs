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
        let name0 = api::get_name(client.clone(), ti.token0).await?;
        let name1 = api::get_name(client.clone(), ti.token1).await?;
        let buy1 = U512::exp10(10);
        let buy2 = U512::exp10(11);
        if let Ok(sell1) = pool::get_exchange_rate(ti.clone(), buy1) {
            println!("Buy {buy1} {name0} for {sell1} {name1}");
        } else {
            println!("Failed to calculate exchange rate for buying {buy1} {name0} with {name1}");
        };
        if let Ok(sell2) = pool::get_exchange_rate(ti.clone(), buy2) {
            println!("Buy {buy2} {name0} for {sell2} {name1}");
        } else {
            println!("Failed to calculate exchange rate for buying {buy2} {name0} with {name1}");
        };
    }
    Ok(())
}

