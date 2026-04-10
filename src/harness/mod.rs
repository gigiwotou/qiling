use crate::config::Config;
use crate::core::communication::CommManager;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc, Timelike};

pub struct HarnessModule {
    config: HarnessConfig,
    comm_manager: Arc<CommManager>,
    workflows: Vec<Workflow>,
    shutdown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessConfig {
    pub max_workflows: u32,
    pub workflow_dir: String,
}

impl From<&Config> for HarnessConfig {
    fn from(config: &Config) -> Self {
        Self {
            max_workflows: config.harness.max_workflows,
            workflow_dir: config.harness.workflow_dir.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub triggers: Vec<WorkflowTrigger>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub action: WorkflowAction,
    pub parameters: std::collections::HashMap<String, String>,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowAction {
    ToolCall {
        tool_name: String,
    },
    BrowserAction {
        action: String, // navigate, search, extract
        url: Option<String>,
    },
    TaskAction {
        action: String, // create, update, complete
        task_id: Option<i64>,
    },
    MemoryAction {
        action: String, // store, retrieve, search
        category: String,
        key: String,
    },
    Conditional {
        condition: String,
        true_steps: Vec<String>, // step IDs
        false_steps: Vec<String>, // step IDs
    },
    Wait {
        seconds: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowTrigger {
    TimeBased {
        cron_expression: String,
    },
    EventBased {
        event_type: String, // voice_input, system_event, user_action
        pattern: Option<String>,
    },
    Manual {
        command: String,
    },
}

impl HarnessModule {
    pub async fn new(config: &Config, comm_manager: &CommManager) -> Self {
        info!("Initializing Harness Module...");
        
        // 确保工作流目录存在
        let harness_config = HarnessConfig::from(config);
        let workflow_dir = std::path::Path::new(&harness_config.workflow_dir);
        if !workflow_dir.exists() {
            std::fs::create_dir_all(workflow_dir).unwrap();
            info!("Created workflow directory: {:?}", workflow_dir);
        }
        
        // 加载工作流
        let workflows = Self::load_workflows(&harness_config.workflow_dir);
        info!("Loaded {} workflows", workflows.len());
        
        Self {
            config: harness_config,
            comm_manager: Arc::new(comm_manager.clone()),
            workflows,
            shutdown: false,
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Harness Module...");
        
        // 启动工作流监控任务
        let config = self.config.clone();
        let comm_manager = self.comm_manager.clone();
        let mut workflows = self.workflows.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            
            while let _ = interval.tick().await {
                // 检查时间触发的工作流
                Self::check_time_triggers(&mut workflows, &comm_manager).await;
            }
        });
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping Harness Module...");
        self.shutdown = true;
    }
    
    pub async fn create_workflow(&mut self, workflow: Workflow) -> Result<Workflow, Box<dyn std::error::Error>> {
        info!("Creating workflow: {}", workflow.name);
        
        if self.workflows.len() >= self.config.max_workflows as usize {
            return Err("Maximum number of workflows reached".into());
        }
        
        // 保存工作流
        self.save_workflow(&workflow)?;
        
        // 添加到内存
        self.workflows.push(workflow.clone());
        
        Ok(workflow)
    }
    
    pub async fn execute_workflow(&self, workflow_id: String) -> Result<(), Box<dyn std::error::Error>> {
        info!("Executing workflow: {}", workflow_id);
        
        // 查找工作流
        let workflow = self.workflows.iter().find(|w| w.id == workflow_id);
        if let Some(workflow) = workflow {
            // 执行工作流
            self.execute_workflow_steps(workflow).await?;
        } else {
            return Err(format!("Workflow not found: {}", workflow_id).into());
        }
        
        Ok(())
    }
    
    pub async fn list_workflows(&self) -> Vec<Workflow> {
        self.workflows.clone()
    }
    
    async fn execute_workflow_steps(&self, workflow: &Workflow) -> Result<(), Box<dyn std::error::Error>> {
        info!("Executing workflow steps for: {}", workflow.name);
        
        for step in &workflow.steps {
            info!("Executing step: {}", step.name);
            
            // 执行步骤
            match &step.action {
                WorkflowAction::ToolCall { tool_name } => {
                    // 调用工具
                    let command = format!("TOOL_EXECUTE:{}", tool_name);
                    self.comm_manager.send_command(command).await;
                }
                WorkflowAction::BrowserAction { action, url } => {
                    // 浏览器操作
                    if let Some(url) = url {
                        let command = format!("BROWSER_FETCH:{}", url);
                        self.comm_manager.send_command(command).await;
                    }
                }
                WorkflowAction::TaskAction { action, task_id } => {
                    // 任务操作
                    // 实现任务操作逻辑
                }
                WorkflowAction::MemoryAction { action, category, key } => {
                    // 记忆操作
                    // 实现记忆操作逻辑
                }
                WorkflowAction::Conditional { condition, true_steps, false_steps } => {
                    // 条件判断
                    // 实现条件判断逻辑
                }
                WorkflowAction::Wait { seconds } => {
                    // 等待
                    tokio::time::sleep(tokio::time::Duration::from_secs(*seconds as u64)).await;
                }
            }
            
            // 检查超时
            // 实现超时检查逻辑
        }
        
        Ok(())
    }
    
    fn load_workflows(workflow_dir: &str) -> Vec<Workflow> {
        let mut workflows = Vec::new();
        let dir = std::path::Path::new(workflow_dir);
        
        if dir.exists() && dir.is_dir() {
            for entry in std::fs::read_dir(dir).unwrap() {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map(|ext| ext == "json").unwrap_or(false) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(workflow) = serde_json::from_str(&content) {
                                workflows.push(workflow);
                            }
                        }
                    }
                }
            }
        }
        
        workflows
    }
    
    fn save_workflow(&self, workflow: &Workflow) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = std::path::Path::new(&self.config.workflow_dir)
            .join(format!("{}.json", workflow.id));
        
        let content = serde_json::to_string_pretty(workflow)?;
        std::fs::write(file_path, content)?;
        
        Ok(())
    }
    
    async fn check_time_triggers(workflows: &mut Vec<Workflow>, comm_manager: &CommManager) {
        let now = Utc::now();
        
        for workflow in workflows {
            for trigger in &workflow.triggers {
                if let WorkflowTrigger::TimeBased { cron_expression } = trigger {
                    // 检查cron表达式是否匹配当前时间
                    // 实现cron表达式解析和匹配逻辑
                    // 这里简化处理，实际应该使用cron库
                    if now.second() == 0 { // 每分钟检查一次
                        info!("Executing time-based workflow: {}", workflow.name);
                        // 执行工作流
                        // 这里应该调用execute_workflow_steps
                    }
                }
            }
        }
    }
}
