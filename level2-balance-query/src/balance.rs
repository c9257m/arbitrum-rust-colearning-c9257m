use ethers::{
    providers::{Provider,Http},
    types::Address,
    utils::format_units,
    middleware::Middleware,
};
use url::Url;
use eyre::Result;

pub async fn get_arbitrum_balance_rpc(address_str: &str) -> Result<String> {
    let rpc_url_str = "https://arbitrum-sepolia-rpc.publicnode.com";
    let rpc_url = Url::parse(rpc_url_str)?;
    let http = Http::new(rpc_url);
    let provider = Provider::new(http);
    let address: Address = address_str.parse()?;
    let balance_wei = provider.get_balance(address, None).await?;
    let balance_eth = format_units(balance_wei, 18)?;
    Ok(balance_eth)
}