use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub window_width: u32,
    pub window_height: u32,
    pub theme: String,
    pub language: String,
    pub auto_save: bool,
    pub last_opened_directory: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            window_width: 800,
            window_height: 600,
            theme: "light".to_string(),
            language: "zh-CN".to_string(),
            auto_save: true,
            last_opened_directory: None,
        }
    }
}

/// 配置管理器，用于管理应用程序的配置
pub struct ConfigManager {
    config_path: PathBuf,
    config: Config,
}

impl ConfigManager {
    /// 创建一个新的配置管理器实例
    /// 从默认位置加载现有配置或创建默认配置
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        
        // 如果配置目录不存在则创建
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            Config::default()
        };

        Ok(ConfigManager { config_path, config })
    }

    /// 获取配置文件的路径
    pub fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        
        let config_path = config_dir.join("AutoZeroFrameAction").join("config.json");
        Ok(config_path)
    }

    fn load_from_file(path: &PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存当前配置到文件
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// 获取当前配置的不可变引用
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// 更改配置的便捷方法，接受一个闭包来修改配置并自动保存
    /// 
    /// # 参数
    /// 
    /// * `f` - 一个接受可变配置引用的闭包，用于修改配置
    /// 
    /// # 示例
    /// 
    /// ```
    /// config_manager.change_config(|config| {
    ///     config.window_width = 1024;
    ///     config.theme = "dark".to_string();
    /// });
    /// ```
    pub fn change_config<F>(&mut self, f: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut Config),
    {
        f(&mut self.config);
        self.save()
    }
}