use crate::config::Config;
use crate::core::communication::CommManager;
use log::{info, warn};
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::Arc;

pub struct BrowserModule {
    config: BrowserConfig,
    comm_manager: Arc<CommManager>,
    client: Client,
    shutdown: bool,
}

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub user_agent: String,
    pub timeout: u32,
}

impl From<&Config> for BrowserConfig {
    fn from(config: &Config) -> Self {
        Self {
            user_agent: config.browser.user_agent.clone(),
            timeout: config.browser.timeout,
        }
    }
}

impl BrowserModule {
    pub async fn new(config: &Config, comm_manager: &CommManager) -> Self {
        info!("Initializing Browser Module...");
        
        let client = Client::builder()
            .user_agent(&config.browser.user_agent)
            .timeout(std::time::Duration::from_secs(config.browser.timeout as u64))
            .build()
            .unwrap();
        
        Self {
            config: BrowserConfig::from(config),
            comm_manager: Arc::new(comm_manager.clone()),
            client,
            shutdown: false,
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Browser Module...");
        
        let config = self.config.clone();
        let comm_manager = self.comm_manager.clone();
        let client = self.client.clone();
        
        // 启动浏览器处理任务
        tokio::spawn(async move {
            // 监听通信频道，处理浏览器请求
            let mut rx = comm_manager.subscribe();
            
            while let Ok(message) = rx.recv().await {
                if message.starts_with("BROWSER_SEARCH:") {
                    let query = message.replace("BROWSER_SEARCH:", "");
                    info!("Searching for: {}", query);
                    
                    // 执行搜索
                    let results = search(&client, &config, query).await;
                    match results {
                        Ok(content) => {
                            info!("Search results obtained");
                            // 发送搜索结果
                            comm_manager.send_command(format!("BROWSER_RESULT:{}", content)).await;
                        }
                        Err(e) => {
                            warn!("Browser search error: {:?}", e);
                        }
                    }
                } else if message.starts_with("BROWSER_FETCH:") {
                    let url = message.replace("BROWSER_FETCH:", "");
                    info!("Fetching URL: {}", url);
                    
                    // 获取URL内容
                    let content = fetch_url(&client, &config, url).await;
                    match content {
                        Ok(content) => {
                            info!("URL content fetched");
                            // 发送URL内容
                            comm_manager.send_command(format!("BROWSER_CONTENT:{}", content)).await;
                        }
                        Err(e) => {
                            warn!("URL fetch error: {:?}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping Browser Module...");
        self.shutdown = true;
    }
}

async fn search(client: &Client, config: &BrowserConfig, query: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Performing search for: {}", query);
    
    // 构建搜索URL
    let search_url = format!("https://www.google.com/search?q={}", urlencoding::encode(&query));
    
    // 发送请求
    let response = client.get(&search_url)
        .header("User-Agent", &config.user_agent)
        .send()
        .await?;
    
    // 获取响应内容
    let content = response.text().await?;
    
    // 解析HTML
    let document = Html::parse_document(&content);
    
    // 提取搜索结果
    let selector = Selector::parse(".g").unwrap();
    let mut results = Vec::new();
    
    for element in document.select(&selector) {
        // 提取标题
        let title_selector = Selector::parse("h3").unwrap();
        let title = element.select(&title_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();
        
        // 提取链接
        let link_selector = Selector::parse("a").unwrap();
        let link = element.select(&link_selector)
            .next()
            .and_then(|e| e.value().attr("href"))
            .unwrap_or_default();
        
        // 提取摘要
        let snippet_selector = Selector::parse(".VwiC3b").unwrap();
        let snippet = element.select(&snippet_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();
        
        if !title.is_empty() && !link.is_empty() {
            results.push(format!("标题: {}\n链接: {}\n摘要: {}\n", title, link, snippet));
        }
    }
    
    // 限制结果数量
    let limited_results = results.into_iter().take(5).collect::<Vec<_>>().join("\n");
    
    Ok(limited_results)
}

async fn fetch_url(client: &Client, config: &BrowserConfig, url: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    info!("Fetching URL: {}", url);
    
    // 发送请求
    let response = client.get(&url)
        .header("User-Agent", &config.user_agent)
        .send()
        .await?;
    
    // 获取响应内容
    let content = response.text().await?;
    
    // 解析HTML并提取主要内容
    let document = Html::parse_document(&content);
    
    // 提取标题
    let title_selector = Selector::parse("title").unwrap();
    let title = document.select(&title_selector)
        .next()
        .map(|e| e.text().collect::<String>())
        .unwrap_or_default();
    
    // 提取正文内容
    let content_selector = Selector::parse("p").unwrap();
    let mut paragraphs = Vec::new();
    
    for element in document.select(&content_selector) {
        let text = element.text().collect::<String>();
        if !text.trim().is_empty() {
            paragraphs.push(text);
        }
    }
    
    // 限制内容长度
    let content_summary = paragraphs.join("\n").chars().take(1000).collect::<String>();
    
    let result = format!("标题: {}\n\n内容: {}\n", title, content_summary);
    
    Ok(result)
}
