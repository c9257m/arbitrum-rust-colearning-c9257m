// // 导入合约交互模块
// mod contract_interact;
// use contract_interact::get_contract_info;
// use eyre::Result;
//
// #[tokio::main]
// async fn main() -> Result<()> {
//     // 调用合约交互函数
//     match get_contract_info().await {
//         Ok(_) => println!("\n合约信息查询完成！"),
//         Err(e) => eprintln!("查询失败：{}", e),
//     }
//
//     Ok(())
// }


use ethers::{
    abi::Abi,
    contract::Contract,
    providers::{Http, Provider},
    types::Address,
};
use eyre::{eyre, Result};
use std::str::FromStr;
use std::sync::Arc;
use url::Url;
#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url_str = "https://arbitrum-sepolia-rpc.publicnode.com";
    let rpc_url = Url::parse(rpc_url_str)?;
    let http = Http::new(rpc_url);
    let provider =  Arc::new(Provider::new(http));
    // ===================== 2. 替换为实测可用的 ERC20 测试合约 =====================
    let contract_address_str = "0xE2A93aF53Dc1b5f4352B9B3AA204B5c75613EA2f"; // 已验证的测试 ERC20
    let contract_address = Address::from_str(contract_address_str)
        .map_err(|e| eyre!("合约地址格式错误：{}", e))?;

    // ===================== 3. 严格匹配的 ERC20 ABI（与合约实际方法一致） =====================
    let erc20_abi = r#"
    [{"inputs":[],"name":"decimals","outputs":[{"internalType":"uint8","name":"","type":"uint8"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"name","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"symbol","outputs":[{"internalType":"string","name":"","type":"string"}],"stateMutability":"view","type":"function"}]
    "#;

    // ===================== 4. 实例化合约并安全调用方法 =====================
    let abi: Abi = serde_json::from_str(erc20_abi)
            .map_err(|e| eyre!("ABI 解析失败（JSON格式错误）：{}", e))?;
    let contract = Contract::new(contract_address, abi, provider);

    // 调用 name()（增加异常捕获）
    let name: String = contract
        .method("name", ())?
        .call()
        .await
        .map_err(|e| eyre!("调用 name() 失败：{}", e))?;
    println!("合约名称：{}", name);

    // 调用 symbol()
    let symbol: String = contract
        .method("symbol", ())?
        .call()
        .await
        .map_err(|e| eyre!("调用 symbol() 失败：{}", e))?;
    println!("合约符号：{}", symbol);

    // 调用 decimals()
    let decimals: u8 = contract
        .method("decimals", ())?
        .call()
        .await
        .map_err(|e| eyre!("调用 decimals() 失败：{}", e))?;
    println!("小数位数：{}", decimals);

    Ok(())
}