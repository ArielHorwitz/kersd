extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use ethers::prelude::Address;
use eyre::{eyre, Result};
use std::{env, path::Path, time::Duration};
use tokio::time::{sleep, timeout};
mod api;

const POLL_INTERVAL_MS: u64 = 2_000;
const DB_PATH: &str = "./db";

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    info!("Starting kersd daemon");
    // Setup
    let api_key = env::var("APIKEY")
        .expect("missing APIKEY environment variable")
        .to_owned();
    let client = api::get_client(&api_key)?;
    let db_dir = env::var("DB_PATH").unwrap_or(DB_PATH.to_owned());
    let db_dir = Path::new(&db_dir);
    info!("Data directory: {db_dir:?}");
    tokio::fs::create_dir_all(db_dir).await?;
    let poll_interval: u64 = env::var("POLL_INTERVAL_MS")
        .unwrap_or(POLL_INTERVAL_MS.to_string())
        .parse()
        .expect("Invalid POLL_INTERVAL_MS");
    info!("Poll interval: {poll_interval} ms");
    let poll_interval = Duration::from_millis(poll_interval);
    let poll_timeout = Duration::from_millis(1);
    let mut last_block_number = api::get_block_number(&client.clone())
        .await
        .expect("failed to get block number, maybe invalid APIKEY?");
    let mut task_handlers = tokio::task::JoinSet::new();
    let all_pools = api::get_all_pools(&client).await?;
    info!("Found {} pools", all_pools.len());
    // Start daemon loop
    info!("Starting daemon loop");
    loop {
        // Spawn tasks if new block exists
        debug!("Polling for new blocks >={}", last_block_number + 1);
        let block_number = api::get_block_number(&client).await?;
        if block_number > last_block_number {
            info!("New block: {block_number}");
            last_block_number = block_number;
            let block_dir = db_dir.join(format!("{block_number}"));
            tokio::fs::create_dir_all(&block_dir).await?;
            let block_dir = block_dir.to_string_lossy().to_string();
            for pool in &all_pools {
                let fut = collect_and_save(client.clone(), block_number, *pool, block_dir.clone());
                task_handlers.spawn(fut);
            }
        }
        // Poll task handlers set
        loop {
            // Wrap join_next in a timeout so that it behaves more like polling
            match timeout(poll_timeout, task_handlers.join_next()).await {
                // Timed out (no tasks done, stop polling)
                Err(_) => {
                    trace!("Awaiting {} tasks", task_handlers.len());
                    break;
                }
                // JoinSet is empty (no tasks pending, stop polling)
                Ok(None) => {
                    trace!("All {} tasks done", all_pools.len());
                    break;
                }
                // Joining task failed (log and poll again)
                Ok(Some(Err(err))) => error!("{err}"),
                // A task has failed (log and poll again)
                Ok(Some(Ok(Err(err)))) => error!("{err}"),
                // A task has completed successfully (poll again)
                Ok(Some(Ok(Ok(completed)))) => trace!("Completed task: {completed:?}"),
            }
        }
        // Loop interval
        sleep(poll_interval).await;
    }
}

async fn collect_and_save(
    client: api::Client,
    block_number: u64,
    pool: Address,
    db_dir: String,
) -> Result<(u64, Address)> {
    let snapshot = api::get_pool_snapshot(&client, &pool)
        .await
        .map_err(|err| {
            eyre!(
                "Error getting exchange rates for {} on block {}: {}",
                pool,
                block_number,
                err
            )
        })?;
    let exchange_info = api::ExchangeRates::new(block_number, pool, &snapshot);
    let json_string = serde_json::to_string_pretty(&exchange_info)?;
    let filepath = Path::new(&db_dir).join(format!("{pool:?}"));
    tokio::fs::write(filepath, json_string.into_bytes())
        .await
        .map_err(|err| {
            eyre!(
                "Error writing data to disk for pool {} on block {}: {}",
                pool,
                block_number,
                err
            )
        })?;
    Ok((block_number, pool))
}
