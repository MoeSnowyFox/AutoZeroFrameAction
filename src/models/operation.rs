//! 操作相关数据模型

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    /// 按键操作
    KeyPress(String),
    /// 鼠标移动
    MouseMove(i32, i32),
    /// 鼠标点击
    MouseClick(MouseButton, i32, i32),
    /// 等待
    Wait(Duration),
}

/// 鼠标按钮
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MouseButton {
    /// 左键
    Left,
    /// 右键
    Right,
    /// 中键
    Middle,
}

/// 操作序列
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionSequence {
    /// 操作列表
    pub actions: Vec<ActionType>,
    /// 序列名称
    pub name: String,
    /// 描述
    pub description: Option<String>,
}

impl ActionSequence {
    /// 创建新的操作序列
    pub fn new(name: String) -> Self {
        Self {
            actions: Vec::new(),
            name,
            description: None,
        }
    }
    
    /// 添加操作
    pub fn add_action(&mut self, action: ActionType) {
        self.actions.push(action);
    }
    
    /// 添加按键操作
    pub fn add_key_press(&mut self, key: String) {
        self.actions.push(ActionType::KeyPress(key));
    }
    
    /// 添加鼠标点击
    pub fn add_mouse_click(&mut self, button: MouseButton, x: i32, y: i32) {
        self.actions.push(ActionType::MouseClick(button, x, y));
    }
    
    /// 添加等待
    pub fn add_wait(&mut self, duration: Duration) {
        self.actions.push(ActionType::Wait(duration));
    }
    
    /// 获取操作数量
    pub fn len(&self) -> usize {
        self.actions.len()
    }
    
    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

/// 游戏操作定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameOperation {
    /// 操作名称
    pub name: String,
    /// 触发热键
    pub hotkey: String,
    /// 游戏内按键
    pub game_key: String,
    /// 操作序列
    pub sequence: ActionSequence,
    /// 是否启用
    pub enabled: bool,
}

impl GameOperation {
    /// 创建新的游戏操作
    pub fn new(name: String, hotkey: String, game_key: String) -> Self {
        Self {
            name: name.clone(),
            hotkey,
            game_key,
            sequence: ActionSequence::new(name),
            enabled: true,
        }
    }
}

/// 预定义的游戏操作
pub struct DefaultOperations;

impl DefaultOperations {
    /// 获取默认的游戏操作列表
    pub fn get_default_operations() -> Vec<GameOperation> {
        vec![
            Self::deploy_operator(),
            Self::activate_skill(),
            Self::retreat_operator(),
            Self::focus_view(),
            Self::pause_game(),
        ]
    }
    
    /// 拖出干员操作
    pub fn deploy_operator() -> GameOperation {
        let mut operation = GameOperation::new(
            "deploy_operator".to_string(),
            "1".to_string(),
            "1".to_string(),
        );
        
        // 添加默认的拖出干员操作序列
        operation.sequence.add_key_press("1".to_string());
        operation.sequence.add_wait(Duration::from_millis(100));
        
        operation
    }
    
    /// 开启技能操作
    pub fn activate_skill() -> GameOperation {
        let mut operation = GameOperation::new(
            "activate_skill".to_string(),
            "2".to_string(),
            "Space".to_string(),
        );
        
        operation.sequence.add_key_press("Space".to_string());
        operation.sequence.add_wait(Duration::from_millis(50));
        
        operation
    }
    
    /// 撤退干员操作
    pub fn retreat_operator() -> GameOperation {
        let mut operation = GameOperation::new(
            "retreat_operator".to_string(),
            "3".to_string(),
            "Delete".to_string(),
        );
        
        operation.sequence.add_key_press("Delete".to_string());
        operation.sequence.add_wait(Duration::from_millis(100));
        
        operation
    }
    
    /// 视角聚焦操作
    pub fn focus_view() -> GameOperation {
        let mut operation = GameOperation::new(
            "focus_view".to_string(),
            "4".to_string(),
            "F".to_string(),
        );
        
        operation.sequence.add_key_press("F".to_string());
        operation.sequence.add_wait(Duration::from_millis(50));
        
        operation
    }
    
    /// 暂停游戏操作
    pub fn pause_game() -> GameOperation {
        let mut operation = GameOperation::new(
            "pause_game".to_string(),
            "Space".to_string(),
            "Escape".to_string(),
        );
        
        operation.sequence.add_key_press("Escape".to_string());
        operation.sequence.add_wait(Duration::from_millis(100));
        
        operation
    }
}