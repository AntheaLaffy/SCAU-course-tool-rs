mod client;
mod login;

use crate::client::ScauClient;
use crate::login::{get_csrf_token, get_rsa_public_key, login};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

/// SCAU 教务系统登录工具
#[derive(Parser, Debug)]
#[command(name = "scau-login")]
#[command(author = "SCAUer")]
#[command(version = "0.1.0")]
#[command(about = "华南农业大学教务系统自动登录工具", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 登录教务系统
    Login {
        /// 学号
        #[arg(short, long)]
        username: String,
        /// 密码
        #[arg(short, long)]
        password: String,
    },
    /// 测试连接
    Test,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Login { username, password } => {
            println!("正在登录到华南农业大学教务系统...");
            println!();

            let mut client = ScauClient::new();

            match login(&mut client, &username, &password).await {
                Ok(response) => {
                    if response.flag {
                        println!("✅ 登录成功！");
                        println!("  消息: {}", response.msg);
                    } else {
                        println!("❌ 登录失败！");
                        println!("  消息: {}", response.msg);
                    }
                }
                Err(e) => {
                    println!("❌ 登录过程中出错: {}", e);
                    return Err(e);
                }
            }
        }
        Commands::Test => {
            println!("测试连接...");

            let client = ScauClient::new();

            // 测试获取 CSRF Token
            match get_csrf_token(&client).await {
                Ok(token) => println!("✅ CSRF Token 获取成功: {}...", &token[..32]),
                Err(e) => println!("❌ CSRF Token 获取失败: {}", e),
            }

            // 测试获取 RSA 公钥
            match get_rsa_public_key(&client).await {
                Ok(key) => {
                    println!("✅ RSA 公钥获取成功");
                    println!("  Modulus 长度: {} 字符", key.modulus.len());
                    println!("  Exponent: {}", key.exponent);
                }
                Err(e) => println!("❌ RSA 公钥获取失败: {}", e),
            }
        }
    }

    Ok(())
}
