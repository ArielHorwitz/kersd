use ethers::prelude::Middleware;
use std::fs;
use eyre::Result;
mod abi;


#[tokio::main]
async fn main() -> Result<()> {
    let api_key = fs::read_to_string("APIKEY")?;
    println!("Creating client...");
    let client = abi::get_client(&api_key)?;
    println!("Client created.");

    let block_number = client.clone().get_block_number().await?;
    println!("Block no. {block_number}");

    // let block = client.clone().get_block(block_number).await?;
    // let block_data = serde_json::to_string(&block)?;
    // println!("Block data: {block_data}");

    abi::test_contract_call(client.clone()).await?;

    let result = abi::get_pools(client.clone()).await?;
    println!("Pools: {result:?}");

    // let (r0, r1) = abi::get_trade_info(client.clone(), ADDRESS).await?;
    // println!("Trade info: {r0:?}, {r1:?}");

    Ok(())
}

