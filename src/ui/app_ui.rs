//! 主应用程序UI
//! 
//! 集成所有组件和服务，实现应用程序生命周期管理

use crate::services::*;
use crate::utils::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error, debug};
use std::path::PathBuf;

// 引入 Slint 生成的模块
slint::include_modules!();

/// 主应用程序
/// 
/// 负责集成所有组件和服务，管理应用程序生命周期
pub struct MainApp {
    /// Slint UI 句柄
    ui_handle: MainWindow,
    /// 配置管理服务
    config_service: Arc<ConfigService>,
    /// 状态管理器
    state_manager: Arc<RwLock<StateManager>>,
    /// 窗口管理服务
    window_service: Arc<RwLock<WindowService>>,
    /// 模式管理器
    mode_manager: Arc<RwLock<ModeManager>>,
}

impl MainApp {
    /// 创建新的主应用程序实例
    pub fn new() -> AppResult<Self> {
        info!("初始化主应用程序");
        
        // 创建UI实例
        let ui_handle = MainWindow::new()
            .map_err(|e| AppError::UI(format!("创建UI失败: {}", e)))?;
        
        // 初始化配置服务
        let config_path = crate::utils::system::get_app_config_dir()
            .map_err(|e| AppError::Config(crate::utils::error::ConfigError::InvalidPath(
                format!("无法获取配置目录: {}", e)
            )))?
            .join("config.json");
        let config_service = Arc::new(
            ConfigService::new(config_path)
                .map_err(AppError::Config)?
        );
        
        // 初始化状态管理器
        let state_manager = Arc::new(RwLock::new(StateManager::new()));
        
        // 初始化窗口管理服务
        let window_service = Arc::new(RwLock::new(WindowService::new()));
        
        // 初始化模式管理器
        let mode_manager = Arc::new(RwLock::new(ModeManager::new()));
        
        info!("主应用程序初始化完成");
        
        Ok(Self {
            ui_handle,
            config_service,
            state_manager,
            window_service,
            mode_manager,
        })
    }
    
    /// 启动应用程序
    pub async fn run(&mut self) -> AppResult<()> {
        info!("启动主应用程序");
        
        // 加载配置并应用到 UI
        self.load_and_apply_config().await?;
        
        // 设置UI回调
        self.setup_ui_callbacks()?;
        
        // 显示主窗口
        self.ui_handle.show()
            .map_err(|e| AppError::UI(format!("显示窗口失败: {}", e)))?;
        
        info!("主应用程序启动完成");
        
        // 运行事件循环
        slint::run_event_loop()
            .map_err(|e| AppError::UI(format!("事件循环失败: {}", e)))?;
        
        // 保存配置
        self.save_config_on_exit().await?;
        
        Ok(())
    }
    
    /// 加载配置并应用到 UI
    async fn load_and_apply_config(&self) -> AppResult<()> {
        debug!("加载并应用配置");
        
        let config = self.config_service.get_config();
        
        // 应用模式设置
        let mode = match config.mode {
            crate::models::config::OperationMode::Macro => 0,
            crate::models::config::OperationMode::Intelligent => 1,
        };
        AppState::get(&self.ui_handle).set_current_mode(mode);
        
        // 应用宏模式配置
        let macro_config = MacroConfig::get(&self.ui_handle);
        if let Some(key) = config.macro_config.hotkeys.get("deploy") {
            macro_config.set_deploy_operator(key.into());
        }
        if let Some(key) = config.macro_config.hotkeys.get("skill") {
            macro_config.set_activate_skill(key.into());
        }
        if let Some(key) = config.macro_config.hotkeys.get("retreat") {
            macro_config.set_retreat_operator(key.into());
        }
        if let Some(key) = config.macro_config.hotkeys.get("focus") {
            macro_config.set_focus_view(key.into());
        }
        if let Some(key) = config.macro_config.hotkeys.get("pause") {
            macro_config.set_pause_game(key.into());
        }
        macro_config.set_show_overlay(config.macro_config.overlay_settings.enabled);
        macro_config.set_overlay_opacity(config.macro_config.overlay_settings.transparency as i32);
        macro_config.set_battle_only(config.macro_config.battle_detection_enabled);
        
        // 应用智能模式配置
        let smart_config = SmartConfig::get(&self.ui_handle);
        if let Some(key) = config.intelligent_config.hotkeys.get("deploy") {
            smart_config.set_deploy_operator(key.into());
        }
        if let Some(key) = config.intelligent_config.hotkeys.get("skill") {
            smart_config.set_activate_skill(key.into());
        }
        if let Some(key) = config.intelligent_config.hotkeys.get("retreat") {
            smart_config.set_retreat_operator(key.into());
        }
        if let Some(key) = config.intelligent_config.hotkeys.get("focus") {
            smart_config.set_focus_view(key.into());
        }
        if let Some(key) = config.intelligent_config.hotkeys.get("pause") {
            smart_config.set_pause_game(key.into());
        }
        smart_config.set_show_overlay(config.intelligent_config.overlay_settings.enabled);
        smart_config.set_overlay_display_mode(match config.intelligent_config.overlay_settings.display_mode {
            crate::models::config::OverlayDisplayMode::Always => 0,
            crate::models::config::OverlayDisplayMode::WhenForeground => 1,
            crate::models::config::OverlayDisplayMode::OnlyAboveProgram => 2,
        });
        smart_config.set_overlay_opacity(config.intelligent_config.overlay_settings.transparency as i32);
        
        // 应用游戏配置
        let game_config = GameConfig::get(&self.ui_handle);
        if let Some(key) = config.global_settings.game_keys.get("deploy") {
            game_config.set_game_deploy(key.into());
        }
        if let Some(key) = config.global_settings.game_keys.get("skill") {
            game_config.set_game_skill(key.into());
        }
        if let Some(key) = config.global_settings.game_keys.get("retreat") {
            game_config.set_game_retreat(key.into());
        }
        if let Some(key) = config.global_settings.game_keys.get("pause") {
            game_config.set_game_pause(key.into());
        }
        if let Some(key) = config.global_settings.game_keys.get("speed_up") {
            game_config.set_game_speed_up(key.into());
        }
        if let Some(key) = config.global_settings.game_keys.get("view_left") {
            game_config.set_game_view_left(key.into());
        }
        if let Some(key) = config.global_settings.game_keys.get("view_right") {
            game_config.set_game_view_right(key.into());
        }
        if let Some(key) = config.global_settings.game_keys.get("view_reset") {
            game_config.set_game_view_reset(key.into());
        }
        game_config.set_auto_start(config.global_settings.auto_start_on_detection);
        
        // 应用软件设置
        let app_settings = AppSettings::get(&self.ui_handle);
        app_settings.set_minimize_to_tray(config.ui_settings.minimize_to_tray);
        app_settings.set_start_with_windows(config.ui_settings.start_with_windows);
        app_settings.set_auto_check_updates(config.ui_settings.auto_check_updates);
        app_settings.set_language(config.ui_settings.language.into());
        app_settings.set_theme(match config.ui_settings.theme {
            crate::models::config::Theme::Light => "light".into(),
            crate::models::config::Theme::Dark => "dark".into(),
            crate::models::config::Theme::Auto => "auto".into(),
        });
        
        debug!("配置应用完成");
        Ok(())
    }
    
    /// 应用主题
    fn apply_theme(&self, _theme_mode: i32) {
        // 移除主题切换功能，只使用浅色主题
    }
    
    /// 设置UI回调
    fn setup_ui_callbacks(&self) -> AppResult<()> {
        debug!("设置UI回调函数");
        
        // ===== 启动按钮 =====
        let state_manager = Arc::clone(&self.state_manager);
        let ui_handle_weak = self.ui_handle.as_weak();
        self.ui_handle.on_start_clicked(move || {
            let state_manager = Arc::clone(&state_manager);
            let ui_weak = ui_handle_weak.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    info!("用户请求启动程序核心");
                    if let Err(e) = state_manager.write().await.start_core().await {
                        error!("启动核心功能失败: {}", e);
                    } else {
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                AppState::get(&ui).set_is_running(true);
                                AppState::get(&ui).set_status_text("运行中".into());
                            }
                        });
                    }
                });
            });
        });
        
        // ===== 停止按钮 =====
        let state_manager = Arc::clone(&self.state_manager);
        let ui_handle_weak = self.ui_handle.as_weak();
        self.ui_handle.on_stop_clicked(move || {
            let state_manager = Arc::clone(&state_manager);
            let ui_weak = ui_handle_weak.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    info!("用户请求停止程序核心");
                    if let Err(e) = state_manager.write().await.stop_core().await {
                        error!("停止核心功能失败: {}", e);
                    } else {
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                AppState::get(&ui).set_is_running(false);
                                AppState::get(&ui).set_status_text("已停止".into());
                            }
                        });
                    }
                });
            });
        });
        
        // ===== 模式切换 =====
        let mode_manager = Arc::clone(&self.mode_manager);
        let config_service = Arc::clone(&self.config_service);
        self.ui_handle.on_mode_changed(move |mode_idx| {
            let mode = if mode_idx == 0 {
                crate::models::config::OperationMode::Macro
            } else {
                crate::models::config::OperationMode::Intelligent
            };
            info!("用户切换模式为: {:?}", mode);
            
            let mode_manager = Arc::clone(&mode_manager);
            let config_service = Arc::clone(&config_service);
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    if let Err(e) = mode_manager.write().await.switch_mode(mode.clone()) {
                        error!("切换模式失败: {}", e);
                    }
                    if let Err(e) = config_service.update_mode(mode).await {
                        error!("保存模式配置失败: {}", e);
                    }
                });
            });
        });
        
        // ===== 按键检测 =====
        let config_service = Arc::clone(&self.config_service);
        let ui_handle_weak = self.ui_handle.as_weak();
        self.ui_handle.on_start_key_detection(move |config_key| {
            let config_key = config_key.to_string();
            let ui_weak = ui_handle_weak.clone();
            info!("开始检测按键配置: {}", config_key);
            
            // 设置检测状态
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    AppState::get(&ui).set_is_detecting_key(true);
                    AppState::get(&ui).set_detecting_for(config_key.into());
                }
            });
            
            // TODO: 实现实际的按键检测逻辑
            // 这里应该启动一个后台任务来监听按键输入
            // 当检测到按键组合时，调用 key-detected 回调
        });
        
        let config_service = Arc::clone(&self.config_service);
        let ui_handle_weak = self.ui_handle.as_weak();
        self.ui_handle.on_key_detected(move |config_key, key_combination| {
            let config_key = config_key.to_string();
            let key_combination = key_combination.to_string();
            let ui_weak = ui_handle_weak.clone();
            let config_service = Arc::clone(&config_service);
            
            info!("检测到按键组合: {} -> {}", config_key, key_combination);
            
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    // 根据配置键更新相应的配置
                    match config_key.as_str() {
                        "macro-deploy" => {
                            let mut hotkeys = std::collections::HashMap::new();
                            hotkeys.insert("deploy".to_string(), key_combination.clone());
                            if let Err(e) = config_service.update_hotkeys(hotkeys).await {
                                error!("保存宏模式部署按键失败: {}", e);
                            }
                        }
                        "macro-skill" => {
                            let mut hotkeys = std::collections::HashMap::new();
                            hotkeys.insert("skill".to_string(), key_combination.clone());
                            if let Err(e) = config_service.update_hotkeys(hotkeys).await {
                                error!("保存宏模式技能按键失败: {}", e);
                            }
                        }
                        "smart-deploy" => {
                            let mut hotkeys = std::collections::HashMap::new();
                            hotkeys.insert("deploy".to_string(), key_combination.clone());
                            if let Err(e) = config_service.update_smart_hotkeys(hotkeys).await {
                                error!("保存智能模式部署按键失败: {}", e);
                            }
                        }
                        // 添加更多按键配置处理...
                        _ => {
                            warn!("未知的配置键: {}", config_key);
                        }
                    }
                    
                    // 重置检测状态并更新UI
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            AppState::get(&ui).set_is_detecting_key(false);
                            AppState::get(&ui).set_detecting_for("".into());
                            
                            // 更新对应的配置值
                            match config_key.as_str() {
                                "macro-deploy" => MacroConfig::get(&ui).set_deploy_operator(key_combination.into()),
                                "macro-skill" => MacroConfig::get(&ui).set_activate_skill(key_combination.into()),
                                "smart-deploy" => SmartConfig::get(&ui).set_deploy_operator(key_combination.into()),
                                // 添加更多UI更新...
                                _ => {}
                            }
                        }
                    });
                });
            });
        });
        
        // ===== 显示日志 =====
        self.ui_handle.on_show_log(move || {
            info!("用户请求查看日志");
            // TODO: 实现日志获取逻辑
        });
        
        // ===== 保存配置 =====
        let config_service = Arc::clone(&self.config_service);
        self.ui_handle.on_save_config(move || {
            info!("用户请求保存配置");
            let config_service = Arc::clone(&config_service);
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    if let Err(e) = config_service.save_config().await {
                        error!("保存配置失败: {}", e);
                    } else {
                        info!("配置保存完成");
                    }
                });
            });
        });
        
        // ===== 宏模式配置回调 =====
        self.setup_macro_config_callbacks();
        
        // ===== 智能模式配置回调 =====
        self.setup_smart_config_callbacks();
        
        // ===== 游戏配置回调 =====
        self.setup_game_config_callbacks();
        
        // ===== 软件设置回调 =====
        self.setup_app_settings_callbacks();
        
        debug!("UI回调函数设置完成");
        Ok(())
    }
    
    /// 设置宏模式配置回调
    fn setup_macro_config_callbacks(&self) {
        let config_service = Arc::clone(&self.config_service);
        
        // 宏模式按键配置
        self.ui_handle.on_macro_deploy_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("deploy".to_string(), keys);
                        if let Err(e) = config_service.update_hotkeys(hotkeys).await {
                            error!("保存宏模式部署按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_macro_skill_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("skill".to_string(), keys);
                        if let Err(e) = config_service.update_hotkeys(hotkeys).await {
                            error!("保存宏模式技能按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_macro_retreat_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("retreat".to_string(), keys);
                        if let Err(e) = config_service.update_hotkeys(hotkeys).await {
                            error!("保存宏模式撤退按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_macro_focus_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("focus".to_string(), keys);
                        if let Err(e) = config_service.update_hotkeys(hotkeys).await {
                            error!("保存宏模式聚焦按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_macro_pause_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("pause".to_string(), keys);
                        if let Err(e) = config_service.update_hotkeys(hotkeys).await {
                            error!("保存宏模式暂停按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        // 宏模式悬浮窗设置
        self.ui_handle.on_macro_overlay_changed({
            let config_service = Arc::clone(&config_service);
            move |enabled| {
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_overlay_enabled(enabled).await {
                            error!("保存宏模式悬浮窗设置失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_macro_overlay_mode_changed({
            let config_service = Arc::clone(&config_service);
            move |mode| {
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_overlay_display_mode(mode).await {
                            error!("保存宏模式悬浮窗显示模式失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_macro_overlay_opacity_changed({
            let config_service = Arc::clone(&config_service);
            move |opacity| {
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_overlay_opacity(opacity as u8).await {
                            error!("保存宏模式悬浮窗透明度失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_macro_battle_only_changed({
            let config_service = Arc::clone(&config_service);
            move |enabled| {
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_battle_detection(enabled).await {
                            error!("保存宏模式战斗检测设置失败: {}", e);
                        }
                    });
                });
            }
        });
    }
    
    /// 设置智能模式配置回调
    fn setup_smart_config_callbacks(&self) {
        let config_service = Arc::clone(&self.config_service);
        
        // 智能模式按键配置
        self.ui_handle.on_smart_deploy_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("deploy".to_string(), keys);
                        if let Err(e) = config_service.update_smart_hotkeys(hotkeys).await {
                            error!("保存智能模式部署按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_smart_skill_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("skill".to_string(), keys);
                        if let Err(e) = config_service.update_smart_hotkeys(hotkeys).await {
                            error!("保存智能模式技能按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_smart_retreat_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("retreat".to_string(), keys);
                        if let Err(e) = config_service.update_smart_hotkeys(hotkeys).await {
                            error!("保存智能模式撤退按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_smart_focus_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("focus".to_string(), keys);
                        if let Err(e) = config_service.update_smart_hotkeys(hotkeys).await {
                            error!("保存智能模式聚焦按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_smart_pause_changed({
            let config_service = Arc::clone(&config_service);
            move |keys| {
                let keys = keys.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut hotkeys = std::collections::HashMap::new();
                        hotkeys.insert("pause".to_string(), keys);
                        if let Err(e) = config_service.update_smart_hotkeys(hotkeys).await {
                            error!("保存智能模式暂停按键失败: {}", e);
                        }
                    });
                });
            }
        });
        
        // 智能模式悬浮窗设置
        self.ui_handle.on_smart_overlay_changed({
            let config_service = Arc::clone(&config_service);
            move |enabled| {
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_smart_overlay_enabled(enabled).await {
                            error!("保存智能模式悬浮窗设置失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_smart_overlay_mode_changed({
            let config_service = Arc::clone(&config_service);
            move |mode| {
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_smart_overlay_display_mode(mode).await {
                            error!("保存智能模式悬浮窗显示模式失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_smart_overlay_opacity_changed({
            let config_service = Arc::clone(&config_service);
            move |opacity| {
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_smart_overlay_opacity(opacity as u8).await {
                            error!("保存智能模式悬浮窗透明度失败: {}", e);
                        }
                    });
                });
            }
        });
        
        // 智能模式功能选择
        self.ui_handle.on_smart_feature_changed({
            let config_service = Arc::clone(&config_service);
            move |feature, enabled| {
                let feature = feature.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_smart_feature(&feature, enabled).await {
                            error!("保存智能模式功能设置失败: {}", e);
                        }
                    });
                });
            }
        });
    }
    
    /// 设置游戏配置回调
    fn setup_game_config_callbacks(&self) {
        let config_service = Arc::clone(&self.config_service);
        
        self.ui_handle.on_game_config_changed({
            let config_service = Arc::clone(&config_service);
            move |key_type, key_value| {
                let key_type = key_type.to_string();
                let key_value = key_value.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_game_key(&key_type, &key_value).await {
                            error!("保存游戏按键配置失败: {}", e);
                        }
                    });
                });
            }
        });
        
        self.ui_handle.on_auto_start_changed({
            let config_service = Arc::clone(&config_service);
            move |enabled| {
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_auto_start(enabled).await {
                            error!("保存自动启动设置失败: {}", e);
                        }
                    });
                });
            }
        });
    }
    
    /// 设置软件设置回调
    fn setup_app_settings_callbacks(&self) {
        let config_service = Arc::clone(&self.config_service);
        
        self.ui_handle.on_app_settings_changed({
            let config_service = Arc::clone(&config_service);
            move |setting_type, setting_value| {
                let setting_type = setting_type.to_string();
                let setting_value = setting_value.to_string();
                let config_service = Arc::clone(&config_service);
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if let Err(e) = config_service.update_app_setting(&setting_type, &setting_value).await {
                            error!("保存软件设置失败: {}", e);
                        }
                    });
                });
            }
        });
    }
    
    /// 退出时保存配置
    async fn save_config_on_exit(&self) -> AppResult<()> {
        info!("程序退出，保存配置");
        
        if let Err(e) = self.config_service.save_config().await {
            warn!("保存配置失败: {}", e);
        }
        
        Ok(())
    }
    
    /// 获取UI句柄
    pub fn ui_handle(&self) -> &MainWindow {
        &self.ui_handle
    }
}

/// 应用程序构建器
pub struct MainAppBuilder {
    config_path: Option<PathBuf>,
    enable_logging: bool,
}

impl MainAppBuilder {
    pub fn new() -> Self {
        Self {
            config_path: None,
            enable_logging: true,
        }
    }
    
    pub fn with_config_path(mut self, path: PathBuf) -> Self {
        self.config_path = Some(path);
        self
    }
    
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.enable_logging = enabled;
        self
    }
    
    pub fn build(self) -> AppResult<MainApp> {
        if self.enable_logging {
            let _ = env_logger::try_init();
        }
        MainApp::new()
    }
}

impl Default for MainAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}
