use crate::config::Config;
use crate::core::communication::CommManager;
use log::{info, warn};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct TaskModule {
    config: TaskConfig,
    comm_manager: Arc<CommManager>,
    db_pool: SqlitePool,
    shutdown: bool,
}

#[derive(Debug, Clone)]
pub struct TaskConfig {
    pub reminder_interval: u32,
    pub max_tasks: u32,
}

impl From<&Config> for TaskConfig {
    fn from(config: &Config) -> Self {
        Self {
            reminder_interval: config.task.reminder_interval,
            max_tasks: config.task.max_tasks,
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub due_date: Option<String>,
    pub completed: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Resource {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub description: String,
    pub tags: String,
    pub created_at: String,
}

impl TaskModule {
    pub async fn new(config: &Config, comm_manager: &CommManager) -> Self {
        info!("Initializing Task Module...");
        
        // 初始化数据库连接
        let db_pool = SqlitePool::connect("sqlite://tasks.db").await.unwrap();
        
        // 创建任务表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                due_date TEXT,
                completed BOOLEAN DEFAULT FALSE,
                created_at TEXT NOT NULL
            )
        "#).execute(&db_pool).await.unwrap();
        
        // 创建资源表
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS resources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                url TEXT,
                description TEXT,
                tags TEXT,
                created_at TEXT NOT NULL
            )
        "#).execute(&db_pool).await.unwrap();
        
        Self {
            config: TaskConfig::from(config),
            comm_manager: Arc::new(comm_manager.clone()),
            db_pool,
            shutdown: false,
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Task Module...");
        
        let config = self.config.clone();
        let comm_manager = self.comm_manager.clone();
        let db_pool = self.db_pool.clone();
        
        // 启动任务提醒任务
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(config.reminder_interval as u64 * 60));
            
            while let _ = interval.tick().await {
                // 检查任务到期情况
                check_due_tasks(&db_pool, &comm_manager).await;
            }
        });
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping Task Module...");
        self.shutdown = true;
    }
    
    pub async fn add_task(&self, title: String, description: String, due_date: Option<String>) -> Result<Task, Box<dyn std::error::Error>> {
        info!("Adding task: {}", title);
        
        let task = sqlx::query_as::<_, Task>(r#"
            INSERT INTO tasks (title, description, due_date, created_at)
            VALUES (?, ?, ?, ?)
            RETURNING id, title, description, due_date, completed, created_at
        "#)
        .bind(title)
        .bind(description)
        .bind(due_date)
        .bind(Utc::now().to_rfc3339())
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(task)
    }
    
    pub async fn get_tasks(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let tasks = sqlx::query_as::<_, Task>(r#"
            SELECT id, title, description, due_date, completed, created_at
            FROM tasks
            ORDER BY due_date ASC NULLS LAST, created_at DESC
        "#)
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(tasks)
    }
    
    pub async fn complete_task(&self, task_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        info!("Completing task: {}", task_id);
        
        sqlx::query(r#"
            UPDATE tasks
            SET completed = 1
            WHERE id = ?
        "#)
        .bind(task_id)
        .execute(&self.db_pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn add_resource(&self, title: String, url: String, description: String, tags: String) -> Result<Resource, Box<dyn std::error::Error>> {
        info!("Adding resource: {}", title);
        
        let resource = sqlx::query_as::<_, Resource>(r#"
            INSERT INTO resources (title, url, description, tags, created_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id, title, url, description, tags, created_at
        "#)
        .bind(title)
        .bind(url)
        .bind(description)
        .bind(tags)
        .bind(Utc::now().to_rfc3339())
        .fetch_one(&self.db_pool)
        .await?;
        
        Ok(resource)
    }
    
    pub async fn search_resources(&self, query: String) -> Result<Vec<Resource>, Box<dyn std::error::Error>> {
        info!("Searching resources for: {}", query);
        
        let resources = sqlx::query_as::<_, Resource>(r#"
            SELECT id, title, url, description, tags, created_at
            FROM resources
            WHERE title LIKE ? OR description LIKE ? OR tags LIKE ?
            ORDER BY created_at DESC
        "#)
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .fetch_all(&self.db_pool)
        .await?;
        
        Ok(resources)
    }
}

async fn check_due_tasks(db_pool: &SqlitePool, comm_manager: &CommManager) {
    info!("Checking due tasks...");
    
    let now = Utc::now();
    let tasks = sqlx::query_as::<_, Task>(r#"
        SELECT id, title, description, due_date, completed, created_at
        FROM tasks
        WHERE completed = 0 AND due_date IS NOT NULL AND due_date <= ?
    "#)
    .bind(now.to_rfc3339())
    .fetch_all(db_pool)
    .await
    .unwrap();
    
    for task in tasks {
        info!("Task due: {}", task.title);
        // 发送任务提醒
        comm_manager.send_command(format!("TASK_REMINDER:{}", task.title)).await;
    }
}
