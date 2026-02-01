//! 统一错误处理系统
//! 
//! 提供统一的错误类型定义、错误转换和错误信息格式化功能

use thiserror::Error;
use std::fmt;
use std::error::Error as StdError;

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// 信息级别 - 不影响功能
    Info,
    /// 警告级别 - 可能影响功能但可以继续
    Warning,
    /// 错误级别 - 影响功能但不致命
    Error,
    /// 致命级别 - 导致程序无法继续
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

/// 错误上下文信息
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// 错误发生的组件
    pub component: String,
    /// 错误发生的操作
    pub operation: String,
    /// 额外的上下文信息
    pub details: Option<String>,
    /// 错误严重程度
    pub severity: ErrorSeverity,
}

impl ErrorContext {
    /// 创建新的错误上下文
    pub fn new(component: &str, operation: &str) -> Self {
        Self {
            component: component.to_string(),
            operation: operation.to_string(),
            details: None,
            severity: ErrorSeverity::Error,
        }
    }
    
    /// 设置详细信息
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }
    
    /// 设置严重程度
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}::{}", self.severity, self.component, self.operation)?;
        if let Some(details) = &self.details {
            write!(f, " - {}", details)?;
        }
        Ok(())
    }
}

/// 应用程序主要错误类型
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
    
    #[error("状态管理错误: {0}")]
    State(#[from] StateError),
    
    #[error("模式管理错误: {0}")]
    Mode(#[from] ModeError),
    
    #[error("UI错误: {0}")]
    UI(String),
    
    #[error("系统错误: {0}")]
    System(String),
    
    #[error("初始化错误: {0}")]
    Initialization(String),
    
    #[error("网络错误: {0}")]
    Network(String),
}

impl AppError {
    /// 获取错误的严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Config(e) => e.severity(),
            AppError::Window(e) => e.severity(),
            AppError::Action(e) => e.severity(),
            AppError::Vision(e) => e.severity(),
            AppError::State(e) => e.severity(),
            AppError::Mode(e) => e.severity(),
            AppError::UI(_) => ErrorSeverity::Error,
            AppError::System(_) => ErrorSeverity::Fatal,
            AppError::Initialization(_) => ErrorSeverity::Fatal,
            AppError::Network(_) => ErrorSeverity::Warning,
        }
    }
    
    /// 获取错误的组件名称
    pub fn component(&self) -> &'static str {
        match self {
            AppError::Config(_) => "Config",
            AppError::Window(_) => "Window",
            AppError::Action(_) => "Action",
            AppError::Vision(_) => "Vision",
            AppError::State(_) => "State",
            AppError::Mode(_) => "Mode",
            AppError::UI(_) => "UI",
            AppError::System(_) => "System",
            AppError::Initialization(_) => "Init",
            AppError::Network(_) => "Network",
        }
    }
    
    /// 格式化错误信息
    pub fn format_error(&self) -> String {
        format!("[{}] {}: {}", self.severity(), self.component(), self)
    }
    
    /// 创建UI错误
    pub fn ui_error(message: &str) -> Self {
        AppError::UI(message.to_string())
    }
    
    /// 创建系统错误
    pub fn system_error(message: &str) -> Self {
        AppError::System(message.to_string())
    }
    
    /// 创建初始化错误
    pub fn initialization_error(message: &str) -> Self {
        AppError::Initialization(message.to_string())
    }
    
    /// 创建网络错误
    pub fn network_error(message: &str) -> Self {
        AppError::Network(message.to_string())
    }
}

/// 配置相关错误
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("配置文件不存在")]
    FileNotFound,
    
    #[error("配置文件格式错误: {0}")]
    ParseError(String),
    
    #[error("配置保存失败: {0}")]
    SaveError(String),
    
    #[error("配置验证失败: {0}")]
    ValidationError(String),
    
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("序列化错误: {0}")]
    SerializationError(String),
    
    #[error("配置路径无效: {0}")]
    InvalidPath(String),
    
    #[error("配置权限不足")]
    PermissionDenied,
    
    #[error("配置文件损坏")]
    CorruptedFile,
}

impl ConfigError {
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ConfigError::FileNotFound => ErrorSeverity::Warning,
            ConfigError::ParseError(_) => ErrorSeverity::Error,
            ConfigError::SaveError(_) => ErrorSeverity::Error,
            ConfigError::ValidationError(_) => ErrorSeverity::Warning,
            ConfigError::IoError(_) => ErrorSeverity::Error,
            ConfigError::SerializationError(_) => ErrorSeverity::Error,
            ConfigError::InvalidPath(_) => ErrorSeverity::Error,
            ConfigError::PermissionDenied => ErrorSeverity::Fatal,
            ConfigError::CorruptedFile => ErrorSeverity::Error,
        }
    }
    
    /// 创建序列化错误
    pub fn serialization_error(message: &str) -> Self {
        ConfigError::SerializationError(message.to_string())
    }
    
    /// 创建验证错误
    pub fn validation_error(message: &str) -> Self {
        ConfigError::ValidationError(message.to_string())
    }
}

// 为serde错误实现转换
impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::SerializationError(err.to_string())
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(err: toml::ser::Error) -> Self {
        ConfigError::SerializationError(err.to_string())
    }
}

/// 窗口管理相关错误
#[derive(Debug, Error)]
pub enum WindowError {
    #[error("未找到目标窗口")]
    WindowNotFound,
    
    #[error("窗口访问被拒绝")]
    AccessDenied,
    
    #[error("窗口截图失败: {0}")]
    CaptureError(String),
    
    #[error("窗口信息获取失败: {0}")]
    InfoError(String),
    
    #[error("坐标转换失败")]
    CoordinateError,
    
    #[error("窗口句柄无效")]
    InvalidHandle,
    
    #[error("窗口已关闭")]
    WindowClosed,
    
    #[error("系统API调用失败: {0}")]
    SystemApiError(String),
}

impl WindowError {
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            WindowError::WindowNotFound => ErrorSeverity::Warning,
            WindowError::AccessDenied => ErrorSeverity::Error,
            WindowError::CaptureError(_) => ErrorSeverity::Warning,
            WindowError::InfoError(_) => ErrorSeverity::Warning,
            WindowError::CoordinateError => ErrorSeverity::Error,
            WindowError::InvalidHandle => ErrorSeverity::Error,
            WindowError::WindowClosed => ErrorSeverity::Warning,
            WindowError::SystemApiError(_) => ErrorSeverity::Error,
        }
    }
    
    /// 创建截图错误
    pub fn capture_error(message: &str) -> Self {
        WindowError::CaptureError(message.to_string())
    }
    
    /// 创建信息获取错误
    pub fn info_error(message: &str) -> Self {
        WindowError::InfoError(message.to_string())
    }
    
    /// 创建系统API错误
    pub fn system_api_error(message: &str) -> Self {
        WindowError::SystemApiError(message.to_string())
    }
}

/// 操作执行相关错误
#[derive(Debug, Error)]
pub enum ActionError {
    #[error("按键发送失败: {0}")]
    KeySendError(String),
    
    #[error("鼠标操作失败: {0}")]
    MouseError(String),
    
    #[error("坐标转换失败")]
    CoordinateError,
    
    #[error("操作序列执行失败: {0}")]
    SequenceError(String),
    
    #[error("操作超时")]
    Timeout,
    
    #[error("操作被取消")]
    Cancelled,
    
    #[error("无效的操作参数: {0}")]
    InvalidParameter(String),
    
    #[error("操作不支持: {0}")]
    UnsupportedOperation(String),
    
    #[error("系统资源不足")]
    InsufficientResources,
    
    #[error("无效的按键: {0}")]
    InvalidKey(String),
    
    #[error("系统调用失败: {0}")]
    SystemCall(String),
    
    #[error("平台不支持: {0}")]
    UnsupportedPlatform(String),
}

impl ActionError {
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ActionError::KeySendError(_) => ErrorSeverity::Error,
            ActionError::MouseError(_) => ErrorSeverity::Error,
            ActionError::CoordinateError => ErrorSeverity::Error,
            ActionError::SequenceError(_) => ErrorSeverity::Error,
            ActionError::Timeout => ErrorSeverity::Warning,
            ActionError::Cancelled => ErrorSeverity::Info,
            ActionError::InvalidParameter(_) => ErrorSeverity::Error,
            ActionError::UnsupportedOperation(_) => ErrorSeverity::Error,
            ActionError::InsufficientResources => ErrorSeverity::Fatal,
            ActionError::InvalidKey(_) => ErrorSeverity::Error,
            ActionError::SystemCall(_) => ErrorSeverity::Error,
            ActionError::UnsupportedPlatform(_) => ErrorSeverity::Warning,
        }
    }
    
    /// 创建按键发送错误
    pub fn key_send_error(message: &str) -> Self {
        ActionError::KeySendError(message.to_string())
    }
    
    /// 创建鼠标操作错误
    pub fn mouse_error(message: &str) -> Self {
        ActionError::MouseError(message.to_string())
    }
    
    /// 创建序列执行错误
    pub fn sequence_error(message: &str) -> Self {
        ActionError::SequenceError(message.to_string())
    }
    
    /// 创建无效参数错误
    pub fn invalid_parameter(message: &str) -> Self {
        ActionError::InvalidParameter(message.to_string())
    }
    
    /// 创建不支持操作错误
    pub fn unsupported_operation(message: &str) -> Self {
        ActionError::UnsupportedOperation(message.to_string())
    }
    
    /// 创建无效按键错误
    pub fn invalid_key(key: &str) -> Self {
        ActionError::InvalidKey(key.to_string())
    }
    
    /// 创建系统调用错误
    pub fn system_call(message: &str) -> Self {
        ActionError::SystemCall(message.to_string())
    }
    
    /// 创建平台不支持错误
    pub fn unsupported_platform(message: &str) -> Self {
        ActionError::UnsupportedPlatform(message.to_string())
    }
}

/// 图像识别相关错误
#[derive(Debug, Error)]
pub enum VisionError {
    #[error("图像处理失败: {0}")]
    ProcessingError(String),
    
    #[error("模板匹配失败")]
    MatchingError,
    
    #[error("OpenCV错误: {0}")]
    OpenCVError(String),
    
    #[error("图像格式不支持: {0}")]
    UnsupportedFormat(String),
    
    #[error("截图失败: {0}")]
    CaptureError(String),
    
    #[error("图像为空")]
    EmptyImage,
    
    #[error("模板文件不存在: {0}")]
    TemplateNotFound(String),
    
    #[error("识别置信度过低: {0}")]
    LowConfidence(f32),
    
    #[error("图像尺寸不匹配")]
    SizeMismatch,
}

impl VisionError {
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            VisionError::ProcessingError(_) => ErrorSeverity::Error,
            VisionError::MatchingError => ErrorSeverity::Warning,
            VisionError::OpenCVError(_) => ErrorSeverity::Error,
            VisionError::UnsupportedFormat(_) => ErrorSeverity::Error,
            VisionError::CaptureError(_) => ErrorSeverity::Warning,
            VisionError::EmptyImage => ErrorSeverity::Warning,
            VisionError::TemplateNotFound(_) => ErrorSeverity::Error,
            VisionError::LowConfidence(_) => ErrorSeverity::Info,
            VisionError::SizeMismatch => ErrorSeverity::Warning,
        }
    }
    
    /// 创建处理错误
    pub fn processing_error(message: &str) -> Self {
        VisionError::ProcessingError(message.to_string())
    }
    
    /// 创建OpenCV错误
    pub fn opencv_error(message: &str) -> Self {
        VisionError::OpenCVError(message.to_string())
    }
    
    /// 创建不支持格式错误
    pub fn unsupported_format(format: &str) -> Self {
        VisionError::UnsupportedFormat(format.to_string())
    }
    
    /// 创建截图错误
    pub fn capture_error(message: &str) -> Self {
        VisionError::CaptureError(message.to_string())
    }
    
    /// 创建模板未找到错误
    pub fn template_not_found(path: &str) -> Self {
        VisionError::TemplateNotFound(path.to_string())
    }
    
    /// 创建低置信度错误
    pub fn low_confidence(confidence: f32) -> Self {
        VisionError::LowConfidence(confidence)
    }
}

// 为OpenCV错误实现转换
impl From<opencv::Error> for VisionError {
    fn from(err: opencv::Error) -> Self {
        VisionError::OpenCVError(err.to_string())
    }
}

/// 状态管理相关错误
#[derive(Debug, Error)]
pub enum StateError {
    #[error("状态转换无效: 从 {from} 到 {to}")]
    InvalidTransition { from: String, to: String },
    
    #[error("状态锁定失败")]
    LockError,
    
    #[error("观察者注册失败: {0}")]
    ObserverError(String),
    
    #[error("状态不一致")]
    InconsistentState,
    
    #[error("状态初始化失败: {0}")]
    InitializationError(String),
    
    #[error("状态持久化失败: {0}")]
    PersistenceError(String),
}

impl StateError {
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            StateError::InvalidTransition { .. } => ErrorSeverity::Error,
            StateError::LockError => ErrorSeverity::Error,
            StateError::ObserverError(_) => ErrorSeverity::Warning,
            StateError::InconsistentState => ErrorSeverity::Error,
            StateError::InitializationError(_) => ErrorSeverity::Fatal,
            StateError::PersistenceError(_) => ErrorSeverity::Warning,
        }
    }
    
    /// 创建无效转换错误
    pub fn invalid_transition(from: &str, to: &str) -> Self {
        StateError::InvalidTransition {
            from: from.to_string(),
            to: to.to_string(),
        }
    }
    
    /// 创建观察者错误
    pub fn observer_error(message: &str) -> Self {
        StateError::ObserverError(message.to_string())
    }
    
    /// 创建初始化错误
    pub fn initialization_error(message: &str) -> Self {
        StateError::InitializationError(message.to_string())
    }
    
    /// 创建持久化错误
    pub fn persistence_error(message: &str) -> Self {
        StateError::PersistenceError(message.to_string())
    }
}

/// 模式管理相关错误
#[derive(Debug, Error)]
pub enum ModeError {
    #[error("模式切换失败: {0}")]
    SwitchError(String),
    
    #[error("模式配置无效: {0}")]
    ConfigError(String),
    
    #[error("不支持的模式: {0}")]
    UnsupportedMode(String),
    
    #[error("模式初始化失败: {0}")]
    InitializationError(String),
    
    #[error("模式验证失败: {0}")]
    ValidationError(String),
}

impl ModeError {
    /// 获取错误严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ModeError::SwitchError(_) => ErrorSeverity::Error,
            ModeError::ConfigError(_) => ErrorSeverity::Error,
            ModeError::UnsupportedMode(_) => ErrorSeverity::Error,
            ModeError::InitializationError(_) => ErrorSeverity::Fatal,
            ModeError::ValidationError(_) => ErrorSeverity::Warning,
        }
    }
    
    /// 创建切换错误
    pub fn switch_error(message: &str) -> Self {
        ModeError::SwitchError(message.to_string())
    }
    
    /// 创建配置错误
    pub fn config_error(message: &str) -> Self {
        ModeError::ConfigError(message.to_string())
    }
    
    /// 创建不支持模式错误
    pub fn unsupported_mode(mode: &str) -> Self {
        ModeError::UnsupportedMode(mode.to_string())
    }
    
    /// 创建初始化错误
    pub fn initialization_error(message: &str) -> Self {
        ModeError::InitializationError(message.to_string())
    }
    
    /// 创建验证错误
    pub fn validation_error(message: &str) -> Self {
        ModeError::ValidationError(message.to_string())
    }
}

/// 错误格式化工具
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// 格式化错误为用户友好的消息
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
    
    /// 格式化错误为详细的技术消息
    pub fn format_technical_message(error: &AppError) -> String {
        format!(
            "[{}] {} - {} - {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            error.severity(),
            error.component(),
            error
        )
    }
    
    /// 格式化错误链
    pub fn format_error_chain(error: &AppError) -> Vec<String> {
        let mut chain = vec![error.to_string()];
        
        // 添加源错误信息
        let mut source = error.source();
        while let Some(err) = source {
            chain.push(err.to_string());
            source = err.source();
        }
        
        chain
    }
    
    /// 检查错误是否可恢复
    pub fn is_recoverable(error: &AppError) -> bool {
        match error.severity() {
            ErrorSeverity::Info | ErrorSeverity::Warning => true,
            ErrorSeverity::Error => {
                // 某些错误类型是可恢复的
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
    
    /// 获取错误的建议解决方案
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

/// 错误报告器
pub struct ErrorReporter;

impl ErrorReporter {
    /// 报告错误到日志系统
    pub fn report_to_log(error: &AppError) {
        let technical_msg = ErrorFormatter::format_technical_message(error);
        
        match error.severity() {
            ErrorSeverity::Info => log::info!("{}", technical_msg),
            ErrorSeverity::Warning => log::warn!("{}", technical_msg),
            ErrorSeverity::Error => log::error!("{}", technical_msg),
            ErrorSeverity::Fatal => log::error!("FATAL: {}", technical_msg),
        }
        
        // 记录错误链
        let chain = ErrorFormatter::format_error_chain(error);
        if chain.len() > 1 {
            log::debug!("Error chain: {:#?}", chain);
        }
        
        // 记录建议解决方案
        if let Some(suggestion) = ErrorFormatter::get_suggestion(error) {
            log::info!("Suggestion: {}", suggestion);
        }
    }
    
    /// 报告错误到UI系统
    pub fn report_to_ui(error: &AppError) -> String {
        let user_msg = ErrorFormatter::format_user_message(error);
        
        if let Some(suggestion) = ErrorFormatter::get_suggestion(error) {
            format!("{}\n建议: {}", user_msg, suggestion)
        } else {
            user_msg
        }
    }
    
    /// 检查是否需要立即处理
    pub fn requires_immediate_attention(error: &AppError) -> bool {
        matches!(error.severity(), ErrorSeverity::Fatal)
            || matches!(
                error,
                AppError::System(_) | AppError::Initialization(_)
            )
    }
}

/// 结果类型别名
pub type AppResult<T> = Result<T, AppError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type WindowResult<T> = Result<T, WindowError>;
pub type ActionResult<T> = Result<T, ActionError>;
pub type VisionResult<T> = Result<T, VisionError>;
pub type StateResult<T> = Result<T, StateError>;
pub type ModeResult<T> = Result<T, ModeError>;

/// 扩展Result类型的便利方法
pub trait ResultExt<T, E> {
    /// 添加错误上下文
    fn with_context(self, context: ErrorContext) -> Result<T, AppError>;
    
    /// 记录错误并继续
    fn log_error(self) -> Self;
    
    /// 转换为用户友好的错误消息
    fn to_user_error(self) -> Result<T, String>;
}

impl<T> ResultExt<T, AppError> for AppResult<T> {
    fn with_context(self, context: ErrorContext) -> Result<T, AppError> {
        self.map_err(|e| {
            log::error!("{}: {}", context, e);
            e
        })
    }
    
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

/// 错误恢复策略
pub enum RecoveryStrategy {
    /// 重试操作
    Retry { max_attempts: u32, delay_ms: u64 },
    /// 使用默认值
    UseDefault,
    /// 跳过操作
    Skip,
    /// 终止程序
    Abort,
}

impl RecoveryStrategy {
    /// 根据错误类型获取推荐的恢复策略
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
            AppError::System(_) | AppError::Initialization(_) => {
                RecoveryStrategy::Abort
            }
            _ => RecoveryStrategy::Skip,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity() {
        let config_error = ConfigError::FileNotFound;
        assert_eq!(config_error.severity(), ErrorSeverity::Warning);
        
        let window_error = WindowError::AccessDenied;
        assert_eq!(window_error.severity(), ErrorSeverity::Error);
        
        let action_error = ActionError::InsufficientResources;
        assert_eq!(action_error.severity(), ErrorSeverity::Fatal);
    }

    #[test]
    fn test_app_error_conversion() {
        let config_error = ConfigError::ParseError("invalid format".to_string());
        let app_error: AppError = config_error.into();
        
        assert_eq!(app_error.component(), "Config");
        assert_eq!(app_error.severity(), ErrorSeverity::Error);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("TestComponent", "test_operation")
            .with_details("test details")
            .with_severity(ErrorSeverity::Warning);
        
        assert_eq!(context.component, "TestComponent");
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.details, Some("test details".to_string()));
        assert_eq!(context.severity, ErrorSeverity::Warning);
    }

    #[test]
    fn test_error_formatter() {
        let error = AppError::Config(ConfigError::FileNotFound);
        let user_msg = ErrorFormatter::format_user_message(&error);
        assert_eq!(user_msg, "配置文件不存在，将使用默认配置");
        
        let is_recoverable = ErrorFormatter::is_recoverable(&error);
        assert!(is_recoverable);
        
        let suggestion = ErrorFormatter::get_suggestion(&error);
        assert_eq!(suggestion, Some("程序将自动创建默认配置文件".to_string()));
    }

    #[test]
    fn test_recovery_strategy() {
        let window_error = AppError::Window(WindowError::WindowNotFound);
        let strategy = RecoveryStrategy::for_error(&window_error);
        
        match strategy {
            RecoveryStrategy::Retry { max_attempts, delay_ms } => {
                assert_eq!(max_attempts, 5);
                assert_eq!(delay_ms, 1000);
            }
            _ => panic!("Expected Retry strategy"),
        }
    }

    #[test]
    fn test_error_creation_helpers() {
        let config_error = ConfigError::validation_error("invalid value");
        assert!(matches!(config_error, ConfigError::ValidationError(_)));
        
        let window_error = WindowError::capture_error("screenshot failed");
        assert!(matches!(window_error, WindowError::CaptureError(_)));
        
        let action_error = ActionError::key_send_error("key not found");
        assert!(matches!(action_error, ActionError::KeySendError(_)));
        
        let vision_error = VisionError::low_confidence(0.3);
        assert!(matches!(vision_error, VisionError::LowConfidence(_)));
    }

    #[test]
    fn test_result_ext() {
        let result: AppResult<i32> = Ok(42);
        let logged_result = result.log_error();
        assert!(logged_result.is_ok());
        
        let error_result: AppResult<i32> = Err(AppError::ui_error("test error"));
        let user_result = error_result.to_user_error();
        assert!(user_result.is_err());
    }
}