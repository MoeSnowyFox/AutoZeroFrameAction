//! 状态管理数据模型

use serde::{Deserialize, Serialize};

/// 程序运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgramState {
    /// 已停止
    Stopped,
    /// 启动中
    Starting,
    /// 运行中
    Running,
    /// 停止中
    Stopping,
}

impl std::fmt::Display for ProgramState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramState::Stopped => write!(f, "已停止"),
            ProgramState::Starting => write!(f, "启动中"),
            ProgramState::Running => write!(f, "运行中"),
            ProgramState::Stopping => write!(f, "停止中"),
        }
    }
}

/// 游戏状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    /// 未检测到游戏
    NotDetected,
    /// 已检测到游戏
    Detected,
    /// 游戏中（战斗状态）
    InBattle,
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameState::NotDetected => write!(f, "未启动游戏"),
            GameState::Detected => write!(f, "已启动游戏"),
            GameState::InBattle => write!(f, "战斗中"),
        }
    }
}

/// 应用程序整体状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    /// 程序运行状态
    pub program_state: ProgramState,
    /// 游戏状态
    pub game_state: GameState,
    /// 最后更新时间（使用SystemTime以支持序列化）
    #[serde(with = "system_time_serde")]
    pub last_updated: std::time::SystemTime,
}

// 自定义SystemTime的序列化/反序列化
mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap();
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            program_state: ProgramState::Stopped,
            game_state: GameState::NotDetected,
            last_updated: std::time::SystemTime::now(),
        }
    }
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 更新程序状态
    pub fn update_program_state(&mut self, state: ProgramState) {
        self.program_state = state;
        self.last_updated = std::time::SystemTime::now();
    }
    
    /// 更新游戏状态
    pub fn update_game_state(&mut self, state: GameState) {
        self.game_state = state;
        self.last_updated = std::time::SystemTime::now();
    }
    
    /// 检查是否可以启动核心功能
    pub fn can_start_core(&self) -> bool {
        matches!(self.program_state, ProgramState::Stopped) 
            && matches!(self.game_state, GameState::Detected | GameState::InBattle)
    }
    
    /// 检查是否可以停止核心功能
    pub fn can_stop_core(&self) -> bool {
        matches!(self.program_state, ProgramState::Running)
    }
    
    /// 检查配置是否应该被禁用
    pub fn should_disable_config(&self) -> bool {
        !matches!(self.program_state, ProgramState::Stopped)
    }
}