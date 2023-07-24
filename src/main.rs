use ethers::prelude::{Address, U512};
use eyre::Result;
use std::{fs, time::Duration};
mod api;

const LOOP_INTERVAL_MS: u64 = 5_000;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = fs::read_to_string("APIKEY")?;
    println!("Searching for pools...");
    let client = api::get_client(&api_key)?;
    let all_pools = api::get_all_pools(&client).await?;
    println!("Found {} pools", all_pools.len());
    let mut last_block_number = 0;
    loop {
        tokio::time::sleep(Duration::from_millis(LOOP_INTERVAL_MS)).await;
        let block_number = api::get_block_number(&client).await?;
        if block_number > last_block_number {
            last_block_number = block_number;
            println!("Block number: {block_number}");
            for pool in &all_pools {
                let fut = collect_exchange_rate(client.clone(), *pool, block_number);
                task_handlers.spawn(fut);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ExchangeRate {
    block_number: u64,
    pool: Address,
    token0: Address,
    token1: Address,
    sell0: U512,
    buy1: U512,
}

async fn collect_exchange_rate(
    client: api::Client,
    pool: Address,
    block_number: u64,
) -> Result<()> {
    let ti = api::get_trade_info(&client, &pool).await?;
    let buy_amount = U512::exp10(10);
    let sell0 = api::calc_exchange_rate(ti.clone(), buy_amount)?;
    let exchange_rate = ExchangeRate {
        block_number,
        pool,
        token0: ti.token0,
        token1: ti.token1,
        sell0,
        buy1: buy_amount,
    };
    println!("{exchange_rate:?}");
    Ok(())
}
