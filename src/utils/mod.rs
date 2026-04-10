use log::{info, warn, error};
use std::time::Duration;
use tokio::time;

pub fn setup_logging() {
    env_logger::init();
    info!("Logging initialized");
}

pub async fn retry<F, T, E>(mut f: F, max_attempts: u32, delay: Duration) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempts = 0;
    loop {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(e);
                }
                warn!("Attempt {} failed, retrying in {:?}...", attempts, delay);
                time::sleep(delay).await;
            }
        }
    }
}

pub fn sanitize_input(input: &str) -> String {
    // 简单的输入 sanitization
    input.trim().to_string()
}

pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    
    if days > 0 {
        format!("{} days, {} hours, {} minutes, {} seconds", days, hours % 24, minutes % 60, seconds % 60)
    } else if hours > 0 {
        format!("{} hours, {} minutes, {} seconds", hours, minutes % 60, seconds % 60)
    } else if minutes > 0 {
        format!("{} minutes, {} seconds", minutes, seconds % 60)
    } else {
        format!("{} seconds", seconds)
    }
}
