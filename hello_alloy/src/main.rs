 
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::Address;
use std::error::Error;
use alloy::sol;

sol! { 
   #[sol(rpc)] 
   contract HelloWeb3 { 
        function hello_web3() pure public returns(string memory); 
   } 
} 

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let rpc_url = "https://arbitrum-sepolia-rpc.publicnode.com".parse()?;
 
    let provider = ProviderBuilder::new().connect_http(rpc_url); 
    
    let latest_block = provider.get_block_number().await?;

    println!("Latest block number: {latest_block}");
    println!("Hello web3");
   

    let contract_address: Address = "0x3f1f78ED98Cd180794f1346F5bD379D5Ec47DE90".parse()?;
    
    let contract = HelloWeb3::new(contract_address, provider);
    
    let result = contract.hello_web3().call().await?;
    println!("合约返回: {}", result);
 
    Ok(())
}