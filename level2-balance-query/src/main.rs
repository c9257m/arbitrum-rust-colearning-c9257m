mod balance;
use balance::get_arbitrum_balance_rpc;
use eyre::Result;
// 主函数：调用余额查询函数并处理结果
#[tokio::main]
async fn main() -> Result<()> {
    // 替换为你要查询的 Arbitrum 测试网地址
    let test_address = "0xF14Beb2A3A05eFBb1e6cfF1F1Ac073468412bA37";

    // 调用 balance 模块中的函数
    match get_arbitrum_balance_rpc(test_address).await {
        Ok(balance) => println!(" 地址 {} 的 ETH 余额：{} ETH", test_address, balance),
        Err(e) => eprintln!("查询失败：{}", e),
    }
    Ok(())
}