//! 配置管理服务
//!
//! 负责应用程序配置的持久化存储和加载，支持异步保存和变更通知

use crate::models::config::AppConfig;
use crate::utils::error::{ConfigError, ConfigResult};
use log::{debug, error, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{broadcast, RwLock};

/// 配置变更事件
#[derive(Debug, Clone)]
pub enum ConfigChangeEvent {
    /// 配置已加载
    Loaded,
    /// 配置已保存
    Saved,
    /// 配置已更新
    Updated,
    /// 配置保存失败
    SaveFailed(String),
}

/// 配置管理服务
///
/// 提供配置的持久化存储、加载、异步保存和变更通知功能
pub struct ConfigService {
    /// 配置数据
    config: Arc<RwLock<AppConfig>>,
    /// 配置文件路径
    config_path: PathBuf,
    /// 是否启用自动保存
    auto_save: bool,
    /// 变更通知发送器
    change_sender: broadcast::Sender<ConfigChangeEvent>,
}

impl ConfigService {
    /// 创建新的配置服务
    ///
    /// # 参数
    /// * `config_path` - 配置文件路径
    ///
    /// # 返回
    /// * `ConfigResult<Self>` - 配置服务实例或错误
    pub fn new(config_path: PathBuf) -> ConfigResult<Self> {
        info!("初始化配置服务，配置文件路径: {:?}", config_path);

        // 验证配置文件路径
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                return Err(ConfigError::InvalidPath(format!(
                    "配置文件目录不存在: {:?}",
                    parent
                )));
            }
        }

        let config = AppConfig::default();
        let (change_sender, _) = broadcast::channel(100);

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            auto_save: true,
            change_sender,
        })
    }

    /// 创建配置服务并立即加载配置
    ///
    /// # 参数
    /// * `config_path` - 配置文件路径
    ///
    /// # 返回
    /// * `ConfigResult<Self>` - 配置服务实例或错误
    pub async fn new_and_load(config_path: PathBuf) -> ConfigResult<Self> {
        let mut service = Self::new(config_path)?;
        service.load_config().await?;
        Ok(service)
    }

    /// 加载配置文件
    ///
    /// 如果配置文件不存在，将使用默认配置并创建新文件
    /// 如果配置文件格式错误，将尝试修复并保存
    pub async fn load_config(&mut self) -> ConfigResult<()> {
        info!("加载配置文件: {:?}", self.config_path);

        // 检查配置文件是否存在
        if !self.config_path.exists() {
            warn!("配置文件不存在，使用默认配置: {:?}", self.config_path);

            // 使用默认配置
            let default_config = AppConfig::default();
            {
                let mut config = self.config.write().await;
                *config = default_config;
            }

            // 创建默认配置文件
            self.save_config().await?;

            // 发送加载事件
            let _ = self.change_sender.send(ConfigChangeEvent::Loaded);

            return Ok(());
        }

        // 读取配置文件内容
        let content = fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| ConfigError::IoError(e))?;

        // 解析配置
        let parsed_config: AppConfig = match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                error!("配置文件解析失败: {}, 尝试使用TOML格式", e);

                // 尝试TOML格式
                match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(toml_err) => {
                        error!("TOML格式解析也失败: {}", toml_err);

                        // 如果解析失败，使用默认配置并备份原文件
                        warn!("配置文件格式错误，使用默认配置并备份原文件");
                        self.backup_corrupted_config().await?;

                        let default_config = AppConfig::default();
                        {
                            let mut config = self.config.write().await;
                            *config = default_config.clone();
                        }

                        // 保存默认配置
                        self.save_config().await?;

                        // 发送加载事件
                        let _ = self.change_sender.send(ConfigChangeEvent::Loaded);

                        return Ok(());
                    }
                }
            }
        };

        // 验证配置
        let mut final_config = parsed_config;
        if let Err(validation_error) = final_config.validate() {
            warn!("配置验证失败: {}, 尝试修复", validation_error);

            // 修复无效配置
            final_config.fix_invalid_values();

            // 再次验证
            if let Err(e) = final_config.validate() {
                error!("配置修复失败: {}", e);
                return Err(ConfigError::ValidationError(format!("配置修复失败: {}", e)));
            }

            info!("配置已修复，将保存修复后的配置");
        }

        // 更新内存中的配置
        {
            let mut config = self.config.write().await;
            *config = final_config.clone();
        }

        // 如果配置被修复，保存修复后的配置
        if final_config.validate().is_ok() {
            self.save_config().await?;
        }

        info!("配置加载成功");

        // 发送加载事件
        let _ = self.change_sender.send(ConfigChangeEvent::Loaded);

        Ok(())
    }

    /// 保存配置到文件
    ///
    /// 使用JSON格式保存配置，确保格式化输出便于阅读
    pub async fn save_config(&self) -> ConfigResult<()> {
        debug!("保存配置到文件: {:?}", self.config_path);

        let config = self.config.read().await;

        // 验证配置
        config
            .validate()
            .map_err(|e| ConfigError::ValidationError(format!("保存前配置验证失败: {}", e)))?;

        // 序列化配置
        let content = serde_json::to_string_pretty(&*config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        // 确保父目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ConfigError::IoError(e))?;
        }

        // 写入文件
        fs::write(&self.config_path, content).await.map_err(|e| {
            let error_msg = format!("保存配置文件失败: {}", e);
            error!("{}", error_msg);

            // 发送保存失败事件
            let _ = self
                .change_sender
                .send(ConfigChangeEvent::SaveFailed(error_msg.clone()));

            ConfigError::SaveError(error_msg)
        })?;

        info!("配置保存成功");

        // 发送保存事件
        let _ = self.change_sender.send(ConfigChangeEvent::Saved);

        Ok(())
    }

    /// 异步保存配置
    ///
    /// 在后台异步保存配置，不阻塞当前操作
    ///
    /// # 返回
    /// * `tokio::task::JoinHandle<ConfigResult<()>>` - 异步任务句柄
    pub fn save_config_async(&self) -> tokio::task::JoinHandle<ConfigResult<()>> {
        let config = self.config.clone();
        let config_path = self.config_path.clone();
        let change_sender = self.change_sender.clone();

        tokio::spawn(async move {
            debug!("异步保存配置到文件: {:?}", config_path);

            let config_data = config.read().await;

            // 验证配置
            config_data.validate().map_err(|e| {
                ConfigError::ValidationError(format!("异步保存前配置验证失败: {}", e))
            })?;

            // 序列化配置
            let content = serde_json::to_string_pretty(&*config_data)
                .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

            // 释放读锁
            drop(config_data);

            // 确保父目录存在
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| ConfigError::IoError(e))?;
            }

            // 写入文件
            fs::write(&config_path, content).await.map_err(|e| {
                let error_msg = format!("异步保存配置文件失败: {}", e);
                error!("{}", error_msg);

                // 发送保存失败事件
                let _ = change_sender.send(ConfigChangeEvent::SaveFailed(error_msg.clone()));

                ConfigError::SaveError(error_msg)
            })?;

            info!("配置异步保存成功");

            // 发送保存事件
            let _ = change_sender.send(ConfigChangeEvent::Saved);

            Ok(())
        })
    }

    /// 获取配置的只读副本（异步）
    ///
    /// # 返回
    /// * `AppConfig` - 配置的克隆副本
    pub async fn get_config_async(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    /// 获取配置的只读副本（同步，用于初始化）
    ///
    /// 注意：此方法使用 blocking_read，不应在异步上下文中使用
    ///
    /// # 返回
    /// * `AppConfig` - 配置的克隆副本
    pub fn get_config(&self) -> AppConfig {
        // 使用 try_read 避免死锁，如果失败返回默认配置
        match self.config.try_read() {
            Ok(config) => config.clone(),
            Err(_) => {
                warn!("无法获取配置读锁，返回默认配置");
                AppConfig::default()
            }
        }
    }

    /// 获取配置的共享引用
    ///
    /// # 返回
    /// * `Arc<RwLock<AppConfig>>` - 配置的共享引用
    pub fn get_config_ref(&self) -> Arc<RwLock<AppConfig>> {
        self.config.clone()
    }

    /// 更新配置
    ///
    /// # 参数
    /// * `updater` - 配置更新函数
    ///
    /// # 返回
    /// * `ConfigResult<()>` - 更新结果
    pub async fn update_config<F>(&self, updater: F) -> ConfigResult<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        debug!("更新配置");

        {
            let mut config = self.config.write().await;
            updater(&mut *config);

            // 验证更新后的配置
            config
                .validate()
                .map_err(|e| ConfigError::ValidationError(format!("配置更新后验证失败: {}", e)))?;
        }

        // 如果启用自动保存，异步保存配置
        if self.auto_save {
            let _save_handle = self.save_config_async();
        }

        info!("配置更新成功");

        // 发送更新事件
        let _ = self.change_sender.send(ConfigChangeEvent::Updated);

        Ok(())
    }

    /// 批量更新配置
    ///
    /// 允许进行多个配置更改，最后一次性保存
    ///
    /// # 参数
    /// * `updaters` - 配置更新函数列表
    ///
    /// # 返回
    /// * `ConfigResult<()>` - 更新结果
    pub async fn batch_update_config<F>(&self, updaters: Vec<F>) -> ConfigResult<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        debug!("批量更新配置，更新数量: {}", updaters.len());

        {
            let mut config = self.config.write().await;

            // 应用所有更新
            for updater in updaters {
                updater(&mut *config);
            }

            // 验证更新后的配置
            config.validate().map_err(|e| {
                ConfigError::ValidationError(format!("批量配置更新后验证失败: {}", e))
            })?;
        }

        // 如果启用自动保存，异步保存配置
        if self.auto_save {
            let _save_handle = self.save_config_async();
        }

        info!("批量配置更新成功");

        // 发送更新事件
        let _ = self.change_sender.send(ConfigChangeEvent::Updated);

        Ok(())
    }

    /// 重置配置为默认值
    ///
    /// # 返回
    /// * `ConfigResult<()>` - 重置结果
    pub async fn reset_to_default(&self) -> ConfigResult<()> {
        info!("重置配置为默认值");

        {
            let mut config = self.config.write().await;
            *config = AppConfig::default();
        }

        // 保存默认配置
        self.save_config().await?;

        info!("配置已重置为默认值");

        // 发送更新事件
        let _ = self.change_sender.send(ConfigChangeEvent::Updated);

        Ok(())
    }

    /// 订阅配置变更事件
    ///
    /// # 返回
    /// * `broadcast::Receiver<ConfigChangeEvent>` - 事件接收器
    pub fn subscribe_changes(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.change_sender.subscribe()
    }

    /// 设置自动保存状态
    ///
    /// # 参数
    /// * `enabled` - 是否启用自动保存
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
        info!("自动保存设置为: {}", enabled);
    }

    /// 获取自动保存状态
    ///
    /// # 返回
    /// * `bool` - 是否启用自动保存
    pub fn is_auto_save_enabled(&self) -> bool {
        self.auto_save
    }

    /// 获取配置文件路径
    ///
    /// # 返回
    /// * `&PathBuf` - 配置文件路径
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// 备份损坏的配置文件
    ///
    /// # 返回
    /// * `ConfigResult<()>` - 备份结果
    async fn backup_corrupted_config(&self) -> ConfigResult<()> {
        let backup_path = self.config_path.with_extension("bak");

        info!("备份损坏的配置文件到: {:?}", backup_path);

        fs::copy(&self.config_path, &backup_path)
            .await
            .map_err(|e| ConfigError::IoError(e))?;

        info!("配置文件备份完成");

        Ok(())
    }

    /// 验证配置文件完整性
    ///
    /// # 返回
    /// * `ConfigResult<bool>` - 配置是否有效
    pub async fn validate_config_file(&self) -> ConfigResult<bool> {
        if !self.config_path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| ConfigError::IoError(e))?;

        // 尝试解析配置
        match serde_json::from_str::<AppConfig>(&content) {
            Ok(config) => {
                // 验证配置内容
                match config.validate() {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            Err(_) => {
                // 尝试TOML格式
                match toml::from_str::<AppConfig>(&content) {
                    Ok(config) => match config.validate() {
                        Ok(_) => Ok(true),
                        Err(_) => Ok(false),
                    },
                    Err(_) => Ok(false),
                }
            }
        }
    }

    /// 导出配置到指定路径
    ///
    /// # 参数
    /// * `export_path` - 导出路径
    ///
    /// # 返回
    /// * `ConfigResult<()>` - 导出结果
    pub async fn export_config(&self, export_path: &PathBuf) -> ConfigResult<()> {
        info!("导出配置到: {:?}", export_path);

        let config = self.config.read().await;

        // 序列化配置
        let content = serde_json::to_string_pretty(&*config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        // 确保父目录存在
        if let Some(parent) = export_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| ConfigError::IoError(e))?;
        }

        // 写入文件
        fs::write(export_path, content)
            .await
            .map_err(|e| ConfigError::SaveError(format!("导出配置失败: {}", e)))?;

        info!("配置导出成功");

        Ok(())
    }

    /// 从指定路径导入配置
    ///
    /// # 参数
    /// * `import_path` - 导入路径
    ///
    /// # 返回
    /// * `ConfigResult<()>` - 导入结果
    pub async fn import_config(&self, import_path: &PathBuf) -> ConfigResult<()> {
        info!("从路径导入配置: {:?}", import_path);

        if !import_path.exists() {
            return Err(ConfigError::FileNotFound);
        }

        // 读取配置文件
        let content = fs::read_to_string(import_path)
            .await
            .map_err(|e| ConfigError::IoError(e))?;

        // 解析配置
        let imported_config: AppConfig =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        // 验证配置
        imported_config
            .validate()
            .map_err(|e| ConfigError::ValidationError(format!("导入的配置无效: {}", e)))?;

        // 更新配置
        {
            let mut config = self.config.write().await;
            *config = imported_config;
        }

        // 保存配置
        self.save_config().await?;

        info!("配置导入成功");

        // 发送更新事件
        let _ = self.change_sender.send(ConfigChangeEvent::Updated);

        Ok(())
    }

    // ========== 便捷更新方法 ==========

    /// 更新操作模式
    pub async fn update_mode(
        &self,
        mode: crate::models::config::OperationMode,
    ) -> ConfigResult<()> {
        self.update_config(|config| {
            config.mode = mode;
        })
        .await
    }

    /// 更新主题设置
    pub async fn update_theme(&self, theme: crate::models::config::Theme) -> ConfigResult<()> {
        self.update_config(|config| {
            config.ui_settings.theme = theme;
        })
        .await
    }

    /// 更新按键配置
    pub async fn update_hotkeys(
        &self,
        hotkeys: std::collections::HashMap<String, String>,
    ) -> ConfigResult<()> {
        self.update_config(|config| {
            config.macro_config.hotkeys = hotkeys.clone();
            config.intelligent_config.hotkeys = hotkeys;
        })
        .await
    }

    /// 更新游戏按键配置
    pub async fn update_game_keys(
        &self,
        game_keys: std::collections::HashMap<String, String>,
    ) -> ConfigResult<()> {
        self.update_config(|config| {
            config.global_settings.game_keys = game_keys;
        })
        .await
    }

    /// 更新悬浮窗设置
    pub async fn update_overlay_settings(
        &self,
        enabled: bool,
        display_mode: i32,
        transparency: u8,
    ) -> ConfigResult<()> {
        self.update_config(|config| {
            config.macro_config.overlay_settings.enabled = enabled;
            config.macro_config.overlay_settings.display_mode = match display_mode {
                0 => crate::models::config::OverlayDisplayMode::Always,
                1 => crate::models::config::OverlayDisplayMode::WhenForeground,
                _ => crate::models::config::OverlayDisplayMode::OnlyAboveProgram,
            };
            config.macro_config.overlay_settings.transparency = transparency;

            // 同步到智能模式配置
            config.intelligent_config.overlay_settings =
                config.macro_config.overlay_settings.clone();
        })
        .await
    }

    /// 更新自动启动设置
    pub async fn update_auto_start(&self, enabled: bool) -> ConfigResult<()> {
        self.update_config(|config| {
            config.global_settings.auto_start_on_detection = enabled;
        })
        .await
    }

    /// 更新战斗检测设置
    pub async fn update_battle_detection(&self, enabled: bool) -> ConfigResult<()> {
        self.update_config(|config| {
            config.macro_config.battle_detection_enabled = enabled;
        })
        .await
    }

    /// 更新智能模式功能
    pub async fn update_intelligent_features(&self, features: Vec<String>) -> ConfigResult<()> {
        self.update_config(|config| {
            config.intelligent_config.intelligent_features = features;
        })
        .await
    }

    /// 更新智能模式按键配置
    pub async fn update_smart_hotkeys(
        &self,
        hotkeys: std::collections::HashMap<String, String>,
    ) -> ConfigResult<()> {
        self.update_config(|config| {
            for (key, value) in hotkeys {
                config.intelligent_config.hotkeys.insert(key, value);
            }
        })
        .await
    }

    /// 重置配置为默认值（别名方法）
    pub async fn reset_to_defaults(&self) -> ConfigResult<()> {
        self.reset_to_default().await
    }

    /// 仅更新悬浮窗启用状态
    pub async fn update_overlay_enabled(&self, enabled: bool) -> ConfigResult<()> {
        self.update_config(|config| {
            config.macro_config.overlay_settings.enabled = enabled;
            config.intelligent_config.overlay_settings.enabled = enabled;
        })
        .await
    }

    /// 仅更新悬浮窗透明度
    pub async fn update_overlay_opacity(&self, opacity: u8) -> ConfigResult<()> {
        self.update_config(|config| {
            config.macro_config.overlay_settings.transparency = opacity;
            config.intelligent_config.overlay_settings.transparency = opacity;
        })
        .await
    }

    /// 更新悬浮窗显示模式
    pub async fn update_overlay_display_mode(&self, mode: i32) -> ConfigResult<()> {
        self.update_config(|config| {
            config.macro_config.overlay_settings.display_mode = match mode {
                0 => crate::models::config::OverlayDisplayMode::Always,
                1 => crate::models::config::OverlayDisplayMode::WhenForeground,
                2 => crate::models::config::OverlayDisplayMode::OnlyAboveProgram,
                _ => crate::models::config::OverlayDisplayMode::Always,
            };
        })
        .await
    }

    /// 更新智能模式悬浮窗启用状态
    pub async fn update_smart_overlay_enabled(&self, enabled: bool) -> ConfigResult<()> {
        self.update_config(|config| {
            config.intelligent_config.overlay_settings.enabled = enabled;
        })
        .await
    }

    /// 更新智能模式功能设置
    pub async fn update_smart_feature(&self, feature: &str, enabled: bool) -> ConfigResult<()> {
        self.update_config(|config| {
            let features = &mut config.intelligent_config.intelligent_features;
            if enabled {
                if !features.contains(&feature.to_string()) {
                    features.push(feature.to_string());
                }
            } else {
                features.retain(|f| f != feature);
            }
        })
        .await
    }

    /// 更新游戏按键配置（单个按键）
    pub async fn update_game_key(&self, key_type: &str, key_value: &str) -> ConfigResult<()> {
        self.update_config(|config| {
            config.global_settings.game_keys.insert(key_type.to_string(), key_value.to_string());
        })
        .await
    }
    
    /// 更新智能模式悬浮窗显示模式
    pub async fn update_smart_overlay_display_mode(&self, mode: i32) -> ConfigResult<()> {
        self.update_config(|config| {
            config.intelligent_config.overlay_settings.display_mode = match mode {
                0 => crate::models::config::OverlayDisplayMode::Always,
                1 => crate::models::config::OverlayDisplayMode::WhenForeground,
                2 => crate::models::config::OverlayDisplayMode::OnlyAboveProgram,
                _ => crate::models::config::OverlayDisplayMode::Always,
            };
        })
        .await
    }
    
    /// 更新智能模式悬浮窗透明度
    pub async fn update_smart_overlay_opacity(&self, opacity: u8) -> ConfigResult<()> {
        self.update_config(|config| {
            config.intelligent_config.overlay_settings.transparency = opacity;
        })
        .await
    }
    
    /// 更新软件设置
    pub async fn update_app_setting(&self, setting_type: &str, setting_value: &str) -> ConfigResult<()> {
        self.update_config(|config| {
            match setting_type {
                "minimize-to-tray" => {
                    config.ui_settings.minimize_to_tray = setting_value == "true";
                }
                "start-with-windows" => {
                    config.ui_settings.start_with_windows = setting_value == "true";
                }
                "auto-check-updates" => {
                    config.ui_settings.auto_check_updates = setting_value == "true";
                }
                "language" => {
                    config.ui_settings.language = setting_value.to_string();
                }
                "theme" => {
                    config.ui_settings.theme = match setting_value {
                        "light" => crate::models::config::Theme::Light,
                        "dark" => crate::models::config::Theme::Dark,
                        "auto" => crate::models::config::Theme::Auto,
                        _ => crate::models::config::Theme::Light,
                    };
                }
                _ => {
                    warn!("未知的软件设置类型: {}", setting_type);
                }
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    /// 创建临时配置文件路径
    fn create_temp_config_path() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("创建临时目录失败");
        let config_path = temp_dir.path().join("test_config.json");
        (temp_dir, config_path)
    }

    #[tokio::test]
    async fn test_config_service_new() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let service = ConfigService::new(config_path.clone()).unwrap();

        assert_eq!(service.get_config_path(), &config_path);
        assert!(service.is_auto_save_enabled());

        // 验证默认配置
        let config = service.get_config().await;
        assert_eq!(config, AppConfig::default());
    }

    #[tokio::test]
    async fn test_config_service_new_invalid_path() {
        let invalid_path = PathBuf::from("/nonexistent/directory/config.json");

        let result = ConfigService::new(invalid_path);
        assert!(result.is_err());

        if let Err(ConfigError::InvalidPath(_)) = result {
            // 预期的错误类型
        } else {
            panic!("期望 InvalidPath 错误");
        }
    }

    #[tokio::test]
    async fn test_load_config_file_not_exists() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let mut service = ConfigService::new(config_path.clone()).unwrap();

        // 配置文件不存在时应该使用默认配置并创建文件
        service.load_config().await.unwrap();

        // 验证文件已创建
        assert!(config_path.exists());

        // 验证配置是默认配置
        let config = service.get_config().await;
        assert_eq!(config, AppConfig::default());
    }

    #[tokio::test]
    async fn test_load_config_valid_file() {
        let (_temp_dir, config_path) = create_temp_config_path();

        // 创建有效的配置文件
        let test_config = AppConfig::default();
        let content = serde_json::to_string_pretty(&test_config).unwrap();
        fs::write(&config_path, content).unwrap();

        let mut service = ConfigService::new(config_path).unwrap();
        service.load_config().await.unwrap();

        let loaded_config = service.get_config().await;
        assert_eq!(loaded_config, test_config);
    }

    #[tokio::test]
    async fn test_load_config_invalid_json() {
        let (_temp_dir, config_path) = create_temp_config_path();

        // 创建无效的JSON文件
        fs::write(&config_path, "invalid json content").unwrap();

        let mut service = ConfigService::new(config_path.clone()).unwrap();
        service.load_config().await.unwrap();

        // 应该使用默认配置并备份原文件
        let config = service.get_config().await;
        assert_eq!(config, AppConfig::default());

        // 验证备份文件存在
        let backup_path = config_path.with_extension("bak");
        assert!(backup_path.exists());
    }

    #[tokio::test]
    async fn test_load_config_toml_format() {
        let (_temp_dir, config_path) = create_temp_config_path();

        // 创建TOML格式的配置文件
        let test_config = AppConfig::default();
        let content = toml::to_string(&test_config).unwrap();
        fs::write(&config_path, content).unwrap();

        let mut service = ConfigService::new(config_path).unwrap();
        service.load_config().await.unwrap();

        let loaded_config = service.get_config().await;
        assert_eq!(loaded_config, test_config);
    }

    #[tokio::test]
    async fn test_save_config() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let service = ConfigService::new(config_path.clone()).unwrap();

        // 保存配置
        service.save_config().await.unwrap();

        // 验证文件已创建
        assert!(config_path.exists());

        // 验证文件内容
        let content = fs::read_to_string(&config_path).unwrap();
        let saved_config: AppConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(saved_config, AppConfig::default());
    }

    #[tokio::test]
    async fn test_save_config_async() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let service = ConfigService::new(config_path.clone()).unwrap();

        // 异步保存配置
        let handle = service.save_config_async();
        handle.await.unwrap().unwrap();

        // 验证文件已创建
        assert!(config_path.exists());

        // 验证文件内容
        let content = fs::read_to_string(&config_path).unwrap();
        let saved_config: AppConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(saved_config, AppConfig::default());
    }

    #[tokio::test]
    async fn test_update_config() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let service = ConfigService::new(config_path.clone()).unwrap();

        // 更新配置
        service
            .update_config(|config| {
                config.global_settings.auto_start_on_detection = true;
            })
            .await
            .unwrap();

        // 验证配置已更新
        let config = service.get_config().await;
        assert!(config.global_settings.auto_start_on_detection);

        // 由于启用了自动保存，等待一下让异步保存完成
        sleep(Duration::from_millis(100)).await;

        // 验证文件已保存
        assert!(config_path.exists());
    }

    #[tokio::test]
    async fn test_batch_update_config() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let service = ConfigService::new(config_path).unwrap();

        // 批量更新配置
        let updaters = vec![
            |config: &mut AppConfig| {
                config.global_settings.auto_start_on_detection = true;
            },
            |config: &mut AppConfig| {
                config.macro_config.battle_detection_enabled = false;
            },
            |config: &mut AppConfig| {
                config.ui_settings.theme = crate::models::config::Theme::Dark;
            },
        ];

        service.batch_update_config(updaters).await.unwrap();

        // 验证所有更新都已应用
        let config = service.get_config().await;
        assert!(config.global_settings.auto_start_on_detection);
        assert!(!config.macro_config.battle_detection_enabled);
        assert_eq!(config.ui_settings.theme, crate::models::config::Theme::Dark);
    }

    #[tokio::test]
    async fn test_reset_to_default() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let service = ConfigService::new(config_path.clone()).unwrap();

        // 先修改配置
        service
            .update_config(|config| {
                config.global_settings.auto_start_on_detection = true;
                config.macro_config.battle_detection_enabled = false;
            })
            .await
            .unwrap();

        // 重置为默认值
        service.reset_to_default().await.unwrap();

        // 验证配置已重置
        let config = service.get_config().await;
        assert_eq!(config, AppConfig::default());

        // 验证文件已保存
        assert!(config_path.exists());
    }

    #[tokio::test]
    async fn test_config_change_events() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let mut service = ConfigService::new(config_path).unwrap();
        let mut receiver = service.subscribe_changes();

        // 加载配置应该触发Loaded事件
        service.load_config().await.unwrap();

        let event = receiver.recv().await.unwrap();
        assert!(matches!(event, ConfigChangeEvent::Loaded));

        // 更新配置应该触发Updated事件
        service
            .update_config(|config| {
                config.global_settings.auto_start_on_detection = true;
            })
            .await
            .unwrap();

        let event = receiver.recv().await.unwrap();
        assert!(matches!(event, ConfigChangeEvent::Updated));
    }

    #[tokio::test]
    async fn test_auto_save_control() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let mut service = ConfigService::new(config_path.clone()).unwrap();

        // 默认应该启用自动保存
        assert!(service.is_auto_save_enabled());

        // 禁用自动保存
        service.set_auto_save(false);
        assert!(!service.is_auto_save_enabled());

        // 更新配置
        service
            .update_config(|config| {
                config.global_settings.auto_start_on_detection = true;
            })
            .await
            .unwrap();

        // 由于禁用了自动保存，文件不应该存在
        sleep(Duration::from_millis(100)).await;
        assert!(!config_path.exists());

        // 手动保存
        service.save_config().await.unwrap();
        assert!(config_path.exists());
    }

    #[tokio::test]
    async fn test_validate_config_file() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let service = ConfigService::new(config_path.clone()).unwrap();

        // 文件不存在时应该返回false
        let is_valid = service.validate_config_file().await.unwrap();
        assert!(!is_valid);

        // 创建有效配置文件
        let config = AppConfig::default();
        let content = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&config_path, content).unwrap();

        let is_valid = service.validate_config_file().await.unwrap();
        assert!(is_valid);

        // 创建无效配置文件
        fs::write(&config_path, "invalid json").unwrap();

        let is_valid = service.validate_config_file().await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_export_import_config() {
        let (_temp_dir, config_path) = create_temp_config_path();
        let export_path = config_path.with_file_name("exported_config.json");

        let service = ConfigService::new(config_path).unwrap();

        // 修改配置
        service
            .update_config(|config| {
                config.global_settings.auto_start_on_detection = true;
                config.macro_config.battle_detection_enabled = false;
            })
            .await
            .unwrap();

        let original_config = service.get_config().await;

        // 导出配置
        service.export_config(&export_path).await.unwrap();
        assert!(export_path.exists());

        // 重置配置
        service.reset_to_default().await.unwrap();

        // 导入配置
        service.import_config(&export_path).await.unwrap();

        // 验证配置已恢复
        let imported_config = service.get_config().await;
        assert_eq!(imported_config, original_config);
    }

    #[tokio::test]
    async fn test_new_and_load() {
        let (_temp_dir, config_path) = create_temp_config_path();

        // 创建配置文件
        let test_config = AppConfig::default();
        let content = serde_json::to_string_pretty(&test_config).unwrap();
        fs::write(&config_path, content).unwrap();

        // 使用new_and_load创建服务
        let service = ConfigService::new_and_load(config_path).await.unwrap();

        let loaded_config = service.get_config().await;
        assert_eq!(loaded_config, test_config);
    }

    #[tokio::test]
    async fn test_get_config_ref() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let service = ConfigService::new(config_path).unwrap();
        let config_ref = service.get_config_ref();

        // 通过引用修改配置
        {
            let mut config = config_ref.write().await;
            config.global_settings.auto_start_on_detection = true;
        }

        // 验证修改已生效
        let config = service.get_config().await;
        assert!(config.global_settings.auto_start_on_detection);
    }

    #[tokio::test]
    async fn test_config_validation_during_update() {
        let (_temp_dir, config_path) = create_temp_config_path();

        let service = ConfigService::new(config_path).unwrap();

        // 尝试设置无效的配置值
        let result = service
            .update_config(|config| {
                config.macro_config.overlay_settings.transparency = 200; // 无效值
            })
            .await;

        // 更新应该失败
        assert!(result.is_err());

        // 配置应该保持原样
        let config = service.get_config().await;
        assert_eq!(config.macro_config.overlay_settings.transparency, 80); // 默认值
    }

    #[tokio::test]
    async fn test_corrupted_config_recovery() {
        let (_temp_dir, config_path) = create_temp_config_path();

        // 创建损坏的配置文件
        fs::write(&config_path, r#"{"invalid": "json", "missing": "fields"}"#).unwrap();

        let mut service = ConfigService::new(config_path.clone()).unwrap();

        // 加载配置应该成功，使用默认配置
        service.load_config().await.unwrap();

        // 验证使用了默认配置
        let config = service.get_config().await;
        assert_eq!(config, AppConfig::default());

        // 验证备份文件存在
        let backup_path = config_path.with_extension("bak");
        assert!(backup_path.exists());

        // 验证新的配置文件是有效的
        let is_valid = service.validate_config_file().await.unwrap();
        assert!(is_valid);
    }
}
