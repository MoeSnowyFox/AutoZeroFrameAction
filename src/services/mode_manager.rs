//! 模式管理器
//! 
//! 负责管理宏模式和智能模式的切换，以及各模式特定的配置和行为

use crate::models::{OperationMode, MacroModeConfig, IntelligentModeConfig, GameOperation, DefaultOperations};
use crate::utils::{ModeResult};
use std::collections::HashMap;
use tokio::sync::broadcast;
use log::{info, debug, warn};

/// 模式配置trait
pub trait ModeConfig: Send + Sync {
    fn get_hotkeys(&self) -> &HashMap<String, String>;
    fn set_hotkey(&mut self, operation: String, hotkey: String);
    fn validate_config(&self) -> ModeResult<()>;
    fn get_enabled_features(&self) -> Vec<String>;
}

impl ModeConfig for MacroModeConfig {
    fn get_hotkeys(&self) -> &HashMap<String, String> {
        &self.hotkeys
    }
    
    fn set_hotkey(&mut self, operation: String, hotkey: String) {
        self.hotkeys.insert(operation, hotkey);
    }
    
    fn validate_config(&self) -> ModeResult<()> {
        // 验证宏模式配置
        if self.hotkeys.is_empty() {
            warn!("宏模式热键配置为空");
        }
        
        // 检查必要的热键是否存在
        let required_keys = ["deploy_operator", "activate_skill", "retreat_operator"];
        for key in &required_keys {
            if !self.hotkeys.contains_key(*key) {
                warn!("缺少必要的热键配置: {}", key);
            }
        }
        
        Ok(())
    }
    
    fn get_enabled_features(&self) -> Vec<String> {
        let mut features = Vec::new();
        
        if self.battle_detection_enabled {
            features.push("battle_detection".to_string());
        }
        
        if self.overlay_settings.enabled {
            features.push("overlay".to_string());
        }
        
        features
    }
}

impl ModeConfig for IntelligentModeConfig {
    fn get_hotkeys(&self) -> &HashMap<String, String> {
        &self.hotkeys
    }
    
    fn set_hotkey(&mut self, operation: String, hotkey: String) {
        self.hotkeys.insert(operation, hotkey);
    }
    
    fn validate_config(&self) -> ModeResult<()> {
        // 验证智能模式配置
        if self.hotkeys.is_empty() {
            warn!("智能模式热键配置为空");
        }
        
        // 检查智能功能配置
        if self.intelligent_features.is_empty() {
            warn!("智能模式未启用任何智能功能");
        }
        
        Ok(())
    }
    
    fn get_enabled_features(&self) -> Vec<String> {
        let mut features = Vec::new();
        
        // 检查智能功能列表中的功能
        for feature in &self.intelligent_features {
            features.push(feature.clone());
        }
        
        if self.overlay_settings.enabled {
            features.push("overlay".to_string());
        }
        
        features
    }
}

/// 模式变更事件
#[derive(Debug, Clone)]
pub enum ModeChangeEvent {
    /// 模式切换
    ModeSwitch {
        from: OperationMode,
        to: OperationMode,
    },
    /// 配置更新
    ConfigUpdate {
        mode: OperationMode,
        config_type: String,
    },
    /// 热键更新
    HotkeyUpdate {
        mode: OperationMode,
        operation: String,
        old_hotkey: Option<String>,
        new_hotkey: String,
    },
}

/// 模式管理器
pub struct ModeManager {
    /// 当前模式
    current_mode: OperationMode,
    /// 宏模式配置
    macro_config: MacroModeConfig,
    /// 智能模式配置
    intelligent_config: IntelligentModeConfig,
    /// 游戏操作定义
    game_operations: HashMap<String, GameOperation>,
    /// 事件广播器
    event_sender: broadcast::Sender<ModeChangeEvent>,
    /// 模式切换历史
    mode_history: Vec<(OperationMode, std::time::Instant)>,
    /// 最大历史记录数
    max_history: usize,
}

impl ModeManager {
    /// 创建新的模式管理器
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        
        let mut manager = Self {
            current_mode: OperationMode::Macro,
            macro_config: MacroModeConfig::default(),
            intelligent_config: IntelligentModeConfig::default(),
            game_operations: HashMap::new(),
            event_sender,
            mode_history: Vec::new(),
            max_history: 50,
        };
        
        // 初始化默认游戏操作
        manager.initialize_default_operations();
        
        // 记录初始模式
        manager.mode_history.push((OperationMode::Macro, std::time::Instant::now()));
        
        manager
    }
    
    /// 使用自定义配置创建模式管理器
    pub fn with_configs(
        macro_config: MacroModeConfig,
        intelligent_config: IntelligentModeConfig,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        
        let mut manager = Self {
            current_mode: OperationMode::Macro,
            macro_config,
            intelligent_config,
            game_operations: HashMap::new(),
            event_sender,
            mode_history: Vec::new(),
            max_history: 50,
        };
        
        manager.initialize_default_operations();
        manager.mode_history.push((OperationMode::Macro, std::time::Instant::now()));
        
        manager
    }
    
    /// 切换模式
    pub fn switch_mode(&mut self, mode: OperationMode) -> ModeResult<()> {
        if self.current_mode == mode {
            debug!("模式已经是 {:?}，无需切换", mode);
            return Ok(());
        }
        
        let old_mode = self.current_mode;
        
        // 验证目标模式配置
        match mode {
            OperationMode::Macro => {
                self.macro_config.validate_config()?;
            }
            OperationMode::Intelligent => {
                self.intelligent_config.validate_config()?;
            }
        }
        
        info!("切换模式: {:?} -> {:?}", old_mode, mode);
        
        // 执行模式切换
        self.current_mode = mode;
        
        // 记录模式切换历史
        self.add_to_history(mode);
        
        // 发送模式切换事件
        let event = ModeChangeEvent::ModeSwitch {
            from: old_mode,
            to: mode,
        };
        
        if let Err(e) = self.event_sender.send(event) {
            warn!("发送模式切换事件失败: {}", e);
        }
        
        info!("模式切换完成: {:?}", mode);
        Ok(())
    }
    
    /// 获取当前模式
    pub fn get_current_mode(&self) -> &OperationMode {
        &self.current_mode
    }
    
    /// 获取宏模式配置
    pub fn get_macro_config(&self) -> &MacroModeConfig {
        &self.macro_config
    }
    
    /// 获取宏模式配置（可变）
    pub fn get_macro_config_mut(&mut self) -> &mut MacroModeConfig {
        &mut self.macro_config
    }
    
    /// 获取智能模式配置
    pub fn get_intelligent_config(&self) -> &IntelligentModeConfig {
        &self.intelligent_config
    }
    
    /// 获取智能模式配置（可变）
    pub fn get_intelligent_config_mut(&mut self) -> &mut IntelligentModeConfig {
        &mut self.intelligent_config
    }
    
    /// 获取当前模式的热键配置
    pub fn get_current_hotkeys(&self) -> &HashMap<String, String> {
        match self.current_mode {
            OperationMode::Macro => &self.macro_config.hotkeys,
            OperationMode::Intelligent => &self.intelligent_config.hotkeys,
        }
    }
    
    /// 更新当前模式的热键
    pub fn update_hotkey(&mut self, operation: String, hotkey: String) -> ModeResult<()> {
        let old_hotkey = match self.current_mode {
            OperationMode::Macro => {
                let old = self.macro_config.hotkeys.get(&operation).cloned();
                self.macro_config.set_hotkey(operation.clone(), hotkey.clone());
                old
            }
            OperationMode::Intelligent => {
                let old = self.intelligent_config.hotkeys.get(&operation).cloned();
                self.intelligent_config.set_hotkey(operation.clone(), hotkey.clone());
                old
            }
        };
        
        // 发送热键更新事件
        let event = ModeChangeEvent::HotkeyUpdate {
            mode: self.current_mode,
            operation: operation.clone(),
            old_hotkey,
            new_hotkey: hotkey,
        };
        
        if let Err(e) = self.event_sender.send(event) {
            warn!("发送热键更新事件失败: {}", e);
        }
        
        debug!("更新热键: {} -> {}", operation, self.get_current_hotkeys().get(&operation).unwrap_or(&"未设置".to_string()));
        Ok(())
    }
    
    /// 获取当前模式的启用功能
    pub fn get_current_enabled_features(&self) -> Vec<String> {
        match self.current_mode {
            OperationMode::Macro => self.macro_config.get_enabled_features(),
            OperationMode::Intelligent => self.intelligent_config.get_enabled_features(),
        }
    }
    
    /// 验证当前模式配置
    pub fn validate_current_config(&self) -> ModeResult<()> {
        match self.current_mode {
            OperationMode::Macro => self.macro_config.validate_config(),
            OperationMode::Intelligent => self.intelligent_config.validate_config(),
        }
    }
    
    /// 获取游戏操作定义
    pub fn get_game_operation(&self, name: &str) -> Option<&GameOperation> {
        self.game_operations.get(name)
    }
    
    /// 获取所有游戏操作
    pub fn get_all_game_operations(&self) -> &HashMap<String, GameOperation> {
        &self.game_operations
    }
    
    /// 添加或更新游戏操作
    pub fn set_game_operation(&mut self, operation: GameOperation) {
        let name = operation.name.clone();
        self.game_operations.insert(name, operation);
        debug!("更新游戏操作定义");
    }
    
    /// 订阅模式变更事件
    pub fn subscribe_events(&self) -> broadcast::Receiver<ModeChangeEvent> {
        self.event_sender.subscribe()
    }
    
    /// 获取模式切换历史
    pub fn get_mode_history(&self) -> &[(OperationMode, std::time::Instant)] {
        &self.mode_history
    }
    
    /// 获取模式统计信息
    pub fn get_mode_stats(&self) -> HashMap<OperationMode, (usize, std::time::Duration)> {
        let mut stats = HashMap::new();
        let mut current_mode_start = None;
        let now = std::time::Instant::now();
        
        for (i, (mode, timestamp)) in self.mode_history.iter().enumerate() {
            let entry = stats.entry(*mode).or_insert((0, std::time::Duration::ZERO));
            entry.0 += 1; // 计数
            
            if i > 0 {
                let duration = timestamp.duration_since(self.mode_history[i - 1].1);
                entry.1 += duration;
            }
            
            if i == self.mode_history.len() - 1 {
                current_mode_start = Some(*timestamp);
            }
        }
        
        // 添加当前模式的持续时间
        if let Some(start_time) = current_mode_start {
            if let Some(entry) = stats.get_mut(&self.current_mode) {
                entry.1 += now.duration_since(start_time);
            }
        }
        
        stats
    }
    
    /// 重置为默认配置
    pub fn reset_to_defaults(&mut self) -> ModeResult<()> {
        info!("重置模式管理器为默认配置");
        
        self.macro_config = MacroModeConfig::default();
        self.intelligent_config = IntelligentModeConfig::default();
        self.current_mode = OperationMode::Macro;
        
        // 重新初始化游戏操作
        self.game_operations.clear();
        self.initialize_default_operations();
        
        // 清空历史记录
        self.mode_history.clear();
        self.add_to_history(OperationMode::Macro);
        
        // 发送配置更新事件
        let event = ModeChangeEvent::ConfigUpdate {
            mode: self.current_mode,
            config_type: "reset_to_defaults".to_string(),
        };
        
        if let Err(e) = self.event_sender.send(event) {
            warn!("发送配置重置事件失败: {}", e);
        }
        
        Ok(())
    }
    
    /// 初始化默认游戏操作
    fn initialize_default_operations(&mut self) {
        let default_operations = DefaultOperations::get_default_operations();
        
        for operation in default_operations {
            self.game_operations.insert(operation.name.clone(), operation);
        }
        
        debug!("初始化了 {} 个默认游戏操作", self.game_operations.len());
    }
    
    /// 添加到历史记录
    fn add_to_history(&mut self, mode: OperationMode) {
        self.mode_history.push((mode, std::time::Instant::now()));
        
        // 限制历史记录数量
        if self.mode_history.len() > self.max_history {
            self.mode_history.remove(0);
        }
    }
    
    /// 设置最大历史记录数
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        
        // 如果当前历史记录超过限制，则截断
        if self.mode_history.len() > max {
            let excess = self.mode_history.len() - max;
            self.mode_history.drain(0..excess);
        }
    }
    
    /// 检查是否可以切换到指定模式
    pub fn can_switch_to(&self, mode: OperationMode) -> bool {
        // 检查配置是否有效
        let config_valid = match mode {
            OperationMode::Macro => self.macro_config.validate_config().is_ok(),
            OperationMode::Intelligent => self.intelligent_config.validate_config().is_ok(),
        };
        
        config_valid
    }
    
    /// 获取模式描述
    pub fn get_mode_description(&self, mode: OperationMode) -> String {
        match mode {
            OperationMode::Macro => {
                let features = self.macro_config.get_enabled_features();
                format!("宏模式 - 启用功能: {}", features.join(", "))
            }
            OperationMode::Intelligent => {
                let features = self.intelligent_config.get_enabled_features();
                format!("智能模式 - 启用功能: {}", features.join(", "))
            }
        }
    }
}

impl Default for ModeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_manager_creation() {
        let manager = ModeManager::new();
        assert_eq!(*manager.get_current_mode(), OperationMode::Macro);
        assert!(!manager.get_all_game_operations().is_empty());
    }

    #[test]
    fn test_mode_switching() {
        let mut manager = ModeManager::new();
        
        assert_eq!(*manager.get_current_mode(), OperationMode::Macro);
        
        manager.switch_mode(OperationMode::Intelligent).unwrap();
        assert_eq!(*manager.get_current_mode(), OperationMode::Intelligent);
        
        manager.switch_mode(OperationMode::Macro).unwrap();
        assert_eq!(*manager.get_current_mode(), OperationMode::Macro);
    }

    #[test]
    fn test_hotkey_updates() {
        let mut manager = ModeManager::new();
        
        manager.update_hotkey("test_operation".to_string(), "Ctrl+T".to_string()).unwrap();
        
        let hotkeys = manager.get_current_hotkeys();
        assert_eq!(hotkeys.get("test_operation"), Some(&"Ctrl+T".to_string()));
    }

    #[test]
    fn test_mode_history() {
        let mut manager = ModeManager::new();
        
        manager.switch_mode(OperationMode::Intelligent).unwrap();
        manager.switch_mode(OperationMode::Macro).unwrap();
        
        let history = manager.get_mode_history();
        assert_eq!(history.len(), 3); // 初始 + 2次切换
    }

    #[test]
    fn test_config_validation() {
        let manager = ModeManager::new();
        
        // 默认配置应该是有效的
        assert!(manager.validate_current_config().is_ok());
        assert!(manager.can_switch_to(OperationMode::Intelligent));
    }

    #[test]
    fn test_game_operations() {
        let mut manager = ModeManager::new();
        
        let operation = GameOperation::new(
            "test_op".to_string(),
            "T".to_string(),
            "T".to_string(),
        );
        
        manager.set_game_operation(operation);
        
        let retrieved = manager.get_game_operation("test_op");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_op");
    }

    #[test]
    fn test_enabled_features() {
        let manager = ModeManager::new();
        
        let features = manager.get_current_enabled_features();
        assert!(!features.is_empty()); // 默认配置应该有一些启用的功能
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut manager = ModeManager::new();
        
        // 修改一些配置
        manager.switch_mode(OperationMode::Intelligent).unwrap();
        manager.update_hotkey("test".to_string(), "X".to_string()).unwrap();
        
        // 重置
        manager.reset_to_defaults().unwrap();
        
        assert_eq!(*manager.get_current_mode(), OperationMode::Macro);
        assert!(!manager.get_current_hotkeys().contains_key("test"));
    }
}