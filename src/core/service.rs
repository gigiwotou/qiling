use crate::config::Config;
use crate::voice::VoiceModule;
use crate::ai::AIModule;
use crate::browser::BrowserModule;
use crate::task::TaskModule;
use crate::tool::ToolModule;
use crate::memory::MemoryModule;
use crate::dream::DreamModule;
use crate::permission::PermissionModule;
use crate::core::memory::MemoryOptimizer;
use crate::core::communication::CommManager;
use log::{info, warn};
use tokio::sync::mpsc;
use tokio::task;

pub struct CoreService {
    config: Config,
    voice_module: VoiceModule,
    ai_module: AIModule,
    browser_module: BrowserModule,
    task_module: TaskModule,
    tool_module: ToolModule,
    memory_module: std::sync::Arc<MemoryModule>,
    dream_module: DreamModule,
    permission_module: PermissionModule,
    memory_optimizer: MemoryOptimizer,
    comm_manager: CommManager,
    shutdown_rx: mpsc::Receiver<()>,
    shutdown_tx: mpsc::Sender<()>,
}

impl CoreService {
    pub async fn new(config: Config) -> Self {
        info!("Initializing Core Service...");
        
        // 初始化通信管理器
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        let comm_manager = CommManager::new();
        
        // 初始化内存优化器
        let memory_optimizer = MemoryOptimizer::new(config.server.low_memory_mode);
        
        // 初始化各个模块
        let voice_module = VoiceModule::new(&config, &comm_manager).await;
        let ai_module = AIModule::new(&config, &comm_manager).await;
        let browser_module = BrowserModule::new(&config, &comm_manager).await;
        let task_module = TaskModule::new(&config, &comm_manager).await;
        let tool_module = ToolModule::new(&config, comm_manager.clone()).await;
        let memory_module = MemoryModule::new(&config, &comm_manager).await;
        let memory_module_arc = std::sync::Arc::new(memory_module);
        let dream_module = DreamModule::new(&config, &comm_manager, memory_module_arc.clone()).await;
        let permission_module = PermissionModule::new(&config, &comm_manager).await;
        
        Self {
            config,
            voice_module,
            ai_module,
            browser_module,
            task_module,
            tool_module,
            memory_module: memory_module_arc,
            dream_module,
            permission_module,
            memory_optimizer,
            comm_manager,
            shutdown_rx,
            shutdown_tx,
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Core Service...");
        
        // 启动各个模块
        self.voice_module.start().await?;
        self.ai_module.start().await?;
        self.browser_module.start().await?;
        self.task_module.start().await?;
        self.tool_module.start().await?;
        
        // 克隆Arc以获取可变引用
        let mut memory_module = (*self.memory_module).clone();
        memory_module.start().await?;
        
        self.dream_module.start().await?;
        self.permission_module.start().await?;
        
        // 启动内存优化器
        self.memory_optimizer.start().await;
        
        info!("Core Service started successfully");
        
        // 等待关闭信号
        self.shutdown_rx.recv().await;
        
        // 停止各个模块
        self.stop().await;
        
        Ok(())
    }
    
    pub async fn stop(&mut self) {
        info!("Stopping Core Service...");
        
        // 停止各个模块
        self.voice_module.stop().await;
        self.ai_module.stop().await;
        self.browser_module.stop().await;
        self.task_module.stop().await;
        self.tool_module.stop().await;
        
        // 克隆Arc以获取可变引用
        let mut memory_module = (*self.memory_module).clone();
        memory_module.stop().await;
        
        self.dream_module.stop().await;
        self.permission_module.stop().await;
        
        // 停止内存优化器
        self.memory_optimizer.stop().await;
        
        info!("Core Service stopped successfully");
    }
    
    pub async fn send_command(&self, command: String) {
        // 发送命令到各个模块
        self.comm_manager.send_command(command).await;
    }
}
