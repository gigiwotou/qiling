use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub voice: VoiceConfig,
    pub ai: AIConfig,
    pub browser: BrowserConfig,
    pub task: TaskConfig,
    pub tool: ToolConfig,
    pub memory: MemoryConfig,
    pub dream: DreamConfig,
    pub permission: PermissionConfig,
    pub harness: HarnessConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub low_memory_mode: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VoiceConfig {
    pub wake_word: String,
    pub sensitivity: f32,
    pub use_offline: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AIConfig {
    pub default_provider: String,
    pub api_keys: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BrowserConfig {
    pub user_agent: String,
    pub timeout: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TaskConfig {
    pub reminder_interval: u32,
    pub max_tasks: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolConfig {
    pub enabled_tools: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MemoryConfig {
    pub db_path: String,
    pub max_memory_items: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DreamConfig {
    pub idle_threshold_seconds: u64,
    pub dream_interval_seconds: u64,
    pub max_dream_duration_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PermissionConfig {
    pub assistant_base_dir: String,
    pub user_protected_dirs: Vec<String>,
    pub allowed_tools: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HarnessConfig {
    pub max_workflows: u32,
    pub workflow_dir: String,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Path::new("config.toml");
        
        if !config_path.exists() {
            // 创建默认配置
            let default_config = Self::default();
            let config_str = toml::to_string(&default_config)?;
            fs::write(config_path, config_str)?;
            Ok(default_config)
        } else {
            let config_str = fs::read_to_string(config_path)?;
            let config = toml::from_str(&config_str)?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = toml::to_string(self)?;
        fs::write("config.toml", config_str)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                port: 8080,
                host: "127.0.0.1".to_string(),
                low_memory_mode: true,
            },
            voice: VoiceConfig {
                wake_word: "嘿，助手".to_string(),
                sensitivity: 0.8,
                use_offline: true,
            },
            ai: AIConfig {
                default_provider: "openai".to_string(),
                api_keys: std::collections::HashMap::new(),
            },
            browser: BrowserConfig {
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
                timeout: 30,
            },
            task: TaskConfig {
                reminder_interval: 5,
                max_tasks: 100,
            },
            tool: ToolConfig {
                enabled_tools: vec!["system".to_string(), "calculator".to_string(), "notepad".to_string()],
            },
            memory: MemoryConfig {
                db_path: "memory.db".to_string(),
                max_memory_items: 1000,
            },
            dream: DreamConfig {
                idle_threshold_seconds: 300,
                dream_interval_seconds: 600,
                max_dream_duration_seconds: 120,
            },
            permission: PermissionConfig {
                assistant_base_dir: "./assistant_data".to_string(),
                user_protected_dirs: vec!["./".to_string(), "~/Documents".to_string(), "~/Desktop".to_string(), "~/Downloads".to_string()],
                allowed_tools: vec!["system".to_string(), "calculator".to_string(), "notepad".to_string()],
            },
            harness: HarnessConfig {
                max_workflows: 50,
                workflow_dir: "./workflows".to_string(),
            },
        }
    }
}
