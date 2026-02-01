//! 日志系统工具

use log::LevelFilter;
use std::io::Write;

/// 初始化日志系统
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] [{}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
    
    Ok(())
}

/// 日志记录器
pub struct Logger {
    logs: Vec<String>,
    max_logs: usize,
}

impl Logger {
    /// 创建新的日志记录器
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Vec::new(),
            max_logs,
        }
    }
    
    /// 添加日志
    pub fn add_log(&mut self, message: String) {
        self.logs.push(message);
        
        // 保持日志数量在限制内
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
    }
    
    /// 获取所有日志
    pub fn get_logs(&self) -> &[String] {
        &self.logs
    }
    
    /// 清空日志
    pub fn clear(&mut self) {
        self.logs.clear();
    }
}