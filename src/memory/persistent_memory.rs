use crate::core::communication::CommManager;
use log::{info, warn};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
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
    db_path: String,
    comm_manager: Arc<CommManager>,
}

impl PersistentMemory {
    pub fn new(db_path: &str, comm_manager: Arc<CommManager>) -> Result<Self> {
        let memory = Self {
            db_path: db_path.to_string(),
            comm_manager,
        };
        
        memory.initialize_db()?;
        Ok(memory)
    }
    
    fn initialize_db(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        
        // 创建记忆表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memory_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                importance INTEGER DEFAULT 0
            )",
            [],
        )?;
        
        // 创建技能表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS skills (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                content TEXT NOT NULL,
                usage_count INTEGER DEFAULT 0,
                last_used TEXT NOT NULL,
                effectiveness REAL DEFAULT 0.5
            )",
            [],
        )?;
        
        // 创建用户配置表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_profile (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;
        
        // 创建索引
        conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_category ON memory_items(category)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_memory_key ON memory_items(key)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_skills_name ON skills(name)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_user_profile_key ON user_profile(key)", [])?;
        
        Ok(())
    }
    
    pub fn add_memory(&self, category: &str, key: &str, value: &str, importance: i32) -> Result<MemoryItem> {
        let conn = Connection::open(&self.db_path)?;
        let timestamp = Self::get_current_timestamp();
        
        let id = conn.execute(
            "INSERT INTO memory_items (category, key, value, timestamp, importance) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![category, key, value, timestamp, importance],
        )?;
        
        Ok(MemoryItem {
            id: id as i64,
            category: category.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            timestamp,
            importance,
        })
    }
    
    pub fn get_memory(&self, category: &str, key: &str) -> Result<Option<MemoryItem>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, category, key, value, timestamp, importance 
             FROM memory_items 
             WHERE category = ?1 AND key = ?2 
             ORDER BY timestamp DESC LIMIT 1"
        )?;
        
        let mut rows = stmt.query(params![category, key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(MemoryItem {
                id: row.get::<_, i64>(0)?,
                category: row.get::<_, String>(1)?,
                key: row.get::<_, String>(2)?,
                value: row.get::<_, String>(3)?,
                timestamp: row.get::<_, String>(4)?,
                importance: row.get::<_, i32>(5)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn search_memory(&self, query: &str) -> Result<Vec<MemoryItem>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, category, key, value, timestamp, importance 
             FROM memory_items 
             WHERE value LIKE ?1 OR key LIKE ?1 OR category LIKE ?1 
             ORDER BY importance DESC, timestamp DESC 
             LIMIT 10"
        )?;
        
        let mut rows = stmt.query(params![format!("%{query}%")])?;
        let mut results = Vec::new();
        
        while let Some(row) = rows.next()? {
            results.push(MemoryItem {
                id: row.get::<_, i64>(0)?,
                category: row.get::<_, String>(1)?,
                key: row.get::<_, String>(2)?,
                value: row.get::<_, String>(3)?,
                timestamp: row.get::<_, String>(4)?,
                importance: row.get::<_, i32>(5)?,
            });
        }
        
        Ok(results)
    }
    
    pub fn add_skill(&self, name: &str, description: &str, content: &str) -> Result<Skill> {
        let conn = Connection::open(&self.db_path)?;
        let timestamp = Self::get_current_timestamp();
        
        let id = conn.execute(
            "INSERT INTO skills (name, description, content, usage_count, last_used, effectiveness) 
             VALUES (?1, ?2, ?3, 0, ?4, 0.5)",
            params![name, description, content, timestamp],
        )?;
        
        Ok(Skill {
            id: id as i64,
            name: name.to_string(),
            description: description.to_string(),
            content: content.to_string(),
            usage_count: 0,
            last_used: timestamp,
            effectiveness: 0.5,
        })
    }
    
    pub fn get_skill(&self, name: &str) -> Result<Option<Skill>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, content, usage_count, last_used, effectiveness 
             FROM skills 
             WHERE name = ?1"
        )?;
        
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Skill {
                id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
                description: row.get::<_, String>(2)?,
                content: row.get::<_, String>(3)?,
                usage_count: row.get::<_, i32>(4)?,
                last_used: row.get::<_, String>(5)?,
                effectiveness: row.get::<_, f32>(6)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn update_skill_effectiveness(&self, skill_id: i64, effectiveness: f32) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        let timestamp = Self::get_current_timestamp();
        
        conn.execute(
            "UPDATE skills 
             SET effectiveness = ?1, last_used = ?2, usage_count = usage_count + 1 
             WHERE id = ?3",
            params![effectiveness, timestamp, skill_id],
        )?;
        
        Ok(())
    }
    
    pub fn get_all_skills(&self) -> Result<Vec<Skill>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, content, usage_count, last_used, effectiveness 
             FROM skills 
             ORDER BY usage_count DESC, effectiveness DESC"
        )?;
        
        let mut rows = stmt.query([])?;
        let mut skills = Vec::new();
        
        while let Some(row) = rows.next()? {
            skills.push(Skill {
                id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
                description: row.get::<_, String>(2)?,
                content: row.get::<_, String>(3)?,
                usage_count: row.get::<_, i32>(4)?,
                last_used: row.get::<_, String>(5)?,
                effectiveness: row.get::<_, f32>(6)?,
            });
        }
        
        Ok(skills)
    }
    
    pub fn set_user_profile(&self, key: &str, value: &str) -> Result<UserProfile> {
        let conn = Connection::open(&self.db_path)?;
        let timestamp = Self::get_current_timestamp();
        
        // 检查是否已存在
        let mut stmt = conn.prepare("SELECT id FROM user_profile WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        
        if let Some(row) = rows.next()? {
            let id: i64 = row.get::<_, i64>(0)?;
            conn.execute(
                "UPDATE user_profile SET value = ?1, timestamp = ?2 WHERE id = ?3",
                params![value, timestamp, id],
            )?;
            Ok(UserProfile {
                id,
                key: key.to_string(),
                value: value.to_string(),
                timestamp,
            })
        } else {
            let id = conn.execute(
                "INSERT INTO user_profile (key, value, timestamp) VALUES (?1, ?2, ?3)",
                params![key, value, timestamp],
            )?;
            Ok(UserProfile {
                id: id as i64,
                key: key.to_string(),
                value: value.to_string(),
                timestamp,
            })
        }
    }
    
    pub fn get_user_profile(&self, key: &str) -> Result<Option<UserProfile>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare(
            "SELECT id, key, value, timestamp FROM user_profile WHERE key = ?1"
        )?;
        
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(UserProfile {
                id: row.get::<_, i64>(0)?,
                key: row.get::<_, String>(1)?,
                value: row.get::<_, String>(2)?,
                timestamp: row.get::<_, String>(3)?,
            }))
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
