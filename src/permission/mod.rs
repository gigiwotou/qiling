use crate::config::Config;
use crate::core::communication::CommManager;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::Path;

pub struct PermissionModule {
    config: PermissionConfig,
    comm_manager: Arc<CommManager>,
    shutdown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    pub assistant_base_dir: String,
    pub user_protected_dirs: Vec<String>,
    pub allowed_tools: Vec<String>,
}

impl From<&Config> for PermissionConfig {
    fn from(config: &Config) -> Self {
        Self {
            assistant_base_dir: config.permission.assistant_base_dir.clone(),
            user_protected_dirs: config.permission.user_protected_dirs.clone(),
            allowed_tools: config.permission.allowed_tools.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionLevel {
    None,
    Read,
    Write,
    Execute,
}

#[derive(Debug, Clone)]
pub enum ResourceType {
    File(String),
    Directory(String),
    Tool(String),
    SystemCommand(String),
}

impl PermissionModule {
    pub async fn new(config: &Config, comm_manager: &CommManager) -> Self {
        info!("Initializing Permission Module...");
        
        // 确保助手基础目录存在
        let perm_config = PermissionConfig::from(config);
        let assistant_dir = Path::new(&perm_config.assistant_base_dir);
        if !assistant_dir.exists() {
            std::fs::create_dir_all(assistant_dir).unwrap();
            info!("Created assistant base directory: {:?}", assistant_dir);
        }
        
        Self {
            config: perm_config,
            comm_manager: Arc::new(comm_manager.clone()),
            shutdown: false,
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Permission Module...");
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping Permission Module...");
        self.shutdown = true;
    }
    
    pub fn check_permission(&self, resource: &ResourceType, requested_level: PermissionLevel) -> bool {
        info!("Checking permission for {:?} with level {:?}", resource, requested_level);
        
        match resource {
            ResourceType::File(path) => self.check_file_permission(path, requested_level),
            ResourceType::Directory(path) => self.check_directory_permission(path, requested_level),
            ResourceType::Tool(tool_name) => self.check_tool_permission(tool_name, requested_level),
            ResourceType::SystemCommand(command) => self.check_system_command_permission(command, requested_level),
        }
    }
    
    fn check_file_permission(&self, path: &str, requested_level: PermissionLevel) -> bool {
        let path = Path::new(path);
        
        // 检查是否是助手的文件
        if self.is_assistant_file(path) {
            info!("File {} is in assistant directory, granting permission", path.display());
            return true;
        }
        
        // 检查是否是用户保护的目录
        if self.is_protected_path(path) {
            warn!("File {} is in protected directory, denying permission", path.display());
            return false;
        }
        
        // 根据请求的权限级别检查
        match requested_level {
            PermissionLevel::None => true,
            PermissionLevel::Read => {
                warn!("Reading user files is not allowed: {}", path.display());
                false
            }
            PermissionLevel::Write => {
                warn!("Writing user files is not allowed: {}", path.display());
                false
            }
            PermissionLevel::Execute => {
                warn!("Executing user files is not allowed: {}", path.display());
                false
            }
        }
    }
    
    fn check_directory_permission(&self, path: &str, requested_level: PermissionLevel) -> bool {
        let path = Path::new(path);
        
        // 检查是否是助手的目录
        if self.is_assistant_directory(path) {
            info!("Directory {} is in assistant directory, granting permission", path.display());
            return true;
        }
        
        // 检查是否是用户保护的目录
        if self.is_protected_path(path) {
            warn!("Directory {} is in protected directory, denying permission", path.display());
            return false;
        }
        
        // 根据请求的权限级别检查
        match requested_level {
            PermissionLevel::None => true,
            PermissionLevel::Read => {
                warn!("Reading user directories is not allowed: {}", path.display());
                false
            }
            PermissionLevel::Write => {
                warn!("Writing user directories is not allowed: {}", path.display());
                false
            }
            PermissionLevel::Execute => {
                warn!("Executing user directories is not allowed: {}", path.display());
                false
            }
        }
    }
    
    fn check_tool_permission(&self, tool_name: &str, requested_level: PermissionLevel) -> bool {
        // 检查工具是否在允许列表中
        if self.config.allowed_tools.contains(&tool_name.to_string()) {
            info!("Tool {} is allowed, granting permission", tool_name);
            return true;
        }
        
        warn!("Tool {} is not allowed, denying permission", tool_name);
        false
    }
    
    fn check_system_command_permission(&self, command: &str, requested_level: PermissionLevel) -> bool {
        // 系统命令需要特别小心，只允许安全的命令
        let safe_commands = vec!["ls", "pwd", "date", "echo"];
        
        let command_parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(cmd) = command_parts.first() {
            if safe_commands.contains(cmd) {
                info!("System command {} is allowed, granting permission", cmd);
                return true;
            }
        }
        
        warn!("System command {} is not allowed, denying permission", command);
        false
    }
    
    fn is_assistant_file(&self, path: &Path) -> bool {
        let assistant_dir = Path::new(&self.config.assistant_base_dir);
        path.starts_with(assistant_dir)
    }
    
    fn is_assistant_directory(&self, path: &Path) -> bool {
        let assistant_dir = Path::new(&self.config.assistant_base_dir);
        path.starts_with(assistant_dir)
    }
    
    fn is_protected_path(&self, path: &Path) -> bool {
        for protected_dir in &self.config.user_protected_dirs {
            let protected_path = Path::new(protected_dir);
            if path.starts_with(protected_path) {
                return true;
            }
        }
        false
    }
    
    pub async fn request_user_permission(&self, action: String, resource: String) -> bool {
        info!("Requesting user permission for {} on {}", action, resource);
        
        // 发送权限请求通知
        self.comm_manager.send_command(format!("PERMISSION_REQUEST:{},{}", action, resource)).await;
        
        // 这里应该等待用户的响应
        // 暂时返回false，需要实现用户交互逻辑
        false
    }
}