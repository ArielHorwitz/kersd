use ethers::prelude::{Provider, Http, abigen, Address, U256};
use std::sync::Arc;
use eyre::Result;


pub type Client = Arc<Provider<Http>>;


const WETH_ADDRESS: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const FACTORY_CONTRACT: &str = "0x1c758aF0688502e49140230F6b0EBd376d429be5";


abigen!(
    IERC20,
    r#"[
        function totalSupply() external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
        function transfer(address recipient, uint256 amount) external returns (bool)
        function allowance(address owner, address spender) external view returns (uint256)
        function approve(address spender, uint256 amount) external returns (bool)
        function transferFrom( address sender, address recipient, uint256 amount) external returns (bool)
        event Transfer(address indexed from, address indexed to, uint256 value)
        event Approval(address indexed owner, address indexed spender, uint256 value)
    ]"#,
);

abigen!(
    KSFactory,
    r#"[
        function allPoolsLength() external override view returns (uint256)
    ]"#,
    // function getPools(IERC20 token0, IERC20 token1) external override view returns (address[] memory _tokenPools)
);


pub fn get_client(api_key: &str) -> Result<Client> {
    let url = format!("https://mainnet.infura.io/v3/{api_key}");
    let provider = Provider::<Http>::try_from(url)?;
    Ok(Arc::new(provider))
}

pub async fn get_pools(client: Client) -> Result<U256>
{
    let address: Address = FACTORY_CONTRACT.parse()?;
    let contract = KSFactory::new(address, client);
    let result = contract.all_pools_length().call().await?;

    Ok(result)
}

// pub async fn get_trade_info<T>(client: Client<T>, address: &str) -> Result<(u128, u128)>
// {
//     abigen!(
//         IERC20,
//         r#"[
//             function getReserves() external override view returns (uint112 _reserve0, uint112 _reserve1)
//             function getTradeInfo() external virtual override view returns (uint112 _reserve0, uint112 _reserve1, uint112 _vReserve0, uint112 _vReserve1, uint256 _feeInPrecision)
//         ]"#,
//     );

//     let address: Address = address.parse()?;
//     let contract = IERC20::new(address, client);
//     let (reserve0, reserve1, _vreserve0, _vreserve1, _fee_in_precision):
//         (u128, u128, u128, u128, U256) = contract.get_trade_info().call().await?;

//     Ok((reserve0, reserve1))
// }


pub async fn test_contract_call(client: Client) -> Result<()> {
    let address: Address = WETH_ADDRESS.parse()?;
    let contract = IERC20::new(address, client);
    let total_supply = contract.total_supply().call().await?;
    println!("WETH total supply is {total_supply:?}");
    Ok(())
}

