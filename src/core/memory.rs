use sysinfo::{System, SystemExt};
use log::{info, warn};
use tokio::time;
use std::time::Duration;

pub struct MemoryOptimizer {
    low_memory_mode: bool,
    system: System,
}

impl MemoryOptimizer {
    pub fn new(low_memory_mode: bool) -> Self {
        Self {
            low_memory_mode,
            system: System::new_all(),
        }
    }
    
    pub async fn start(&mut self) {
        info!("Starting Memory Optimizer...");
        
        // 启动内存监控任务
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            let mut system = System::new_all();
            
            loop {
                interval.tick().await;
                system.refresh_all();
                
                let total_memory = system.total_memory();
                let used_memory = system.used_memory();
                let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
                
                info!("Memory usage: {:.2}% ({} MB / {} MB)", 
                      memory_usage_percent, 
                      used_memory / 1024 / 1024, 
                      total_memory / 1024 / 1024);
                
                // 如果内存使用率超过80%，进行内存优化
                if memory_usage_percent > 80.0 {
                    warn!("High memory usage detected: {:.2}%", memory_usage_percent);
                    // 这里可以添加内存优化逻辑，比如清理缓存等
                }
            }
        });
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping Memory Optimizer...");
        // 内存优化器是一个后台任务，不需要显式停止
    }
    
    pub fn get_memory_usage(&mut self) -> (u64, u64, f64) {
        self.system.refresh_all();
        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
        
        (total_memory, used_memory, memory_usage_percent)
    }
}
