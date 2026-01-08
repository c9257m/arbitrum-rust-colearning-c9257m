use ethers::{
    providers::{Provider, Http},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionRequest,U64, U256, H256},
    utils::{format_units,parse_units},
    middleware::{Middleware,SignerMiddleware},
};
use eyre::{Result, Context};
use std::str::FromStr;
use std::env;
use std::sync::Arc;
use dotenv::dotenv;
use ethers::types::transaction::eip2718::TypedTransaction;

/// 检查地址格式是否有效
pub fn validate_address(address_str: &str) -> Result<Address> {
    let address = Address::from_str(address_str)
        .context(format!("无效的地址格式: {}", address_str))?;
    
    // 检查是否是零地址
    if address == Address::zero() {
        return Err(eyre::eyre!("地址不能为零地址"));
    }
    
    Ok(address)
}

/// 获取 Arbitrum Sepolia 测试网 Provider
pub fn get_arbitrum_sepolia_provider() -> Result<Provider<Http>> {
    let rpc_url = "https://arbitrum-sepolia-rpc.publicnode.com";
    let provider = Provider::<Http>::try_from(rpc_url)
        .context("无法连接到 Arbitrum Sepolia 测试网")?;
    
    Ok(provider)
}
/// 创建带签名者的客户端
pub fn create_signer_client(
    provider: Provider<Http>,
    wallet: LocalWallet,
) -> SignerMiddleware<Provider<Http>, LocalWallet> {
    SignerMiddleware::new(provider, wallet)
}

/// 从环境变量加载钱包
pub fn load_wallet_from_env() -> Result<LocalWallet> {
    dotenv().ok(); // 加载 .env 文件
    
    let private_key = env::var("PRIVATE_KEY")
        .context("请在 .env 文件中设置 PRIVATE_KEY 环境变量")?;
    
    // 移除可能的 "0x" 前缀
    let private_key = private_key.trim_start_matches("0x");
    
    let wallet = private_key.parse::<LocalWallet>()
        .context("私钥格式无效")?;
    
    Ok(wallet)
}

/// 获取账户余额（ETH）
pub async fn get_balance_eth(address: Address) -> Result<String> {
    let provider = get_arbitrum_sepolia_provider()?;
    
    let balance_wei = provider.get_balance(address, None)
        .await
        .context("获取余额失败")?;
    
    let balance_eth = format_units(balance_wei, "ether")?;
    
    Ok(balance_eth)
}

/// 计算合适的 Gas 价格（添加 10% 溢价以确保快速确认）
pub async fn get_gas_price_with_premium() -> Result<U256> {
    let provider = get_arbitrum_sepolia_provider()?;
    
    let base_gas_price = provider.get_gas_price()
        .await
        .context("获取 Gas 价格失败")?;
    
    // 添加 10% 溢价
    let premium = base_gas_price * 110 / 100;
    
    Ok(premium)
}

/// 估算转账所需的 Gas 限额
pub async fn estimate_gas_limit(
    from: Address,
    to: Address,
    value: U256,
) -> Result<U256> {
    let provider = get_arbitrum_sepolia_provider()?;
    
    // 创建交易请求
    let tx = TransactionRequest::new()
        .from(from)
        .to(to)
        .value(value);

    let typed_tx: TypedTransaction = tx.into();
    
    // 估算 Gas 限额
    let gas_limit = provider.estimate_gas(&typed_tx, None)
        .await
        .unwrap_or_else(|_| U256::from(21000)); // 失败时使用基础值
    
    // 添加 20% 缓冲
    let gas_limit_with_buffer = gas_limit * 120 / 100;
    
    Ok(gas_limit_with_buffer)
}

/// 发送 ETH 转账
pub async fn send_eth_transfer(
    from_wallet: LocalWallet,
    to_address: Address,
    amount_eth: &str,
) -> Result<H256> {
    let provider = get_arbitrum_sepolia_provider()?;
    
  // 2. 设置链 ID（Arbitrum Sepolia = 421614）
    let wallet = from_wallet.clone().with_chain_id(421614u64);
    // 创建带签名者的客户端
    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet
    ));

    // 使用 ethers 官方工具解析金额
    let parsed_amount = parse_units(amount_eth, "ether").context("金额格式无效")?;
    let amount_wei: U256 = parsed_amount.into();

    // 获取 nonce
    let nonce = client.get_transaction_count(client.address(), None)
        .await
        .context("获取 nonce 失败")?;
    
    // 获取 Gas 价格
    let gas_price = get_gas_price_with_premium().await?;
    
    // 估算 Gas 限额
    let gas_limit = estimate_gas_limit(
        client.address(),
        to_address,
        amount_wei,
    ).await?;
    
    println!("交易参数:");
    println!("• From: {:?}", client.address());
    println!("• To: {:?}", to_address);
    println!("• 金额: {} ETH", amount_eth);
    println!("• Nonce: {}", nonce);
    println!("• Gas 价格: {} wei", gas_price);
    println!("• Gas 限额: {}", gas_limit);
    
    // 计算预估 Gas 费
    let estimated_fee = gas_price * gas_limit;
    let estimated_fee_eth = format_units(estimated_fee, "ether")?;
    println!("• 预估 Gas 费: {} ETH", estimated_fee_eth);
    
    // 检查余额是否足够
    let balance = client.get_balance(client.address(), None).await?;
    let total_cost = amount_wei + estimated_fee;
    
    if balance < total_cost {
        let balance_eth = format_units(balance, "ether")?;
        let total_cost_eth = format_units(total_cost, "ether")?;
        return Err(eyre::eyre!(
            "余额不足！\n当前余额: {} ETH\n所需金额: {} ETH\n缺少: {} ETH",
            balance_eth,
            total_cost_eth,
            format_units(total_cost - balance, "ether")?
        ));
    }
    
    // 构建并发送交易
    println!("\n正在发送交易...");
    
    let tx = TransactionRequest::new()
        .to(to_address)
        .value(amount_wei)
        .gas_price(gas_price)
        .gas(gas_limit)
        .nonce(nonce);
    
    let pending_tx = client.send_transaction(tx, None).await
        .context("发送交易失败")?;
    
    let tx_hash = pending_tx.tx_hash();
    println!("✓ 交易已发送！交易哈希: {:?}", tx_hash);
    
    // 等待交易确认
    println!("等待交易确认...");
    let receipt = pending_tx
        .await
        .context("等待交易确认失败")?;
    
    match receipt {
        Some(receipt) => {
            println!("✓ 交易已确认！");
            println!("  区块高度: {:?}", receipt.block_number.unwrap_or_default());
            println!("  Gas 使用量: {:?}", receipt.gas_used.unwrap_or_default());
            println!("  状态: {}", 
                if receipt.status.unwrap_or_default() == U64::from(1) {
                    "成功"
                } else {
                    "失败"
                }
            );
        }
        None => {
            println!("⚠ 交易确认，但未收到收据");
        }
    }
    
    Ok(tx_hash)
}

/// 主函数：执行完整的转账流程
#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Arbitrum Sepolia 测试网 ETH 转账脚本 ===\n");
    
    // 1. 加载发送方钱包
    println!("1. 加载发送方钱包...");
    let from_wallet = match load_wallet_from_env() {
        Ok(wallet) => {
            println!("钱包加载成功");
            println!("地址: {:?}", wallet.address());
            wallet
        }
        Err(e) => {
            eprintln!("加载钱包失败: {}", e);
            eprintln!("\n请按照以下步骤设置:");
            eprintln!("1. 在项目根目录创建 .env 文件");
            eprintln!("2. 在 .env 中添加: PRIVATE_KEY=你的私钥（不带0x前缀）");
            eprintln!("3. 确保私钥对应的地址有测试网 ETH");
            return Ok(());
        }
    };
    
    // 2. 输入接收方地址
    println!("\n2. 输入接收方地址...");
    
    // 接收方地址
    let default_to_address = "0x6FC35791B6D73Fc90951aF166134fFDBa4E933E9";
    // 验证地址
    let to_address = match validate_address(default_to_address) {
        Ok(addr) => {
            println!("接收方地址有效: {:?}", addr);
            addr
        }
        Err(e) => {
            eprintln!("接收方地址无效: {}", e);
            return Ok(());
        }
    };
    
    //
    // 小金额测试
    let amount_eth = "0.00001";
    if amount_eth.is_empty() {
        eprintln!("转账金额不能为空");
        return Ok(());
    }
    
    // 验证金额
    if let Err(e) = amount_eth.parse::<f64>() {
        eprintln!("无效的金额格式: {}", e);
        return Ok(());
    }
    
    println!("转账金额: {} ETH", amount_eth);
    
    // 4. 检查发送方余额
    println!("\n4. 检查余额...");
    let balance = get_balance_eth(from_wallet.address()).await?;
    println!("  发送方余额: {} ETH", balance);
    
    let receiver_balance = get_balance_eth(to_address).await?;
    println!("  接收方余额: {} ETH", receiver_balance);
    
    // 5. 发送转账
    println!("\n5. 执行转账...");
    match send_eth_transfer(from_wallet.clone(), to_address, amount_eth).await {
        Ok(tx_hash) => {
            println!("\n🎉 转账成功！");
            println!("交易哈希: 0x{}", hex::encode(tx_hash.as_bytes()));
            
            // 构建区块浏览器链接
            let explorer_url = format!(
                "https://sepolia.arbiscan.io/tx/0x{}",
                hex::encode(tx_hash.as_bytes())
            );
            println!("查看交易: {}", explorer_url);
        }
        Err(e) => {
            eprintln!("\n转账失败: {}", e);
            return Ok(());
        }
    }
    
    // 6. 转账后余额检查
    println!("\n6. 转账后余额检查...");
    let new_sender_balance = get_balance_eth(from_wallet.address()).await?;
    let new_receiver_balance = get_balance_eth(to_address).await?;
    
    println!("  发送方新余额: {} ETH", new_sender_balance);
    println!("  接收方新余额: {} ETH", new_receiver_balance);
    
    Ok(())
}