use crate::config::Config;
use crate::core::communication::CommManager;
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct AIModule {
    config: AIConfig,
    comm_manager: Arc<CommManager>,
    client: Client,
    shutdown: bool,
}

#[derive(Debug, Clone)]
pub struct AIConfig {
    pub default_provider: String,
    pub api_keys: std::collections::HashMap<String, String>,
}

impl From<&Config> for AIConfig {
    fn from(config: &Config) -> Self {
        Self {
            default_provider: config.ai.default_provider.clone(),
            api_keys: config.ai.api_keys.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIRequest {
    pub prompt: String,
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIResponse {
    pub text: String,
    pub provider: String,
}

impl AIModule {
    pub async fn new(config: &Config, comm_manager: &CommManager) -> Self {
        info!("Initializing AI Module...");
        
        Self {
            config: AIConfig::from(config),
            comm_manager: Arc::new(comm_manager.clone()),
            client: Client::new(),
            shutdown: false,
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting AI Module...");
        
        let config = self.config.clone();
        let comm_manager = self.comm_manager.clone();
        let client = self.client.clone();
        
        // 启动AI处理任务
        tokio::spawn(async move {
            // 监听通信频道，处理AI请求
            let mut rx = comm_manager.subscribe();
            
            while let Ok(message) = rx.recv().await {
                if message.starts_with("VOICE_INPUT:") {
                    let input = message.replace("VOICE_INPUT:", "");
                    info!("Processing AI request for: {}", input);
                    
                    // 调用AI API
                    let response = call_ai(&config, &client, input).await;
                    match response {
                        Ok(result) => {
                            info!("AI response: {}", result.text);
                            // 发送AI响应
                            comm_manager.send_command(format!("AI_RESPONSE:{}", result.text)).await;
                        }
                        Err(e) => {
                            warn!("AI API error: {:?}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping AI Module...");
        self.shutdown = true;
    }
}

async fn call_ai(config: &AIConfig, client: &Client, prompt: String) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>> {
    info!("Calling AI API with provider: {}", config.default_provider);
    
    match config.default_provider.as_str() {
        "openai" => call_openai_api(client, config, prompt).await,
        "anthropic" => call_anthropic_api(client, config, prompt).await,
        "google" => call_google_api(client, config, prompt).await,
        _ => {
            // 模拟响应
            Ok(AIResponse {
                text: format!("这是对'{}'的模拟响应", prompt),
                provider: config.default_provider.clone(),
            })
        }
    }
}

async fn call_openai_api(client: &Client, config: &AIConfig, prompt: String) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>> {
    info!("Calling OpenAI API");
    
    let api_key = config.api_keys.get("openai").ok_or("OpenAI API key not found")?;
    
    // 构建请求
    let request = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {"role": "system", "content": "You are a helpful PC assistant."},
            {"role": "user", "content": prompt}
        ],
        "max_tokens": 1000
    });
    
    // 发送请求
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    
    // 解析响应
    let response_json: serde_json::Value = response.json().await?;
    let text = response_json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
    
    Ok(AIResponse {
        text,
        provider: "openai".to_string(),
    })
}

async fn call_anthropic_api(client: &Client, config: &AIConfig, prompt: String) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>> {
    info!("Calling Anthropic API");
    
    let api_key = config.api_keys.get("anthropic").ok_or("Anthropic API key not found")?;
    
    // 构建请求
    let request = serde_json::json!({
        "model": "claude-3-opus-20240229",
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "max_tokens": 1000
    });
    
    // 发送请求
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send()
        .await?;
    
    // 解析响应
    let response_json: serde_json::Value = response.json().await?;
    let text = response_json["content"][0]["text"].as_str().unwrap_or("").to_string();
    
    Ok(AIResponse {
        text,
        provider: "anthropic".to_string(),
    })
}

async fn call_google_api(client: &Client, config: &AIConfig, prompt: String) -> Result<AIResponse, Box<dyn std::error::Error + Send + Sync>> {
    info!("Calling Google Gemini API");
    
    let api_key = config.api_keys.get("google").ok_or("Google API key not found")?;
    
    // 构建请求
    let request = serde_json::json!({
        "contents": [
            {
                "parts": [
                    {"text": prompt}
                ]
            }
        ],
        "generationConfig": {
            "maxOutputTokens": 1000
        }
    });
    
    // 发送请求
    let response = client
        .post(format!("https://generativelanguage.googleapis.com/v1/models/gemini-pro:generateContent?key={}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    
    // 解析响应
    let response_json: serde_json::Value = response.json().await?;
    let text = response_json["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("").to_string();
    
    Ok(AIResponse {
        text,
        provider: "google".to_string(),
    })
}
