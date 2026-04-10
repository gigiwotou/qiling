use crate::config::Config;
use crate::core::communication::CommManager;
use crate::memory::MemoryModule;
use log::{info, warn};
use tokio::time;
use chrono::Timelike;
use std::sync::Arc;
use std::time::Duration;

pub struct DreamModule {
    config: DreamConfig,
    comm_manager: Arc<CommManager>,
    memory_module: Arc<MemoryModule>,
    shutdown: bool,
    last_activity: time::Instant,
}

#[derive(Debug, Clone)]
pub struct DreamConfig {
    pub idle_threshold_seconds: u64,
    pub dream_interval_seconds: u64,
    pub max_dream_duration_seconds: u64,
}

impl From<&Config> for DreamConfig {
    fn from(config: &Config) -> Self {
        Self {
            idle_threshold_seconds: config.dream.idle_threshold_seconds,
            dream_interval_seconds: config.dream.dream_interval_seconds,
            max_dream_duration_seconds: config.dream.max_dream_duration_seconds,
        }
    }
}

impl DreamModule {
    pub async fn new(config: &Config, comm_manager: &CommManager, memory_module: Arc<MemoryModule>) -> Self {
        info!("Initializing Dream Module...");
        
        Self {
            config: DreamConfig::from(config),
            comm_manager: Arc::new(comm_manager.clone()),
            memory_module,
            shutdown: false,
            last_activity: time::Instant::now(),
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Dream Module...");
        
        let config = self.config.clone();
        let comm_manager = self.comm_manager.clone();
        let memory_module = self.memory_module.clone();
        let mut last_activity = self.last_activity;
        
        // 启动做梦任务
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60)); // 每分钟检查一次
            
            while let _ = interval.tick().await {
                // 检查是否有活动
                let now = time::Instant::now();
                let idle_duration = now.duration_since(last_activity);
                
                // 如果空闲时间超过阈值，开始做梦
                if idle_duration >= Duration::from_secs(config.idle_threshold_seconds) {
                    info!("System is idle, starting dream cycle...");
                    
                    // 执行做梦逻辑
                    if let Err(e) = dream(&memory_module, &comm_manager).await {
                        warn!("Dream cycle failed: {:?}", e);
                    }
                    
                    // 重置最后活动时间
                    last_activity = time::Instant::now();
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping Dream Module...");
        self.shutdown = true;
    }
    
    pub fn update_activity(&mut self) {
        self.last_activity = time::Instant::now();
    }
}

async fn dream(memory_module: &MemoryModule, comm_manager: &CommManager) -> Result<(), Box<dyn std::error::Error>> {
    info!("Entering dream state...");
    
    // 1. 分析最近的交互历史
    let recent_interactions = memory_module.get_recent_interactions(50).await?;
    info!("Analyzing {} recent interactions...", recent_interactions.len());
    
    // 2. 提取用户的工作需求和模式
    let work_patterns = analyze_work_patterns(&recent_interactions);
    info!("Identified work patterns: {:?}", work_patterns);
    
    // 3. 生成自我提高建议
    let improvement_suggestions = generate_improvement_suggestions(&work_patterns);
    info!("Generated improvement suggestions: {:?}", improvement_suggestions);
    
    // 4. 存储分析结果到长期记忆
    for (key, value) in work_patterns {
        memory_module.store_memory("work_pattern".to_string(), key, value).await?;
    }
    
    for (key, value) in improvement_suggestions {
        memory_module.store_memory("improvement".to_string(), key, value).await?;
    }
    
    // 5. 发送做梦完成的通知
    comm_manager.send_command("DREAM_COMPLETED".to_string()).await;
    
    info!("Dream cycle completed");
    Ok(())
}

fn analyze_work_patterns(interactions: &[crate::memory::InteractionHistory]) -> std::collections::HashMap<String, String> {
    let mut patterns = std::collections::HashMap::new();
    
    // 分析用户的查询类型
    let mut query_types = std::collections::HashMap::new();
    for interaction in interactions {
        let input = &interaction.user_input;
        
        // 简单的查询类型分类
        if input.contains("天气") {
            *query_types.entry("weather").or_insert(0) += 1;
        } else if input.contains("搜索") || input.contains("查一下") {
            *query_types.entry("search").or_insert(0) += 1;
        } else if input.contains("提醒") || input.contains("任务") {
            *query_types.entry("task").or_insert(0) += 1;
        } else if input.contains("系统") || input.contains("状态") {
            *query_types.entry("system").or_insert(0) += 1;
        } else {
            *query_types.entry("other").or_insert(0) += 1;
        }
    }
    
    // 存储查询类型统计
    patterns.insert("query_types".to_string(), format!("{:?}", query_types));
    
    // 分析用户的工作时间模式
    let mut time_patterns = std::collections::HashMap::new();
    for interaction in interactions {
        let timestamp = &interaction.timestamp;
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
            let hour = dt.hour();
            *time_patterns.entry(hour).or_insert(0) += 1;
        }
    }
    
    // 存储时间模式
    patterns.insert("time_patterns".to_string(), format!("{:?}", time_patterns));
    
    patterns
}

fn generate_improvement_suggestions(work_patterns: &std::collections::HashMap<String, String>) -> std::collections::HashMap<String, String> {
    let mut suggestions = std::collections::HashMap::new();
    
    // 根据工作模式生成建议
    if let Some(query_types) = work_patterns.get("query_types") {
        if query_types.contains("weather") {
            suggestions.insert("weather_improvement".to_string(), "可以考虑添加每日天气自动提醒功能，让你提前了解当天天气情况".to_string());
        }
        if query_types.contains("search") {
            suggestions.insert("search_improvement".to_string(), "可以优化搜索结果的展示方式，提供更精准的信息提取".to_string());
        }
        if query_types.contains("task") {
            suggestions.insert("task_improvement".to_string(), "可以添加任务优先级管理和智能提醒功能，提高工作效率".to_string());
        }
    }
    
    // 通用建议
    suggestions.insert("general_improvement".to_string(), "建议定期整理工作资料，建立个人知识库，提高信息检索效率".to_string());
    suggestions.insert("learning_improvement".to_string(), "可以学习一些 productivity 工具和快捷键，进一步提高工作效率".to_string());
    
    suggestions
}
