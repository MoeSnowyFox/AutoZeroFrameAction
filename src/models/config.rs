//! 配置数据模型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::utils::error::ConfigError;

/// 应用程序主配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// 当前工作模式
    pub mode: OperationMode,
    /// 宏模式配置
    pub macro_config: MacroModeConfig,
    /// 智能模式配置
    pub intelligent_config: IntelligentModeConfig,
    /// 全局设置
    pub global_settings: GlobalSettings,
    /// UI设置
    pub ui_settings: UISettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mode: OperationMode::Macro,
            macro_config: MacroModeConfig::default(),
            intelligent_config: IntelligentModeConfig::default(),
            global_settings: GlobalSettings::default(),
            ui_settings: UISettings::default(),
        }
    }
}

impl AppConfig {
    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 验证宏模式配置
        self.macro_config.validate()
            .map_err(|e| ConfigError::ValidationError(format!("宏模式配置无效: {}", e)))?;
        
        // 验证智能模式配置
        self.intelligent_config.validate()
            .map_err(|e| ConfigError::ValidationError(format!("智能模式配置无效: {}", e)))?;
        
        // 验证全局设置
        self.global_settings.validate()
            .map_err(|e| ConfigError::ValidationError(format!("全局设置无效: {}", e)))?;
        
        // 验证UI设置
        self.ui_settings.validate()
            .map_err(|e| ConfigError::ValidationError(format!("UI设置无效: {}", e)))?;
        
        Ok(())
    }
    
    /// 修复无效的配置项，使用默认值替换
    pub fn fix_invalid_values(&mut self) {
        self.macro_config.fix_invalid_values();
        self.intelligent_config.fix_invalid_values();
        self.global_settings.fix_invalid_values();
        self.ui_settings.fix_invalid_values();
    }
    
    /// 获取所有支持的操作名称
    pub fn get_supported_operations() -> Vec<&'static str> {
        vec![
            "deploy_operator",    // 拖出干员
            "activate_skill",     // 开启技能
            "retreat_operator",   // 撤退干员
            "focus_view",         // 视角聚焦
            "pause_game",         // 暂停游戏
        ]
    }
    
    /// 获取所有支持的游戏内功能
    pub fn get_supported_game_functions() -> Vec<&'static str> {
        vec![
            "battle_speed",       // 战斗内变速
            "skill_activation",   // 释放技能
            "retreat_operator",   // 撤退干员
            "exit_return",        // 退出返回
        ]
    }
}

/// 操作模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationMode {
    /// 宏模式 - 执行预设操作序列
    Macro,
    /// 智能模式 - 基于图像识别的智能操作
    Intelligent,
}

/// 宏模式配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroModeConfig {
    /// 按键配置 (操作名 -> 监听按键)
    pub hotkeys: HashMap<String, String>,
    /// 悬浮窗设置
    pub overlay_settings: OverlaySettings,
    /// 是否启用战斗状态检测
    pub battle_detection_enabled: bool,
}

impl Default for MacroModeConfig {
    fn default() -> Self {
        let mut hotkeys = HashMap::new();
        hotkeys.insert("deploy_operator".to_string(), "1".to_string());
        hotkeys.insert("activate_skill".to_string(), "2".to_string());
        hotkeys.insert("retreat_operator".to_string(), "3".to_string());
        hotkeys.insert("focus_view".to_string(), "4".to_string());
        hotkeys.insert("pause_game".to_string(), "Space".to_string());
        
        Self {
            hotkeys,
            overlay_settings: OverlaySettings::default(),
            battle_detection_enabled: true,
        }
    }
}

impl MacroModeConfig {
    /// 验证宏模式配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证必需的按键配置是否存在
        let required_operations = AppConfig::get_supported_operations();
        for operation in required_operations {
            if !self.hotkeys.contains_key(operation) {
                return Err(format!("缺少必需的按键配置: {}", operation));
            }
            
            let hotkey = &self.hotkeys[operation];
            if hotkey.trim().is_empty() {
                return Err(format!("按键配置不能为空: {}", operation));
            }
        }
        
        // 验证悬浮窗设置
        self.overlay_settings.validate()?;
        
        Ok(())
    }
    
    /// 修复无效的配置项
    pub fn fix_invalid_values(&mut self) {
        // 确保所有必需的按键配置都存在
        let default_config = MacroModeConfig::default();
        for operation in AppConfig::get_supported_operations() {
            if !self.hotkeys.contains_key(operation) || self.hotkeys[operation].trim().is_empty() {
                if let Some(default_key) = default_config.hotkeys.get(operation) {
                    self.hotkeys.insert(operation.to_string(), default_key.clone());
                }
            }
        }
        
        // 修复悬浮窗设置
        self.overlay_settings.fix_invalid_values();
    }
}

/// 智能模式配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntelligentModeConfig {
    /// 按键配置 (操作名 -> 监听按键)
    pub hotkeys: HashMap<String, String>,
    /// 悬浮窗设置
    pub overlay_settings: OverlaySettings,
    /// 启用的智能功能列表
    pub intelligent_features: Vec<String>,
}

impl Default for IntelligentModeConfig {
    fn default() -> Self {
        let mut hotkeys = HashMap::new();
        hotkeys.insert("deploy_operator".to_string(), "1".to_string());
        hotkeys.insert("activate_skill".to_string(), "2".to_string());
        hotkeys.insert("retreat_operator".to_string(), "3".to_string());
        hotkeys.insert("focus_view".to_string(), "4".to_string());
        hotkeys.insert("pause_game".to_string(), "Space".to_string());
        
        let mut features = Vec::new();
        features.push("small_number_selection".to_string());
        
        Self {
            hotkeys,
            overlay_settings: OverlaySettings::default(),
            intelligent_features: features,
        }
    }
}

impl IntelligentModeConfig {
    /// 验证智能模式配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证必需的按键配置是否存在
        let required_operations = AppConfig::get_supported_operations();
        for operation in required_operations {
            if !self.hotkeys.contains_key(operation) {
                return Err(format!("缺少必需的按键配置: {}", operation));
            }
            
            let hotkey = &self.hotkeys[operation];
            if hotkey.trim().is_empty() {
                return Err(format!("按键配置不能为空: {}", operation));
            }
        }
        
        // 验证智能功能配置
        let supported_features = Self::get_supported_intelligent_features();
        for feature in &self.intelligent_features {
            if !supported_features.contains(&feature.as_str()) {
                return Err(format!("不支持的智能功能: {}", feature));
            }
        }
        
        // 验证悬浮窗设置
        self.overlay_settings.validate()?;
        
        Ok(())
    }
    
    /// 修复无效的配置项
    pub fn fix_invalid_values(&mut self) {
        // 确保所有必需的按键配置都存在
        let default_config = IntelligentModeConfig::default();
        for operation in AppConfig::get_supported_operations() {
            if !self.hotkeys.contains_key(operation) || self.hotkeys[operation].trim().is_empty() {
                if let Some(default_key) = default_config.hotkeys.get(operation) {
                    self.hotkeys.insert(operation.to_string(), default_key.clone());
                }
            }
        }
        
        // 移除不支持的智能功能
        let supported_features = Self::get_supported_intelligent_features();
        self.intelligent_features.retain(|feature| supported_features.contains(&feature.as_str()));
        
        // 如果没有启用任何智能功能，添加默认功能
        if self.intelligent_features.is_empty() {
            self.intelligent_features.push("small_number_selection".to_string());
        }
        
        // 修复悬浮窗设置
        self.overlay_settings.fix_invalid_values();
    }
    
    /// 获取所有支持的智能功能
    pub fn get_supported_intelligent_features() -> Vec<&'static str> {
        vec![
            "small_number_selection",  // 小数字选择干员
            "auto_skill_timing",       // 自动技能时机
            "smart_deployment",        // 智能部署位置
        ]
    }
}

/// 全局设置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalSettings {
    /// 游戏内按键配置 (功能名 -> 游戏内按键)
    pub game_keys: HashMap<String, String>,
    /// 识别到窗口后立即运行
    pub auto_start_on_detection: bool,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        let mut game_keys = HashMap::new();
        game_keys.insert("battle_speed".to_string(), "2".to_string());
        game_keys.insert("skill_activation".to_string(), "Space".to_string());
        game_keys.insert("retreat_operator".to_string(), "Delete".to_string());
        game_keys.insert("exit_return".to_string(), "Escape".to_string());
        
        Self {
            game_keys,
            auto_start_on_detection: false,
        }
    }
}

impl GlobalSettings {
    /// 验证全局设置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证必需的游戏内按键配置是否存在
        let required_functions = AppConfig::get_supported_game_functions();
        for function in required_functions {
            if !self.game_keys.contains_key(function) {
                return Err(format!("缺少必需的游戏内按键配置: {}", function));
            }
            
            let key = &self.game_keys[function];
            if key.trim().is_empty() {
                return Err(format!("游戏内按键配置不能为空: {}", function));
            }
        }
        
        Ok(())
    }
    
    /// 修复无效的配置项
    pub fn fix_invalid_values(&mut self) {
        // 确保所有必需的游戏内按键配置都存在
        let default_config = GlobalSettings::default();
        for function in AppConfig::get_supported_game_functions() {
            if !self.game_keys.contains_key(function) || self.game_keys[function].trim().is_empty() {
                if let Some(default_key) = default_config.game_keys.get(function) {
                    self.game_keys.insert(function.to_string(), default_key.clone());
                }
            }
        }
    }
}

/// UI设置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UISettings {
    /// 主题
    pub theme: Theme,
    /// 语言
    pub language: String,
    /// 窗口大小
    pub window_size: (u32, u32),
    /// 最小化到托盘
    #[serde(default = "default_minimize_to_tray")]
    pub minimize_to_tray: bool,
    /// 开机自启动
    #[serde(default = "default_start_with_windows")]
    pub start_with_windows: bool,
    /// 自动检查更新
    #[serde(default = "default_auto_check_updates")]
    pub auto_check_updates: bool,
}

fn default_minimize_to_tray() -> bool {
    true
}

fn default_start_with_windows() -> bool {
    false
}

fn default_auto_check_updates() -> bool {
    true
}

impl Default for UISettings {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            language: "zh-CN".to_string(),
            window_size: (800, 600),
            minimize_to_tray: true,
            start_with_windows: false,
            auto_check_updates: true,
        }
    }
}

impl UISettings {
    /// 验证UI设置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证窗口大小
        if self.window_size.0 < 400 || self.window_size.0 > 3840 {
            return Err(format!("窗口宽度无效: {}, 应在400-3840之间", self.window_size.0));
        }
        
        if self.window_size.1 < 300 || self.window_size.1 > 2160 {
            return Err(format!("窗口高度无效: {}, 应在300-2160之间", self.window_size.1));
        }
        
        // 验证语言代码
        let supported_languages = Self::get_supported_languages();
        if !supported_languages.contains(&self.language.as_str()) {
            return Err(format!("不支持的语言: {}", self.language));
        }
        
        Ok(())
    }
    
    /// 修复无效的配置项
    pub fn fix_invalid_values(&mut self) {
        // 修复窗口大小
        if self.window_size.0 < 400 {
            self.window_size.0 = 400;
        } else if self.window_size.0 > 3840 {
            self.window_size.0 = 3840;
        }
        
        if self.window_size.1 < 300 {
            self.window_size.1 = 300;
        } else if self.window_size.1 > 2160 {
            self.window_size.1 = 2160;
        }
        
        // 修复语言设置
        let supported_languages = Self::get_supported_languages();
        if !supported_languages.contains(&self.language.as_str()) {
            self.language = "zh-CN".to_string();
        }
    }
    
    /// 获取所有支持的语言
    pub fn get_supported_languages() -> Vec<&'static str> {
        vec![
            "zh-CN",  // 简体中文
            "zh-TW",  // 繁体中文
            "en-US",  // 英语
            "ja-JP",  // 日语
            "ko-KR",  // 韩语
        ]
    }
}

/// 悬浮窗设置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverlaySettings {
    /// 是否启用悬浮窗
    pub enabled: bool,
    /// 显示模式
    pub display_mode: OverlayDisplayMode,
    /// 透明度 (0-100)
    pub transparency: u8,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            display_mode: OverlayDisplayMode::WhenForeground,
            transparency: 80,
        }
    }
}

impl OverlaySettings {
    /// 验证悬浮窗设置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证透明度值范围
        if self.transparency > 100 {
            return Err(format!("透明度值无效: {}, 应在0-100之间", self.transparency));
        }
        
        Ok(())
    }
    
    /// 修复无效的配置项
    pub fn fix_invalid_values(&mut self) {
        // 修复透明度值
        if self.transparency > 100 {
            self.transparency = 100;
        }
    }
}

/// 悬浮窗显示模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OverlayDisplayMode {
    /// 总是显示
    Always,
    /// 程序在前台时显示
    WhenForeground,
    /// 仅显示于程序上层
    OnlyAboveProgram,
}

/// 主题
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    /// 明亮主题
    Light,
    /// 暗黑主题
    Dark,
    /// 跟随系统
    Auto,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        
        // 验证默认值
        assert_eq!(config.mode, OperationMode::Macro);
        assert!(config.macro_config.battle_detection_enabled);
        assert!(config.macro_config.overlay_settings.enabled);
        assert_eq!(config.macro_config.overlay_settings.transparency, 80);
        assert_eq!(config.ui_settings.theme, Theme::Light);
        assert_eq!(config.ui_settings.language, "zh-CN");
        assert_eq!(config.ui_settings.window_size, (800, 600));
        assert!(!config.global_settings.auto_start_on_detection);
    }

    #[test]
    fn test_app_config_validation() {
        let mut config = AppConfig::default();
        
        // 默认配置应该是有效的
        assert!(config.validate().is_ok());
        
        // 测试无效的透明度值
        config.macro_config.overlay_settings.transparency = 150;
        assert!(config.validate().is_err());
        
        // 修复无效值
        config.fix_invalid_values();
        assert_eq!(config.macro_config.overlay_settings.transparency, 100);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_macro_mode_config_validation() {
        let mut config = MacroModeConfig::default();
        
        // 默认配置应该是有效的
        assert!(config.validate().is_ok());
        
        // 测试缺少必需的按键配置
        config.hotkeys.remove("deploy_operator");
        assert!(config.validate().is_err());
        
        // 修复缺失的配置
        config.fix_invalid_values();
        assert!(config.hotkeys.contains_key("deploy_operator"));
        assert!(config.validate().is_ok());
        
        // 测试空的按键配置
        config.hotkeys.insert("activate_skill".to_string(), "".to_string());
        assert!(config.validate().is_err());
        
        // 修复空的配置
        config.fix_invalid_values();
        assert!(!config.hotkeys["activate_skill"].is_empty());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_intelligent_mode_config_validation() {
        let mut config = IntelligentModeConfig::default();
        
        // 默认配置应该是有效的
        assert!(config.validate().is_ok());
        
        // 测试不支持的智能功能
        config.intelligent_features.push("unsupported_feature".to_string());
        assert!(config.validate().is_err());
        
        // 修复不支持的功能
        config.fix_invalid_values();
        assert!(!config.intelligent_features.contains(&"unsupported_feature".to_string()));
        assert!(config.validate().is_ok());
        
        // 测试空的智能功能列表
        config.intelligent_features.clear();
        config.fix_invalid_values();
        assert!(!config.intelligent_features.is_empty());
    }

    #[test]
    fn test_global_settings_validation() {
        let mut settings = GlobalSettings::default();
        
        // 默认设置应该是有效的
        assert!(settings.validate().is_ok());
        
        // 测试缺少必需的游戏按键配置
        settings.game_keys.remove("battle_speed");
        assert!(settings.validate().is_err());
        
        // 修复缺失的配置
        settings.fix_invalid_values();
        assert!(settings.game_keys.contains_key("battle_speed"));
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_ui_settings_validation() {
        let mut settings = UISettings::default();
        
        // 默认设置应该是有效的
        assert!(settings.validate().is_ok());
        
        // 测试无效的窗口大小
        settings.window_size = (100, 100); // 太小
        assert!(settings.validate().is_err());
        
        settings.window_size = (5000, 5000); // 太大
        assert!(settings.validate().is_err());
        
        // 修复无效的窗口大小
        settings.fix_invalid_values();
        assert!(settings.window_size.0 >= 400 && settings.window_size.0 <= 3840);
        assert!(settings.window_size.1 >= 300 && settings.window_size.1 <= 2160);
        assert!(settings.validate().is_ok());
        
        // 测试不支持的语言
        settings.language = "invalid-lang".to_string();
        assert!(settings.validate().is_err());
        
        // 修复不支持的语言
        settings.fix_invalid_values();
        assert_eq!(settings.language, "zh-CN");
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_overlay_settings_validation() {
        let mut settings = OverlaySettings::default();
        
        // 默认设置应该是有效的
        assert!(settings.validate().is_ok());
        
        // 测试无效的透明度值
        settings.transparency = 150;
        assert!(settings.validate().is_err());
        
        // 修复无效的透明度值
        settings.fix_invalid_values();
        assert_eq!(settings.transparency, 100);
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_supported_operations() {
        let operations = AppConfig::get_supported_operations();
        assert_eq!(operations.len(), 5);
        assert!(operations.contains(&"deploy_operator"));
        assert!(operations.contains(&"activate_skill"));
        assert!(operations.contains(&"retreat_operator"));
        assert!(operations.contains(&"focus_view"));
        assert!(operations.contains(&"pause_game"));
    }

    #[test]
    fn test_supported_game_functions() {
        let functions = AppConfig::get_supported_game_functions();
        assert_eq!(functions.len(), 4);
        assert!(functions.contains(&"battle_speed"));
        assert!(functions.contains(&"skill_activation"));
        assert!(functions.contains(&"retreat_operator"));
        assert!(functions.contains(&"exit_return"));
    }

    #[test]
    fn test_supported_intelligent_features() {
        let features = IntelligentModeConfig::get_supported_intelligent_features();
        assert_eq!(features.len(), 3);
        assert!(features.contains(&"small_number_selection"));
        assert!(features.contains(&"auto_skill_timing"));
        assert!(features.contains(&"smart_deployment"));
    }

    #[test]
    fn test_supported_languages() {
        let languages = UISettings::get_supported_languages();
        assert_eq!(languages.len(), 5);
        assert!(languages.contains(&"zh-CN"));
        assert!(languages.contains(&"zh-TW"));
        assert!(languages.contains(&"en-US"));
        assert!(languages.contains(&"ja-JP"));
        assert!(languages.contains(&"ko-KR"));
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        
        // 测试序列化
        let json = serde_json::to_string(&config).expect("序列化失败");
        assert!(!json.is_empty());
        
        // 测试反序列化
        let deserialized: AppConfig = serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_fix_comprehensive() {
        let mut config = AppConfig::default();
        
        // 故意破坏配置
        config.macro_config.hotkeys.clear();
        config.macro_config.overlay_settings.transparency = 200;
        config.intelligent_config.intelligent_features.clear();
        config.intelligent_config.intelligent_features.push("invalid_feature".to_string());
        config.global_settings.game_keys.clear();
        config.ui_settings.window_size = (50, 50);
        config.ui_settings.language = "invalid".to_string();
        
        // 验证配置确实无效
        assert!(config.validate().is_err());
        
        // 修复所有无效值
        config.fix_invalid_values();
        
        // 验证修复后的配置是有效的
        assert!(config.validate().is_ok());
        
        // 验证具体的修复结果
        assert_eq!(config.macro_config.hotkeys.len(), 5);
        assert_eq!(config.macro_config.overlay_settings.transparency, 100);
        assert!(!config.intelligent_config.intelligent_features.is_empty());
        assert!(!config.intelligent_config.intelligent_features.contains(&"invalid_feature".to_string()));
        assert_eq!(config.global_settings.game_keys.len(), 4);
        assert!(config.ui_settings.window_size.0 >= 400);
        assert!(config.ui_settings.window_size.1 >= 300);
        assert_eq!(config.ui_settings.language, "zh-CN");
    }
}

/// 简单的配置验证函数，用于手动测试
pub fn test_config_validation() -> Result<(), String> {
    println!("测试配置数据结构...");
    
    // 测试默认配置
    let config = AppConfig::default();
    println!("✓ 默认配置创建成功");
    
    // 测试配置验证
    config.validate().map_err(|e| format!("默认配置验证失败: {}", e))?;
    println!("✓ 默认配置验证通过");
    
    // 测试序列化
    let json = serde_json::to_string(&config).map_err(|e| format!("序列化失败: {}", e))?;
    println!("✓ 配置序列化成功，长度: {}", json.len());
    
    // 测试反序列化
    let deserialized: AppConfig = serde_json::from_str(&json).map_err(|e| format!("反序列化失败: {}", e))?;
    if config == deserialized {
        println!("✓ 配置反序列化成功且一致");
    } else {
        return Err("配置反序列化不一致".to_string());
    }
    
    // 测试配置修复功能
    let mut broken_config = AppConfig::default();
    broken_config.macro_config.hotkeys.clear();
    broken_config.macro_config.overlay_settings.transparency = 150;
    broken_config.ui_settings.window_size = (50, 50);
    
    println!("测试配置修复功能...");
    if broken_config.validate().is_ok() {
        return Err("破坏的配置不应该验证通过".to_string());
    }
    println!("✓ 破坏的配置正确地验证失败");
    
    broken_config.fix_invalid_values();
    broken_config.validate().map_err(|e| format!("修复后的配置验证失败: {}", e))?;
    println!("✓ 修复后的配置验证通过");
    
    // 验证修复结果
    if broken_config.macro_config.hotkeys.len() != 5 {
        return Err("按键配置修复失败".to_string());
    }
    println!("✓ 按键配置已修复");
    
    if broken_config.macro_config.overlay_settings.transparency != 100 {
        return Err("透明度修复失败".to_string());
    }
    println!("✓ 透明度已修复");
    
    if broken_config.ui_settings.window_size.0 < 400 || broken_config.ui_settings.window_size.1 < 300 {
        return Err("窗口大小修复失败".to_string());
    }
    println!("✓ 窗口大小已修复");
    
    println!("配置测试完成！");
    Ok(())
}