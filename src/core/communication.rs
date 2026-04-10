use tokio::sync::broadcast;
use log::info;

#[derive(Clone)]
pub struct CommManager {
    tx: broadcast::Sender<String>,
}

impl CommManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
        }
    }
    
    pub async fn send_command(&self, command: String) {
        info!("Sending command: {}", command);
        let _ = self.tx.send(command);
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}
