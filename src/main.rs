use ethers::prelude::{Address, U512};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};
use tokio::time::{sleep, timeout};
mod api;

const LOOP_INTERVAL_MS: u64 = 5_000;
const POLL_INTERVAL_MS: u64 = 50;
const DB_PATH: &str = "./db";

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = tokio::fs::read_to_string("APIKEY").await?;
    let db_dir = Path::new(DB_PATH);
    tokio::fs::create_dir_all(db_dir).await?;
    println!("Searching for pools...");
    let client = api::get_client(&api_key)?;
    let all_pools = api::get_all_pools(&client).await?;
    println!("Found {} pools", all_pools.len());
    let mut last_block_number = 0;
    let mut task_handlers = tokio::task::JoinSet::new();
    let interval = Duration::from_millis(LOOP_INTERVAL_MS);
    let poll_timeout = Duration::from_millis(POLL_INTERVAL_MS);
    loop {
        while let Ok(Some(Ok(completed))) = timeout(poll_timeout, task_handlers.join_next()).await {
            println!("{completed:?}");
        }
        let block_number = api::get_block_number(&client).await?;
        if block_number > last_block_number {
            println!("Block number: {block_number}");
            last_block_number = block_number;
            let block_dir = db_dir.join(format!("{block_number}"));
            tokio::fs::create_dir_all(&block_dir).await?;
            let block_dir = block_dir.to_string_lossy().to_string();
            for pool in &all_pools {
                let fut =
                    collect_exchange_rate(client.clone(), *pool, block_number, block_dir.clone());
                task_handlers.spawn(fut);
            }
        }
        sleep(interval).await;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    db_dir: String,
) -> Result<(u64, Address)> {
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
    let json_string = serde_json::to_string_pretty(&exchange_rate)?;
    let filepath = Path::new(&db_dir).join(format!("{pool:?}"));
    tokio::fs::write(filepath, json_string.into_bytes()).await?;
    Ok((block_number, pool))
}
