use ethers::prelude::{Provider, Http, U64, Middleware};
use std::fs;
use eyre::Result;


#[tokio::main]
async fn main() -> Result<()> {
    let api_key = fs::read_to_string("APIKEY")?;
    let url = format!("https://mainnet.infura.io/v3/{}", api_key);
    let provider = Provider::<Http>::try_from(url)?;

    let block_number: U64 = provider.get_block_number().await?;
    println!("Block no. {}", block_number);

    let block = provider.get_block(block_number).await?;
    let block_text = serde_json::to_string(&block)?;
    println!("Got block: {}", block_text);

    Ok(())
}

