//! 错误处理系统演示

// 只导入错误处理相关的模块，避免OpenCV依赖
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::fmt;
use std::error::Error as StdError;

// 复制必要的错误类型定义以避免依赖问题
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARN"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("配置文件不存在")]
    FileNotFound,
    #[error("配置文件格式错误: {0}")]
    ParseError(String),
    #[error("配置权限不足")]
    PermissionDenied,
}

impl ConfigError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ConfigError::FileNotFound => ErrorSeverity::Warning,
            ConfigError::ParseError(_) => ErrorSeverity::Error,
            ConfigError::PermissionDenied => ErrorSeverity::Fatal,
        }
    }
}

#[derive(Debug, Error)]
pub enum WindowError {
    #[error("未找到目标窗口")]
    WindowNotFound,
    #[error("窗口访问被拒绝")]
    AccessDenied,
    #[error("窗口截图失败: {0}")]
    CaptureError(String),
}

impl WindowError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            WindowError::WindowNotFound => ErrorSeverity::Warning,
            WindowError::AccessDenied => ErrorSeverity::Error,
            WindowError::CaptureError(_) => ErrorSeverity::Warning,
        }
    }
    
    pub fn capture_error(message: &str) -> Self {
        WindowError::CaptureError(message.to_string())
    }
}

#[derive(Debug, Error)]
pub enum ActionError {
    #[error("按键发送失败: {0}")]
    KeySendError(String),
    #[error("操作超时")]
    Timeout,
    #[error("系统资源不足")]
    InsufficientResources,
}

impl ActionError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ActionError::KeySendError(_) => ErrorSeverity::Error,
            ActionError::Timeout => ErrorSeverity::Warning,
            ActionError::InsufficientResources => ErrorSeverity::Fatal,
        }
    }
    
    pub fn key_send_error(message: &str) -> Self {
        ActionError::KeySendError(message.to_string())
    }
}

#[derive(Debug, Error)]
pub enum VisionError {
    #[error("模板匹配失败")]
    MatchingError,
    #[error("识别置信度过低: {0}")]
    LowConfidence(f32),
    #[error("模板文件不存在: {0}")]
    TemplateNotFound(String),
}

impl VisionError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            VisionError::MatchingError => ErrorSeverity::Warning,
            VisionError::LowConfidence(_) => ErrorSeverity::Info,
            VisionError::TemplateNotFound(_) => ErrorSeverity::Error,
        }
    }
    
    pub fn low_confidence(confidence: f32) -> Self {
        VisionError::LowConfidence(confidence)
    }
    
    pub fn template_not_found(path: &str) -> Self {
        VisionError::TemplateNotFound(path.to_string())
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("配置错误: {0}")]
    Config(#[from] ConfigError),
    #[error("窗口管理错误: {0}")]
    Window(#[from] WindowError),
    #[error("操作执行错误: {0}")]
    Action(#[from] ActionError),
    #[error("图像识别错误: {0}")]
    Vision(#[from] VisionError),
    #[error("UI错误: {0}")]
    UI(String),
    #[error("系统错误: {0}")]
    System(String),
}

impl AppError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Config(e) => e.severity(),
            AppError::Window(e) => e.severity(),
            AppError::Action(e) => e.severity(),
            AppError::Vision(e) => e.severity(),
            AppError::UI(_) => ErrorSeverity::Error,
            AppError::System(_) => ErrorSeverity::Fatal,
        }
    }
    
    pub fn component(&self) -> &'static str {
        match self {
            AppError::Config(_) => "Config",
            AppError::Window(_) => "Window",
            AppError::Action(_) => "Action",
            AppError::Vision(_) => "Vision",
            AppError::UI(_) => "UI",
            AppError::System(_) => "System",
        }
    }
    
    pub fn format_error(&self) -> String {
        format!("[{}] {}: {}", self.severity(), self.component(), self)
    }
    
    pub fn ui_error(message: &str) -> Self {
        AppError::UI(message.to_string())
    }
    
    pub fn system_error(message: &str) -> Self {
        AppError::System(message.to_string())
    }
}

pub struct ErrorFormatter;

impl ErrorFormatter {
    pub fn format_user_message(error: &AppError) -> String {
        match error {
            AppError::Config(ConfigError::FileNotFound) => {
                "配置文件不存在，将使用默认配置".to_string()
            }
            AppError::Config(ConfigError::ParseError(_)) => {
                "配置文件格式错误，请检查配置文件".to_string()
            }
            AppError::Window(WindowError::WindowNotFound) => {
                "未找到明日方舟游戏窗口，请确保游戏已启动".to_string()
            }
            AppError::Window(WindowError::AccessDenied) => {
                "无法访问游戏窗口，请以管理员身份运行程序".to_string()
            }
            AppError::Action(ActionError::Timeout) => {
                "操作执行超时，请检查游戏状态".to_string()
            }
            AppError::Vision(VisionError::MatchingError) => {
                "图像识别失败，请检查游戏界面".to_string()
            }
            _ => error.to_string(),
        }
    }
    
    pub fn format_technical_message(error: &AppError) -> String {
        format!(
            "[{}] {} - {} - {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            error.severity(),
            error.component(),
            error
        )
    }
    
    pub fn format_error_chain(error: &AppError) -> Vec<String> {
        let mut chain = vec![error.to_string()];
        
        let mut source = error.source();
        while let Some(err) = source {
            chain.push(err.to_string());
            source = err.source();
        }
        
        chain
    }
    
    pub fn is_recoverable(error: &AppError) -> bool {
        match error.severity() {
            ErrorSeverity::Info | ErrorSeverity::Warning => true,
            ErrorSeverity::Error => {
                matches!(
                    error,
                    AppError::Window(WindowError::WindowNotFound)
                        | AppError::Vision(VisionError::MatchingError)
                        | AppError::Action(ActionError::Timeout)
                        | AppError::Config(ConfigError::FileNotFound)
                )
            }
            ErrorSeverity::Fatal => false,
        }
    }
    
    pub fn get_suggestion(error: &AppError) -> Option<String> {
        match error {
            AppError::Window(WindowError::WindowNotFound) => {
                Some("请启动明日方舟游戏后重试".to_string())
            }
            AppError::Window(WindowError::AccessDenied) => {
                Some("请以管理员身份运行程序".to_string())
            }
            AppError::Config(ConfigError::FileNotFound) => {
                Some("程序将自动创建默认配置文件".to_string())
            }
            AppError::Config(ConfigError::PermissionDenied) => {
                Some("请检查配置文件的读写权限".to_string())
            }
            AppError::Vision(VisionError::TemplateNotFound(_)) => {
                Some("请确保模板文件存在且路径正确".to_string())
            }
            _ => None,
        }
    }
}

pub struct ErrorReporter;

impl ErrorReporter {
    pub fn report_to_log(error: &AppError) {
        let technical_msg = ErrorFormatter::format_technical_message(error);
        
        match error.severity() {
            ErrorSeverity::Info => println!("INFO: {}", technical_msg),
            ErrorSeverity::Warning => println!("WARN: {}", technical_msg),
            ErrorSeverity::Error => println!("ERROR: {}", technical_msg),
            ErrorSeverity::Fatal => println!("FATAL: {}", technical_msg),
        }
        
        let chain = ErrorFormatter::format_error_chain(error);
        if chain.len() > 1 {
            println!("Error chain: {:#?}", chain);
        }
        
        if let Some(suggestion) = ErrorFormatter::get_suggestion(error) {
            println!("Suggestion: {}", suggestion);
        }
    }
    
    pub fn report_to_ui(error: &AppError) -> String {
        let user_msg = ErrorFormatter::format_user_message(error);
        
        if let Some(suggestion) = ErrorFormatter::get_suggestion(error) {
            format!("{}\n建议: {}", user_msg, suggestion)
        } else {
            user_msg
        }
    }
    
    pub fn requires_immediate_attention(error: &AppError) -> bool {
        matches!(error.severity(), ErrorSeverity::Fatal)
            || matches!(
                error,
                AppError::System(_)
            )
    }
}

#[derive(Debug)]
pub enum RecoveryStrategy {
    Retry { max_attempts: u32, delay_ms: u64 },
    UseDefault,
    Skip,
    Abort,
}

impl RecoveryStrategy {
    pub fn for_error(error: &AppError) -> Self {
        match error {
            AppError::Window(WindowError::WindowNotFound) => {
                RecoveryStrategy::Retry { max_attempts: 5, delay_ms: 1000 }
            }
            AppError::Config(ConfigError::FileNotFound) => {
                RecoveryStrategy::UseDefault
            }
            AppError::Vision(VisionError::MatchingError) => {
                RecoveryStrategy::Skip
            }
            AppError::Action(ActionError::Timeout) => {
                RecoveryStrategy::Retry { max_attempts: 3, delay_ms: 500 }
            }
            AppError::System(_) => {
                RecoveryStrategy::Abort
            }
            _ => RecoveryStrategy::Skip,
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

pub trait ResultExt<T, E> {
    fn log_error(self) -> Self;
    fn to_user_error(self) -> Result<T, String>;
}

impl<T> ResultExt<T, AppError> for AppResult<T> {
    fn log_error(self) -> Self {
        if let Err(ref e) = self {
            ErrorReporter::report_to_log(e);
        }
        self
    }
    
    fn to_user_error(self) -> Result<T, String> {
        self.map_err(|e| ErrorReporter::report_to_ui(&e))
    }
}

fn main() {
    println!("=== 明日方舟智能鼠标宏 - 错误处理系统演示 ===\n");

    demo_config_errors();
    demo_window_errors();
    demo_action_errors();
    demo_vision_errors();
    demo_error_formatting();
    demo_recovery_strategies();
    
    let _ = demo_result_extensions();
}

fn demo_config_errors() {
    println!("1. 配置错误演示:");
    
    let errors = vec![
        AppError::Config(ConfigError::FileNotFound),
        AppError::Config(ConfigError::ParseError("无效的JSON格式".to_string())),
        AppError::Config(ConfigError::PermissionDenied),
    ];
    
    for error in errors {
        println!("  错误: {}", error);
        println!("  严重程度: {}", error.severity());
        println!("  用户消息: {}", ErrorFormatter::format_user_message(&error));
        if let Some(suggestion) = ErrorFormatter::get_suggestion(&error) {
            println!("  建议: {}", suggestion);
        }
        println!("  可恢复: {}", ErrorFormatter::is_recoverable(&error));
        println!();
    }
}

fn demo_window_errors() {
    println!("2. 窗口错误演示:");
    
    let errors = vec![
        AppError::Window(WindowError::WindowNotFound),
        AppError::Window(WindowError::AccessDenied),
        AppError::Window(WindowError::capture_error("截图设备不可用")),
    ];
    
    for error in errors {
        println!("  错误: {}", error);
        println!("  组件: {}", error.component());
        println!("  技术消息: {}", ErrorFormatter::format_technical_message(&error));
        println!();
    }
}

fn demo_action_errors() {
    println!("3. 操作错误演示:");
    
    let errors = vec![
        AppError::Action(ActionError::key_send_error("按键设备未响应")),
        AppError::Action(ActionError::Timeout),
        AppError::Action(ActionError::InsufficientResources),
    ];
    
    for error in errors {
        println!("  错误: {}", error);
        println!("  严重程度: {}", error.severity());
        ErrorReporter::report_to_log(&error);
        println!();
    }
}

fn demo_vision_errors() {
    println!("4. 图像识别错误演示:");
    
    let errors = vec![
        AppError::Vision(VisionError::MatchingError),
        AppError::Vision(VisionError::low_confidence(0.3)),
        AppError::Vision(VisionError::template_not_found("template.png")),
    ];
    
    for error in errors {
        println!("  错误: {}", error);
        println!("  UI消息: {}", ErrorReporter::report_to_ui(&error));
        println!();
    }
}

fn demo_error_formatting() {
    println!("5. 错误格式化演示:");
    
    let error = AppError::Config(ConfigError::ParseError("配置文件第10行语法错误".to_string()));
    
    println!("  原始错误: {}", error);
    println!("  格式化错误: {}", error.format_error());
    println!("  错误链: {:?}", ErrorFormatter::format_error_chain(&error));
    println!();
}

fn demo_recovery_strategies() {
    println!("6. 恢复策略演示:");
    
    let errors = vec![
        AppError::Window(WindowError::WindowNotFound),
        AppError::Config(ConfigError::FileNotFound),
        AppError::Action(ActionError::Timeout),
        AppError::system_error("系统崩溃"),
    ];
    
    for error in errors {
        let strategy = RecoveryStrategy::for_error(&error);
        println!("  错误: {}", error);
        match strategy {
            RecoveryStrategy::Retry { max_attempts, delay_ms } => {
                println!("  策略: 重试 (最多{}次, 间隔{}ms)", max_attempts, delay_ms);
            }
            RecoveryStrategy::UseDefault => {
                println!("  策略: 使用默认值");
            }
            RecoveryStrategy::Skip => {
                println!("  策略: 跳过操作");
            }
            RecoveryStrategy::Abort => {
                println!("  策略: 终止程序");
            }
        }
        println!("  需要立即处理: {}", ErrorReporter::requires_immediate_attention(&error));
        println!();
    }
}

fn demo_result_extensions() -> AppResult<()> {
    println!("7. Result扩展方法演示:");
    
    let result: AppResult<i32> = Err(AppError::ui_error("测试错误"));
    
    let _logged = result.log_error();
    
    let user_result: Result<i32, String> = Err(AppError::Config(ConfigError::FileNotFound)).to_user_error();
    match user_result {
        Ok(_) => println!("  成功"),
        Err(msg) => println!("  用户错误: {}", msg),
    }
    
    println!();
    Ok(())
}