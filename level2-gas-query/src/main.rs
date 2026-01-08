use ethers::{
    providers::{Provider, Http},
    types::{U256},
    utils::{format_units},
    middleware::Middleware,
};
use eyre::{Result, Context};

/// 获取 Arbitrum 测试网 Gas 价格（单位：gwei）
pub async fn get_arbitrum_gas_price() -> Result<U256> {
    let rpc_url = "https://arbitrum-sepolia-rpc.publicnode.com";
    let provider = Provider::<Http>::try_from(rpc_url)
        .context("Failed to create provider")?;
    
    let gas_price = provider.get_gas_price()
        .await
        .context("Failed to get gas price")?;
    
    Ok(gas_price)
}

/// 获取基础的 ETH 转账 Gas 限额
/// 普通 ETH 转账的基础 Gas 限额是 21000
pub fn get_base_transfer_gas_limit() -> U256 {
    U256::from(21000)
}

/// 计算预估的转账 Gas 费（单位：ETH）
pub async fn estimate_transfer_gas_fee() -> Result<String> {
    // 1. 获取实时 Gas 价格
    let gas_price = get_arbitrum_gas_price().await?;
    
    // 2. 获取基础 Gas 限额
    let gas_limit = get_base_transfer_gas_limit();
    
    // 3. 计算 Gas 费（Gas 费 = Gas 价格 × Gas 限额）
    let gas_fee_wei = gas_price * gas_limit;
    
    // 4. 转换为 ETH 单位
    let gas_fee_eth = format_units(gas_fee_wei, 18)?;
    
    Ok(gas_fee_eth)
}

/// 更详细的版本，返回所有信息
pub async fn get_gas_info() -> Result<GasInfo> {
    let rpc_url = "https://arbitrum-sepolia-rpc.publicnode.com";
    let provider = Provider::<Http>::try_from(rpc_url)
        .context("Failed to create provider")?;
    
    // 获取 Gas 价格
    let gas_price_wei = provider.get_gas_price().await?;
    
    // 转换为不同的单位以便显示
    let gas_price_gwei = format_units(gas_price_wei, "gwei")?;
    
    // 基础 Gas 限额
    let base_gas_limit = U256::from(21000);
    
    // 计算预估费用
    let estimated_fee_wei = gas_price_wei * base_gas_limit;
    let estimated_fee_eth = format_units(estimated_fee_wei, "ether")?;
    
    Ok(GasInfo {
        gas_price_wei,
        gas_price_gwei,
        base_gas_limit,
        estimated_fee_wei,
        estimated_fee_eth,
    })
}

/// 包含实时 Gas 信息的数据结构
#[derive(Debug, Clone)]
pub struct GasInfo {
    /// Gas 价格（wei）
    pub gas_price_wei: U256,
    /// Gas 价格（gwei）
    pub gas_price_gwei: String,
    /// 基础 Gas 限额
    pub base_gas_limit: U256,
    /// 预估费用（wei）
    pub estimated_fee_wei: U256,
    /// 预估费用（ETH）
    pub estimated_fee_eth: String,
}

impl GasInfo {
    /// 格式化显示所有信息
    pub fn display(&self) -> String {
        format!(
            "Gas 价格: {} gwei ({} wei)\n\
             基础 Gas 限额: {}\n\
             预估转账费用: {} ETH ({} wei)",
            self.gas_price_gwei,
            self.gas_price_wei,
            self.base_gas_limit,
            self.estimated_fee_eth,
            self.estimated_fee_wei
        )
    }
}

/// 主函数
#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Arbitrum Sepolia 测试网 Gas 费估算 ===\n");
    
    // 1. 获取基础 Gas 信息
    match get_gas_info().await {
        Ok(gas_info) => {
            println!("当前网络 Gas 信息:");
            println!("{}", gas_info.display());
            println!();
        }
        Err(e) => eprintln!("获取 Gas 信息失败: {}", e),
    }
    Ok(())
}