use reqwest::{Client, ClientBuilder};
use std::time::Duration;

/// 华南农业大学教务系统 API 客户端
#[derive(Clone)]
pub struct ScauClient {
    client: Client,
    base_url: String,
}

impl ScauClient {
    /// 创建新的客户端
    pub fn new() -> Self {
        let client = ClientBuilder::new()
            .cookie_store(true)
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: "https://jwzf.scau.edu.cn".to_string(),
        }
    }

    /// 获取基础 URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// 获取 HTTP 客户端引用
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// 获取可变的 HTTP 客户端引用
    pub fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }

    /// 构建完整 URL
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

impl Default for ScauClient {
    fn default() -> Self {
        Self::new()
    }
}
