use crate::client::ScauClient;
use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// RSA 公钥响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RsaPublicKey {
    pub modulus: String,
    pub exponent: String,
}

/// 登录响应
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub flag: bool,
    pub msg: String,
}

/// 获取 CSRF Token
///
/// 从登录页面 HTML 中提取 csrftoken
pub async fn get_csrf_token(client: &ScauClient) -> Result<String> {
    let url = client.url("/jwglxt/xtgl/login_slogin.html");

    let response = client
        .client()
        .get(&url)
        .send()
        .await
        .context("Failed to fetch login page")?;

    let text = response.text().await.context("Failed to get response text")?;

    // 从页面中提取 csrftoken
    // 格式: <input type="hidden" id="csrftoken" value="xxx">
    if let Some(start) = text.find("id=\"csrftoken\"") {
        if let Some(value_start) = text[start..].find("value=\"") {
            let value_start = start + value_start + 7;
            if let Some(value_end) = text[value_start..].find('"') {
                let token = text[value_start..value_start + value_end].to_string();
                // 只取逗号前的部分
                let clean_token = token.split(',').next().unwrap_or(&token).to_string();
                return Ok(clean_token);
            }
        }
    }

    // 备选方案：查找 name="csrftoken"
    if let Some(start) = text.find("name=\"csrftoken\"") {
        if let Some(value_start) = text[start..].find("value=\"") {
            let value_start = start + value_start + 7;
            if let Some(value_end) = text[value_start..].find('"') {
                let token = text[value_start..value_start + value_end].to_string();
                // 只取逗号前的部分
                let clean_token = token.split(',').next().unwrap_or(&token).to_string();
                return Ok(clean_token);
            }
        }
    }

    anyhow::bail!("Could not find csrftoken in login page")
}

/// 获取 RSA 公钥
pub async fn get_rsa_public_key(client: &ScauClient) -> Result<RsaPublicKey> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let url = format!(
        "{}?time={}",
        client.url("/jwglxt/xtgl/login_getPublicKey.html"),
        timestamp
    );

    let response = client
        .client()
        .get(&url)
        .send()
        .await
        .context("Failed to fetch RSA public key")?;

    if response.status() != StatusCode::OK {
        anyhow::bail!("Failed to get RSA public key: status {}", response.status());
    }

    let json = response.json().await.context("Failed to parse RSA public key")?;

    Ok(json)
}

/// 登录函数
pub async fn login(client: &mut ScauClient, username: &str, password: &str) -> Result<LoginResponse> {
    // 1. 获取 CSRF Token
    println!("[1/3] 获取 CSRF Token...");
    let csrf_token = get_csrf_token(client).await?;

    // 2. 获取 RSA 公钥
    println!("[2/3] 获取 RSA 公钥...");
    let rsa_key = get_rsa_public_key(client).await?;

    // 3. 加密密码
    println!("[3/3] 登录中...");
    let encrypted_password = encrypt_password(&rsa_key, password)?;

    // 4. 发送登录请求
    let url = client.url("/jwglxt/xtgl/login_slogin.html");

    let params = [
        ("csrftoken", csrf_token.as_str()),
        ("language", "zh_CN"),
        ("yhm", username),
        ("mm", encrypted_password.as_str()),
    ];

    // 构建请求，添加必要的 headers
    let request = client
        .client()
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Referer", "https://jwzf.scau.edu.cn/jwglxt/xtgl/login_slogin.html")
        .form(&params);

    let response = request.send().await.context("Failed to send login request")?;

    // 获取最终 URL（处理 302 跳转）
    let final_url = response.url().to_string();
    let text = response.text().await.context("Failed to get response text")?;

    // 判断逻辑：
    // 1. 如果跳转到了主页/菜单页面，说明成功
    if final_url.contains("index_initMenu.html") || final_url.contains("xtgl/index.html") {
        return Ok(LoginResponse { flag: true, msg: "登录成功".to_string() });
    }

    // 2. 检查 HTML 里的错误提示
    if text.contains("用户名或密码不正确") || text.contains("密码错误") {
        anyhow::bail!("登录失败：用户名或密码不正确");
    } else if text.contains("验证码") {
        anyhow::bail!("登录失败：需要验证码");
    }

    // 3. 如果响应是 JSON（可能是加了 X-Requested-With 的情况）
    if let Ok(json) = serde_json::from_str::<LoginResponse>(&text) {
        return Ok(json);
    }

    // 失败兜底
    anyhow::bail!("登录失败，当前 URL: {}", final_url);
}

/// 使用 RSA 加密密码
fn encrypt_password(rsa_key: &RsaPublicKey, password: &str) -> Result<String> {
    use base64::{Engine, Engine as _};
    use num_bigint_dig::BigUint;
    use rsa::pkcs1v15::Pkcs1v15Encrypt;
    use rand::rngs::OsRng;

    // 解析 modulus (Base64 解码 - 需要先处理 URL 转义)
    // 正方教务系统返回的 JSON 中 `\/` 表示 `/`
    let modulus_unescaped = rsa_key.modulus.replace("\\/", "/");
    let modulus_bytes = base64::prelude::BASE64_STANDARD
        .decode(&modulus_unescaped)
        .context("Failed to decode modulus base64")?;
    let modulus = BigUint::from_bytes_be(&modulus_bytes);

    // 解析 exponent (Base64 解码)
    let exponent_unescaped = rsa_key.exponent.replace("\\/", "/");
    let exponent_bytes = base64::prelude::BASE64_STANDARD
        .decode(&exponent_unescaped)
        .context("Failed to decode exponent base64")?;
    let exponent = BigUint::from_bytes_be(&exponent_bytes);

    // 构建 RSA 公钥
    let public_key = rsa::RsaPublicKey::new(modulus, exponent)
        .context("Failed to create RSA public key")?;

    // 加密 (PKCS1v15)
    let padding = Pkcs1v15Encrypt;
    let mut rng = OsRng;
    let encrypted = public_key
        .encrypt(&mut rng, padding, password.as_bytes())
        .context("Failed to encrypt password")?;

    // Base64 编码
    let result = base64::Engine::encode(&base64::prelude::BASE64_STANDARD, &encrypted);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_csrf_token() {
        let client = ScauClient::new();
        let token = get_csrf_token(&client).await;
        assert!(token.is_ok(), "Should get CSRF token");
        let token = token.unwrap();
        assert!(!token.is_empty(), "CSRF token should not be empty");
        // CSRF Token 应该是 UUID 格式（不含逗号后的部分）
        assert!(token.contains("-"), "CSRF token should be UUID format");
        println!("CSRF Token: {}", token);
    }

    #[tokio::test]
    async fn test_get_rsa_public_key() {
        let client = ScauClient::new();
        let key = get_rsa_public_key(&client).await;
        assert!(key.is_ok(), "Should get RSA public key");
        let key = key.unwrap();
        assert!(!key.modulus.is_empty(), "Modulus should not be empty");
        assert!(!key.exponent.is_empty(), "Exponent should not be empty");
        // 正方教务系统返回的是 Hex 格式（包含大小写字母和数字）
        println!("Modulus (first 20 chars): {}", &key.modulus[..20.min(key.modulus.len())]);
        println!("Exponent: {}", key.exponent);
    }
}
