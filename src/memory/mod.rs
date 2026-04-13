pub mod persistent_memory;

use crate::config::Config;
use crate::core::communication::CommManager;
use log::{info, warn};
use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;

#[derive(Clone)]
pub struct MemoryModule {
    db_pool: SqlitePool,
    comm_manager: Arc<CommManager>,
    shutdown: bool,
    persistent_memory: persistent_memory::PersistentMemory,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: i64,
    pub category: String,
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserPreference {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct InteractionHistory {
    pub id: i64,
    pub user_input: String,
    pub assistant_response: String,
    pub timestamp: String,
}

impl MemoryModule {
    pub async fn new(config: &Config, comm_manager: &CommManager) -> Self {
        info!("Initializing Memory Module...");
        
        // 初始化数据库连接
        let db_pool = SqlitePool::connect(&format!("sqlite://{}", config.memory.db_path)).await.unwrap();
        
        // 创建记忆表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memory_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#).execute(&db_pool).await.unwrap();
        
        // 创建用户偏好表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS user_preferences (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#).execute(&db_pool).await.unwrap();
        
        // 创建交互历史表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS interaction_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_input TEXT NOT NULL,
                assistant_response TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )
        "#).execute(&db_pool).await.unwrap();
        
        // 初始化持久记忆模块
        let persistent_memory = persistent_memory::PersistentMemory::new(
            &config.memory.db_path,
            Arc::new(comm_manager.clone())
        ).unwrap();
        
        Self {
            db_pool,
            comm_manager: Arc::new(comm_manager.clone()),
            shutdown: false,
            persistent_memory,
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Memory Module...");
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping Memory Module...");
        self.shutdown = true;
    }
    
    pub async fn store_memory(&self, category: String, key: String, value: String) -> Result<MemoryItem, Box<dyn std::error::Error>> {
        info!("Storing memory: {} - {}", category, key);
        
        // 同时存储到持久记忆
        self.persistent_memory.add_memory(&category, &key, &value, 1)?;
        
        let now = Utc::now().to_rfc3339();
        
        let memory = sqlx::query_as::<_, MemoryItem>(r#"
            INSERT INTO memory_items (category, key, value, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, category, key, value, created_at, updated_at
        "#)
        .bind(category)
        .bind(key)
        .bind(value)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(memory)
    }
    
    pub async fn get_memory(&self, category: String, key: String) -> Result<Option<MemoryItem>, Box<dyn std::error::Error>> {
        info!("Getting memory: {} - {}", category, key);
        
        // 优先从持久记忆中获取
        if let Ok(Some(persistent_item)) = self.persistent_memory.get_memory(&category, &key) {
            let timestamp = persistent_item.timestamp.clone();
            return Ok(Some(MemoryItem {
                id: persistent_item.id,
                category: persistent_item.category,
                key: persistent_item.key,
                value: persistent_item.value,
                created_at: timestamp.clone(),
                updated_at: timestamp,
            }));
        }
        
        let memory = sqlx::query_as::<_, MemoryItem>(r#"
            SELECT id, category, key, value, created_at, updated_at
            FROM memory_items
            WHERE category = ? AND key = ?
            ORDER BY updated_at DESC
            LIMIT 1
        "#)
        .bind(category)
        .bind(key)
        .fetch_optional(&self.db_pool)
        .await?;
        
        Ok(memory)
    }
    
    pub async fn set_preference(&self, key: String, value: String) -> Result<UserPreference, Box<dyn std::error::Error>> {
        info!("Setting preference: {} = {}", key, value);
        
        // 同时更新持久记忆中的用户配置
        self.persistent_memory.set_user_profile(&key, &value)?;
        
        let now = Utc::now().to_rfc3339();
        
        // 尝试更新，如果不存在则插入
        let result = sqlx::query(r#"
            UPDATE user_preferences
            SET value = ?, updated_at = ?
            WHERE key = ?
        "#)
        .bind(value.clone())
        .bind(&now)
        .bind(key.clone())
        .execute(&self.db_pool)
        .await?;
        
        if result.rows_affected() == 0 {
            // 插入新记录
            let preference = sqlx::query_as::<_, UserPreference>(r#"
                INSERT INTO user_preferences (key, value, created_at, updated_at)
                VALUES (?, ?, ?, ?)
                RETURNING id, key, value, created_at, updated_at
            "#)
            .bind(key)
            .bind(value)
            .bind(&now)
            .bind(&now)
            .fetch_one(&self.db_pool)
            .await?;
            
            Ok(preference)
        } else {
            // 获取更新后的记录
            let preference = sqlx::query_as::<_, UserPreference>(r#"
                SELECT id, key, value, created_at, updated_at
                FROM user_preferences
                WHERE key = ?
            "#)
            .bind(key)
            .fetch_one(&self.db_pool)
            .await?;
            
            Ok(preference)
        }
    }
    
    pub async fn get_preference(&self, key: String) -> Result<Option<UserPreference>, Box<dyn std::error::Error>> {
        info!("Getting preference: {}", key);
        
        // 优先从持久记忆中获取
        if let Ok(Some(profile)) = self.persistent_memory.get_user_profile(&key) {
            let timestamp = profile.timestamp.clone();
            return Ok(Some(UserPreference {
                id: profile.id,
                key: profile.key,
                value: profile.value,
                created_at: timestamp.clone(),
                updated_at: timestamp,
            }));
        }
        
        let preference = sqlx::query_as::<_, UserPreference>(r#"
            SELECT id, key, value, created_at, updated_at
            FROM user_preferences
            WHERE key = ?
        "#)
        .bind(key)
        .fetch_optional(&self.db_pool)
        .await?;
        
        Ok(preference)
    }
    
    pub async fn add_interaction(&self, user_input: String, assistant_response: String) -> Result<InteractionHistory, Box<dyn std::error::Error>> {
        info!("Adding interaction to history");
        
        let timestamp = Utc::now().to_rfc3339();
        
        let history = sqlx::query_as::<_, InteractionHistory>(r#"
            INSERT INTO interaction_history (user_input, assistant_response, timestamp)
            VALUES (?, ?, ?)
            RETURNING id, user_input, assistant_response, timestamp
        "#)
        .bind(user_input)
        .bind(assistant_response)
        .bind(timestamp)
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(history)
    }
    
    pub async fn get_recent_interactions(&self, limit: i32) -> Result<Vec<InteractionHistory>, Box<dyn std::error::Error>> {
        info!("Getting recent interactions (limit: {})", limit);
        
        let interactions = sqlx::query_as::<_, InteractionHistory>(r#"
            SELECT id, user_input, assistant_response, timestamp
            FROM interaction_history
            ORDER BY timestamp DESC
            LIMIT ?
        "#)
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(interactions)
    }
    
    pub async fn search_memory(&self, query: String) -> Result<Vec<MemoryItem>, Box<dyn std::error::Error>> {
        info!("Searching memory for: {}", query);
        
        // 优先从持久记忆中搜索
        if let Ok(persistent_results) = self.persistent_memory.search_memory(&query) {
            let persistent_results: Vec<persistent_memory::MemoryItem> = persistent_results;
            if !persistent_results.is_empty() {
                return Ok(persistent_results.into_iter().map(|item| {
                    let timestamp = item.timestamp.clone();
                    MemoryItem {
                        id: item.id,
                        category: item.category,
                        key: item.key,
                        value: item.value,
                        created_at: timestamp.clone(),
                        updated_at: timestamp,
                    }
                }).collect());
            }
        }
        
        let items = sqlx::query_as::<_, MemoryItem>(r#"
            SELECT id, category, key, value, created_at, updated_at
            FROM memory_items
            WHERE category LIKE ? OR key LIKE ? OR value LIKE ?
            ORDER BY updated_at DESC
        "#)
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(items)
    }
    
    // 新增：技能管理相关方法
    pub fn add_skill(&self, name: &str, description: &str, content: &str) -> Result<persistent_memory::Skill, Box<dyn std::error::Error>> {
        Ok(self.persistent_memory.add_skill(name, description, content)?)
    }
    
    pub fn get_skill(&self, name: &str) -> Result<Option<persistent_memory::Skill>, Box<dyn std::error::Error>> {
        Ok(self.persistent_memory.get_skill(name)?)
    }
    
    pub fn update_skill_effectiveness(&self, skill_id: i64, effectiveness: f32) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self.persistent_memory.update_skill_effectiveness(skill_id, effectiveness)?)
    }
    
    pub fn get_all_skills(&self) -> Result<Vec<persistent_memory::Skill>, Box<dyn std::error::Error>> {
        Ok(self.persistent_memory.get_all_skills()?)
    }
    
    // 新增：添加持久记忆
    pub fn add_persistent_memory(&self, category: &str, key: &str, value: &str, importance: i32) -> Result<persistent_memory::MemoryItem, Box<dyn std::error::Error>> {
        Ok(self.persistent_memory.add_memory(category, key, value, importance)?)
    }
}
