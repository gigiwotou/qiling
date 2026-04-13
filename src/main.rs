use crate::core::service::CoreService;
use crate::config::Config;
use log::{info, error};

mod core;
mod config;
mod voice;
mod ai;
mod browser;
mod task;
mod tool;
mod utils;
mod memory;
mod dream;
mod permission;
mod harness;
mod skill;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();
    info!("Starting PC Assistant Core Service...");
    
    // 加载配置
    let config = Config::load().unwrap();
    info!("Configuration loaded successfully");
    
    // 初始化核心服务
    let mut core_service = CoreService::new(config).await;
    
    // 启动服务
    if let Err(e) = core_service.start().await {
        error!("Failed to start core service: {:?}", e);
    }
}

