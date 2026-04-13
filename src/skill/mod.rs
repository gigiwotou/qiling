use crate::config::Config;
use crate::core::communication::CommManager;
use crate::memory::MemoryModule;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub created_at: String,
    pub last_used: String,
    pub usage_count: u32,
    pub effectiveness: f32,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillExecution {
    pub skill_id: String,
    pub input: String,
    pub output: String,
    pub success: bool,
    pub timestamp: String,
    pub feedback: Option<String>,
}

pub struct SkillModule {
    config: SkillConfig,
    comm_manager: Arc<CommManager>,
    memory: Arc<MemoryModule>,
    skills: Vec<Skill>,
}

#[derive(Debug, Clone)]
pub struct SkillConfig {
    pub enabled: bool,
    pub auto_create: bool,
    pub max_skills: u32,
    pub min_effectiveness: f32,
}

impl From<&Config> for SkillConfig {
    fn from(config: &Config) -> Self {
        Self {
            enabled: true,
            auto_create: true,
            max_skills: 100,
            min_effectiveness: 0.5,
        }
    }
}

impl SkillModule {
    pub async fn new(config: &Config, comm_manager: &CommManager, memory: &MemoryModule) -> Self {
        info!("Initializing Skill Module...");
        
        let mut skill_module = Self {
            config: SkillConfig::from(config),
            comm_manager: Arc::new(comm_manager.clone()),
            memory: Arc::new(memory.clone()),
            skills: Vec::new(),
        };
        
        // 加载现有技能
        skill_module.load_skills().await;
        
        skill_module
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Skill Module...");
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping Skill Module...");
    }
    
    async fn load_skills(&mut self) {
        // 从持久记忆中加载技能
        if let Ok(skills) = self.memory.get_all_skills() {
            for skill in skills {
                let last_used = skill.last_used.clone();
                self.skills.push(Skill {
                    id: skill.id.to_string(),
                    name: skill.name,
                    description: skill.description,
                    content: skill.content,
                    created_at: last_used.clone(),
                    last_used: last_used,
                    usage_count: skill.usage_count as u32,
                    effectiveness: skill.effectiveness,
                    tags: Vec::new(),
                });
            }
            info!("Loaded {} skills from memory", self.skills.len());
        }
    }
    
    pub async fn create_skill(&mut self, name: &str, description: &str, content: &str, tags: Vec<String>) -> Result<Skill, Box<dyn std::error::Error>> {
        info!("Creating new skill: {}", name);
        
        let skill = Skill {
            id: format!("skill_{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs()),
            name: name.to_string(),
            description: description.to_string(),
            content: content.to_string(),
            created_at: Self::get_current_timestamp(),
            last_used: Self::get_current_timestamp(),
            usage_count: 0,
            effectiveness: 0.5,
            tags,
        };
        
        // 保存到持久记忆
        self.memory.add_skill(&skill.name, &skill.description, &skill.content)?;
        
        // 添加到技能列表
        self.skills.push(skill.clone());
        
        info!("Skill created: {}", skill.name);
        Ok(skill)
    }
    
    pub async fn execute_skill(&mut self, skill_id: &str, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("Executing skill: {}", skill_id);
        
        // 查找技能
        if let Some(skill) = self.skills.iter_mut().find(|s| s.id == skill_id) {
            // 执行技能逻辑（这里是示例，实际需要根据技能内容执行）
            let output = format!("Executed skill '{}' with input: {}", skill.name, input);
            
            // 更新技能使用统计
            skill.usage_count += 1;
            skill.last_used = Self::get_current_timestamp();
            
            // 暂时设置效果为0.8（实际应该根据执行结果评估）
            skill.effectiveness = 0.8;
            
            // 更新持久记忆中的技能
            if let Ok(Some(persistent_skill)) = self.memory.get_skill(&skill.name) {
                self.memory.update_skill_effectiveness(persistent_skill.id, skill.effectiveness)?;
            }
            
            info!("Skill executed: {}, Usage count: {}", skill.name, skill.usage_count);
            Ok(output)
        } else {
            Err(format!("Skill not found: {}", skill_id).into())
        }
    }
    
    pub async fn get_skill(&self, skill_id: &str) -> Option<Skill> {
        self.skills.iter().find(|s| s.id == skill_id).cloned()
    }
    
    pub async fn get_all_skills(&self) -> Vec<Skill> {
        self.skills.clone()
    }
    
    pub async fn search_skills(&self, query: &str) -> Vec<Skill> {
        self.skills.iter()
            .filter(|skill| 
                skill.name.contains(query) || 
                skill.description.contains(query) ||
                skill.tags.iter().any(|tag| tag.contains(query))
            )
            .cloned()
            .collect()
    }
    
    pub async fn improve_skill(&mut self, skill_id: &str, feedback: &str) -> Result<Skill, Box<dyn std::error::Error>> {
        info!("Improving skill: {}", skill_id);
        
        if let Some(skill) = self.skills.iter_mut().find(|s| s.id == skill_id) {
            // 这里可以根据反馈改进技能内容
            // 示例：简单地在内容末尾添加反馈
            skill.content += &format!("\n\n// Feedback: {}", feedback);
            
            // 更新效果评估
            skill.effectiveness = 0.9;
            skill.last_used = Self::get_current_timestamp();
            
            // 更新持久记忆中的技能
            if let Ok(Some(persistent_skill)) = self.memory.get_skill(&skill.name) {
                self.memory.update_skill_effectiveness(persistent_skill.id, skill.effectiveness)?;
            }
            
            info!("Skill improved: {}", skill.name);
            Ok(skill.clone())
        } else {
            Err(format!("Skill not found: {}", skill_id).into())
        }
    }
    
    pub async fn auto_create_skill(&mut self, task: &str, solution: &str) -> Result<Skill, Box<dyn std::error::Error>> {
        if !self.config.auto_create {
            return Err("Auto skill creation is disabled".into());
        }
        
        info!("Auto-creating skill for task: {}", task);
        
        // 生成技能名称和描述
        let skill_name = format!("task_{}", task.split_whitespace().take(3).collect::<Vec<_>>().join("_"));
        let description = format!("Automatically created skill for task: {}", task);
        
        // 创建技能内容
        let content = format!("// Task: {}\n// Solution: {}", task, solution);
        
        // 提取标签
        let tags: Vec<String> = task.split_whitespace().take(5).map(|word| word.to_string()).collect();
        
        // 创建技能
        self.create_skill(&skill_name, &description, &content, tags).await
    }
    
    fn get_current_timestamp() -> String {
        let now = SystemTime::now();
        let datetime: chrono::DateTime<chrono::Utc> = now.into();
        datetime.to_rfc3339()
    }
}
