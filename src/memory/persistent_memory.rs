use crate::core::communication::CommManager;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: i64,
    pub category: String,
    pub key: String,
    pub value: String,
    pub timestamp: String,
    pub importance: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skill {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub content: String,
    pub usage_count: i32,
    pub last_used: String,
    pub effectiveness: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub timestamp: String,
}

#[derive(Clone)]
pub struct PersistentMemory {
    skills_dir: String,
    memory_dir: String,
    profile_dir: String,
    comm_manager: Arc<CommManager>,
}

impl PersistentMemory {
    pub fn new(base_path: &str, comm_manager: Arc<CommManager>) -> Result<Self, Box<dyn std::error::Error>> {
        let skills_dir = format!("{}/skills", base_path);
        let memory_dir = format!("{}/memory", base_path);
        let profile_dir = format!("{}/profile", base_path);
        
        // 创建必要的目录
        fs::create_dir_all(&skills_dir)?;
        fs::create_dir_all(&memory_dir)?;
        fs::create_dir_all(&profile_dir)?;
        
        let memory = Self {
            skills_dir,
            memory_dir,
            profile_dir,
            comm_manager,
        };
        
        Ok(memory)
    }
    
    pub fn add_memory(&self, category: &str, key: &str, value: &str, importance: i32) -> Result<MemoryItem, Box<dyn std::error::Error>> {
        let timestamp = Self::get_current_timestamp();
        let id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;
        
        let memory_item = MemoryItem {
            id,
            category: category.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            timestamp,
            importance,
        };
        
        // 保存到文件
        let category_dir = format!("{}/{}", self.memory_dir, category);
        fs::create_dir_all(&category_dir)?;
        
        let file_path = format!("{}/{}.json", category_dir, key);
        let content = serde_json::to_string_pretty(&memory_item)?;
        fs::write(file_path, content)?;
        
        Ok(memory_item)
    }
    
    pub fn get_memory(&self, category: &str, key: &str) -> Result<Option<MemoryItem>, Box<dyn std::error::Error>> {
        let file_path = format!("{}/{}/{}.json", self.memory_dir, category, key);
        
        if Path::new(&file_path).exists() {
            let content = fs::read_to_string(file_path)?;
            let memory_item: MemoryItem = serde_json::from_str(&content)?;
            Ok(Some(memory_item))
        } else {
            Ok(None)
        }
    }
    
    pub fn search_memory(&self, query: &str) -> Result<Vec<MemoryItem>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();
        
        // 遍历所有记忆文件
        if let Ok(entries) = fs::read_dir(&self.memory_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.file_type()?.is_dir() {
                        let category = entry.file_name().to_string_lossy().to_string();
                        let category_path = entry.path();
                        
                        if let Ok(category_entries) = fs::read_dir(category_path) {
                            for category_entry in category_entries {
                                if let Ok(category_entry) = category_entry {
                                    if category_entry.file_type()?.is_file() && 
                                       category_entry.file_name().to_string_lossy().ends_with(".json") {
                                        let content = fs::read_to_string(category_entry.path())?;
                                        let memory_item: MemoryItem = serde_json::from_str(&content)?;
                                        
                                        if memory_item.value.contains(query) || 
                                           memory_item.key.contains(query) || 
                                           memory_item.category.contains(query) {
                                            results.push(memory_item);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 按重要性和时间戳排序
        results.sort_by(|a, b| {
            if a.importance != b.importance {
                b.importance.cmp(&a.importance)
            } else {
                b.timestamp.cmp(&a.timestamp)
            }
        });
        
        // 限制返回数量
        results.truncate(10);
        
        Ok(results)
    }
    
    pub fn add_skill(&self, name: &str, description: &str, content: &str) -> Result<Skill, Box<dyn std::error::Error>> {
        let timestamp = Self::get_current_timestamp();
        let id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;
        
        let skill = Skill {
            id,
            name: name.to_string(),
            description: description.to_string(),
            content: content.to_string(),
            usage_count: 0,
            last_used: timestamp,
            effectiveness: 0.5,
        };
        
        // 保存到文件
        let sanitized_name = name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
        let file_path = format!("{}/{}.json", self.skills_dir, sanitized_name);
        let content = serde_json::to_string_pretty(&skill)?;
        fs::write(file_path, content)?;
        
        Ok(skill)
    }
    
    pub fn get_skill(&self, name: &str) -> Result<Option<Skill>, Box<dyn std::error::Error>> {
        let sanitized_name = name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
        let file_path = format!("{}/{}.json", self.skills_dir, sanitized_name);
        
        if Path::new(&file_path).exists() {
            let content = fs::read_to_string(file_path)?;
            let skill: Skill = serde_json::from_str(&content)?;
            Ok(Some(skill))
        } else {
            // 尝试查找所有技能文件
            if let Ok(entries) = fs::read_dir(&self.skills_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if entry.file_type()?.is_file() && 
                           entry.file_name().to_string_lossy().ends_with(".json") {
                            let content = fs::read_to_string(entry.path())?;
                            let skill: Skill = serde_json::from_str(&content)?;
                            if skill.name == name {
                                return Ok(Some(skill));
                            }
                        }
                    }
                }
            }
            Ok(None)
        }
    }
    
    pub fn update_skill_effectiveness(&self, skill_id: i64, effectiveness: f32) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = Self::get_current_timestamp();
        
        // 查找并更新技能文件
        if let Ok(entries) = fs::read_dir(&self.skills_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.file_type()?.is_file() && 
                       entry.file_name().to_string_lossy().ends_with(".json") {
                        let file_path = entry.path();
                        let content = fs::read_to_string(&file_path)?;
                        let mut skill: Skill = serde_json::from_str(&content)?;
                        
                        if skill.id == skill_id {
                            skill.effectiveness = effectiveness;
                            skill.last_used = timestamp;
                            skill.usage_count += 1;
                            
                            let updated_content = serde_json::to_string_pretty(&skill)?;
                            fs::write(file_path, updated_content)?;
                            return Ok(());
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn get_all_skills(&self) -> Result<Vec<Skill>, Box<dyn std::error::Error>> {
        let mut skills = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.skills_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.file_type()?.is_file() && 
                       entry.file_name().to_string_lossy().ends_with(".json") {
                        let content = fs::read_to_string(entry.path())?;
                        let skill: Skill = serde_json::from_str(&content)?;
                        skills.push(skill);
                    }
                }
            }
        }
        
        // 按使用次数和效果排序
        skills.sort_by(|a, b| {
            if a.usage_count != b.usage_count {
                b.usage_count.cmp(&a.usage_count)
            } else {
                b.effectiveness.partial_cmp(&a.effectiveness).unwrap_or(std::cmp::Ordering::Equal)
            }
        });
        
        Ok(skills)
    }
    
    pub fn set_user_profile(&self, key: &str, value: &str) -> Result<UserProfile, Box<dyn std::error::Error>> {
        let timestamp = Self::get_current_timestamp();
        let id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as i64;
        
        let user_profile = UserProfile {
            id,
            key: key.to_string(),
            value: value.to_string(),
            timestamp,
        };
        
        // 保存到文件
        let sanitized_key = key.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
        let file_path = format!("{}/{}.json", self.profile_dir, sanitized_key);
        let content = serde_json::to_string_pretty(&user_profile)?;
        fs::write(file_path, content)?;
        
        Ok(user_profile)
    }
    
    pub fn get_user_profile(&self, key: &str) -> Result<Option<UserProfile>, Box<dyn std::error::Error>> {
        let sanitized_key = key.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
        let file_path = format!("{}/{}.json", self.profile_dir, sanitized_key);
        
        if Path::new(&file_path).exists() {
            let content = fs::read_to_string(file_path)?;
            let user_profile: UserProfile = serde_json::from_str(&content)?;
            Ok(Some(user_profile))
        } else {
            Ok(None)
        }
    }
    
    fn get_current_timestamp() -> String {
        let now = SystemTime::now();
        let datetime: chrono::DateTime<chrono::Utc> = now.into();
        datetime.to_rfc3339()
    }
}