use ethers::prelude::{Address, U512};
use eyre::{eyre, Error, Result};
use serde::{Deserialize, Serialize};
use std::{env, path::Path, time::Duration};
use tokio::time::{sleep, timeout};
mod api;

const POLL_INTERVAL_MS: u64 = 2_000;
const DB_PATH: &str = "./db";

#[tokio::main]
async fn main() -> Result<()> {
    println!("KERSD DAEMON STARTING");
    // Setup
    let api_key = env::var("APIKEY")
        .expect("missing APIKEY environment variable")
        .to_owned();
    let client = api::get_client(&api_key)?;
    let db_dir = env::var("DB_PATH").unwrap_or(DB_PATH.to_owned());
    let db_dir = Path::new(&db_dir);
    tokio::fs::create_dir_all(db_dir).await?;
    let poll_interval: u64 = env::var("POLL_INTERVAL_MS")
        .unwrap_or(POLL_INTERVAL_MS.to_string())
        .parse()
        .expect("Invalid POLL_INTERVAL_MS");
    let poll_interval = Duration::from_millis(poll_interval);
    let poll_timeout = Duration::from_millis(1);
    let mut last_block_number = api::get_block_number(&client.clone())
        .await
        .expect("failed to get block number, maybe invalid APIKEY?");
    let mut task_handlers = tokio::task::JoinSet::new();
    let all_pools = api::get_all_pools(&client).await?;
    println!("Found {} pools", all_pools.len());
    // Start daemon loop
    loop {
        // Spawn tasks if new block exists
        let block_number = api::get_block_number(&client).await?;
        if block_number > last_block_number {
            println!(
                "Block number: {block_number} (spawning {} tasks)",
                all_pools.len()
            );
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
        // Poll task handlers set
        loop {
            // Wrap join_next in a timeout so that it behaves more like polling
            match timeout(poll_timeout, task_handlers.join_next()).await {
                // Timed out (no tasks done, stop polling)
                Err(_) => break,
                // JoinSet is empty (no tasks pending, stop polling)
                Ok(None) => break,
                // Joining task failed (log and poll again)
                Ok(Some(Err(err))) => log(eyre!(err)),
                // A task has failed (log and poll again)
                Ok(Some(Ok(Err(err)))) => log(eyre!(err)),
                // A task has completed successfully (poll again)
                Ok(Some(Ok(Ok(_)))) => (),
            }
        }
        // Loop interval
        sleep(poll_interval).await;
    }
}

fn log(err: Error) {
    eprintln!("{}", format!("{err:?}").replace('\n', " "));
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
) -> Result<()> {
    // Wrap inner function to catch errors and include context
    do_collect_exchange_rate(client, pool, block_number, db_dir)
        .await
        .map_err(|err| eyre!("Error collecting block {block_number} for pool {pool}: {err}"))
}

async fn do_collect_exchange_rate(
    client: api::Client,
    pool: Address,
    block_number: u64,
    db_dir: String,
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
    let json_string = serde_json::to_string_pretty(&exchange_rate)?;
    let filepath = Path::new(&db_dir).join(format!("{pool:?}"));
    tokio::fs::write(filepath, json_string.into_bytes()).await?;
    Ok(())
}
