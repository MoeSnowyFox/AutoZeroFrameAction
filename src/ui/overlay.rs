//! 悬浮窗组件
//! 
//! 提供游戏内悬浮窗显示功能，包括按键提示、状态显示等

use crate::models::{OverlaySettings, OverlayDisplayMode, WindowInfo, GameOperation};
use crate::utils::{AppResult, AppError};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use log::{info, debug, warn};

/// 悬浮窗内容类型
#[derive(Debug, Clone)]
pub enum OverlayContent {
    /// 按键提示
    KeyHints(Vec<KeyHint>),
    /// 状态信息
    StatusInfo(StatusInfo),
    /// 自定义文本
    CustomText(String),
}

/// 按键提示信息
#[derive(Debug, Clone)]
pub struct KeyHint {
    /// 操作名称
    pub operation: String,
    /// 按键
    pub key: String,
    /// 描述
    pub description: String,
    /// 是否启用
    pub enabled: bool,
}

/// 状态信息
#[derive(Debug, Clone)]
pub struct StatusInfo {
    /// 程序状态
    pub program_status: String,
    /// 游戏状态
    pub game_status: String,
    /// 当前模式
    pub current_mode: String,
    /// 额外信息
    pub extra_info: HashMap<String, String>,
}

/// 悬浮窗位置
#[derive(Debug, Clone, Copy)]
pub struct OverlayPosition {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for OverlayPosition {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 300,
            height: 200,
        }
    }
}

/// 悬浮窗管理器
pub struct OverlayManager {
    /// 悬浮窗设置
    settings: OverlaySettings,
    /// 是否可见
    visible: bool,
    /// 当前内容
    content: Vec<OverlayContent>,
    /// 位置信息
    position: OverlayPosition,
    /// 最后更新时间
    last_update: Instant,
    /// 更新间隔
    update_interval: Duration,
    /// 是否需要重绘
    needs_redraw: bool,
    /// 悬浮窗句柄（Windows平台）
    #[cfg(windows)]
    window_handle: Option<winapi::shared::windef::HWND>,
}

impl OverlayManager {
    /// 创建新的悬浮窗管理器
    pub fn new() -> Self {
        Self {
            settings: OverlaySettings::default(),
            visible: false,
            content: Vec::new(),
            position: OverlayPosition::default(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
            needs_redraw: false,
            #[cfg(windows)]
            window_handle: None,
        }
    }
    
    /// 使用自定义设置创建悬浮窗管理器
    pub fn with_settings(settings: OverlaySettings) -> Self {
        Self {
            settings,
            visible: false,
            content: Vec::new(),
            position: OverlayPosition::default(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
            needs_redraw: false,
            #[cfg(windows)]
            window_handle: None,
        }
    }
    
    /// 显示悬浮窗
    pub fn show(&mut self) -> AppResult<()> {
        if !self.settings.enabled {
            debug!("悬浮窗未启用，跳过显示");
            return Ok(());
        }
        
        if self.visible {
            debug!("悬浮窗已经可见");
            return Ok(());
        }
        
        info!("显示悬浮窗");
        
        // 创建悬浮窗
        self.create_overlay_window()?;
        
        self.visible = true;
        self.needs_redraw = true;
        
        Ok(())
    }
    
    /// 隐藏悬浮窗
    pub fn hide(&mut self) {
        if !self.visible {
            debug!("悬浮窗已经隐藏");
            return;
        }
        
        info!("隐藏悬浮窗");
        
        // 销毁悬浮窗
        self.destroy_overlay_window();
        
        self.visible = false;
    }
    
    /// 更新设置
    pub fn update_settings(&mut self, settings: OverlaySettings) {
        let old_enabled = self.settings.enabled;
        let old_transparency = self.settings.transparency;
        let old_display_mode = self.settings.display_mode.clone();
        
        self.settings = settings;
        
        // 如果启用状态改变
        if old_enabled != self.settings.enabled {
            if self.settings.enabled && !self.visible {
                if let Err(e) = self.show() {
                    warn!("启用悬浮窗失败: {}", e);
                }
            } else if !self.settings.enabled && self.visible {
                self.hide();
            }
        }
        
        // 如果透明度或显示模式改变，需要重绘
        if old_transparency != self.settings.transparency || old_display_mode != self.settings.display_mode {
            self.needs_redraw = true;
        }
        
        info!("更新悬浮窗设置");
    }
    
    /// 检查是否应该显示
    pub fn should_show(&self, window_state: &WindowInfo, app_foreground: bool) -> bool {
        if !self.settings.enabled {
            return false;
        }
        
        match self.settings.display_mode {
            OverlayDisplayMode::Always => true,
            OverlayDisplayMode::WhenForeground => app_foreground,
            OverlayDisplayMode::OnlyAboveProgram => app_foreground && window_state.is_foreground,
        }
    }
    
    /// 更新内容
    pub fn update_content(&mut self, content: Vec<OverlayContent>) {
        self.content = content;
        self.needs_redraw = true;
        debug!("更新悬浮窗内容，共 {} 项", self.content.len());
    }
    
    /// 添加按键提示
    pub fn add_key_hints(&mut self, operations: &HashMap<String, GameOperation>) {
        let hints: Vec<KeyHint> = operations.iter()
            .filter(|(_, operation)| operation.enabled)
            .map(|(name, operation)| KeyHint {
                operation: name.clone(),
                key: operation.hotkey.clone(),
                description: operation.name.clone(),
                enabled: true,
            })
            .collect();
        
        self.content.push(OverlayContent::KeyHints(hints.clone()));
        self.needs_redraw = true;
        debug!("添加 {} 个按键提示", hints.len());
    }
    
    /// 更新状态信息
    pub fn update_status(&mut self, status: StatusInfo) {
        // 查找现有的状态信息并更新，或添加新的
        let mut found = false;
        for content in &mut self.content {
            if let OverlayContent::StatusInfo(ref mut existing_status) = content {
                *existing_status = status.clone();
                found = true;
                break;
            }
        }
        
        if !found {
            self.content.push(OverlayContent::StatusInfo(status));
        }
        
        self.needs_redraw = true;
        debug!("更新状态信息");
    }
    
    /// 设置位置
    pub fn set_position(&mut self, position: OverlayPosition) {
        self.position = position;
        self.needs_redraw = true;
        debug!("设置悬浮窗位置: ({}, {}) {}x{}", 
               position.x, position.y, position.width, position.height);
    }
    
    /// 获取位置
    pub fn get_position(&self) -> OverlayPosition {
        self.position
    }
    
    /// 是否可见
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// 获取设置
    pub fn get_settings(&self) -> &OverlaySettings {
        &self.settings
    }
    
    /// 更新悬浮窗（定期调用）
    pub fn update(&mut self) -> AppResult<()> {
        if !self.visible {
            return Ok(());
        }
        
        let now = Instant::now();
        if now.duration_since(self.last_update) < self.update_interval {
            return Ok(());
        }
        
        self.last_update = now;
        
        // 如果需要重绘
        if self.needs_redraw {
            self.redraw()?;
            self.needs_redraw = false;
        }
        
        Ok(())
    }
    
    /// 清除内容
    pub fn clear_content(&mut self) {
        self.content.clear();
        self.needs_redraw = true;
        debug!("清除悬浮窗内容");
    }
    
    /// 设置更新间隔
    pub fn set_update_interval(&mut self, interval: Duration) {
        self.update_interval = interval;
        debug!("设置悬浮窗更新间隔: {:?}", interval);
    }
    
    /// 创建悬浮窗窗口
    #[cfg(windows)]
    fn create_overlay_window(&mut self) -> AppResult<()> {
        use winapi::um::winuser::*;
        use winapi::um::libloaderapi::GetModuleHandleW;
        use std::ptr;
        
        debug!("创建Windows悬浮窗");
        
        unsafe {
            let class_name = "AutoZeroFrameActionOverlay\0".encode_utf16().collect::<Vec<u16>>();
            let window_name = "Overlay\0".encode_utf16().collect::<Vec<u16>>();
            
            // 创建窗口
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
                class_name.as_ptr(),
                window_name.as_ptr(),
                WS_POPUP,
                self.position.x,
                self.position.y,
                self.position.width as i32,
                self.position.height as i32,
                ptr::null_mut(),
                ptr::null_mut(),
                GetModuleHandleW(ptr::null()),
                ptr::null_mut(),
            );
            
            if hwnd.is_null() {
                return Err(AppError::ui_error("创建悬浮窗失败"));
            }
            
            // 设置透明度
            let opacity = (self.settings.transparency as f32 / 100.0 * 255.0) as u8;
            SetLayeredWindowAttributes(hwnd, 0, opacity, LWA_ALPHA);
            
            // 显示窗口
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
            
            self.window_handle = Some(hwnd);
            debug!("Windows悬浮窗创建成功");
        }
        
        Ok(())
    }
    
    /// 销毁悬浮窗窗口
    #[cfg(windows)]
    fn destroy_overlay_window(&mut self) {
        if let Some(hwnd) = self.window_handle.take() {
            unsafe {
                winapi::um::winuser::DestroyWindow(hwnd);
            }
            debug!("销毁Windows悬浮窗");
        }
    }
    
    /// 重绘悬浮窗
    #[cfg(windows)]
    fn redraw(&mut self) -> AppResult<()> {
        if let Some(_hwnd) = self.window_handle {
            // TODO: 实现实际的绘制逻辑
            // 这里需要使用GDI+或Direct2D来绘制内容
            debug!("重绘悬浮窗内容");
        }
        Ok(())
    }
    
    /// 非Windows平台的占位符实现
    #[cfg(not(windows))]
    fn create_overlay_window(&mut self) -> AppResult<()> {
        warn!("悬浮窗功能在非Windows平台上不支持");
        Ok(())
    }
    
    #[cfg(not(windows))]
    fn destroy_overlay_window(&mut self) {
        debug!("非Windows平台悬浮窗销毁（占位符）");
    }
    
    #[cfg(not(windows))]
    fn redraw(&mut self) -> AppResult<()> {
        debug!("非Windows平台悬浮窗重绘（占位符）");
        Ok(())
    }
}

impl Drop for OverlayManager {
    fn drop(&mut self) {
        if self.visible {
            self.hide();
        }
    }
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::WindowHandle;

    #[test]
    fn test_overlay_manager_creation() {
        let manager = OverlayManager::new();
        assert!(!manager.is_visible());
        assert!(manager.get_settings().enabled);
    }

    #[test]
    fn test_overlay_settings_update() {
        let mut manager = OverlayManager::new();
        let mut settings = OverlaySettings::default();
        settings.transparency = 50;
        
        manager.update_settings(settings);
        assert_eq!(manager.get_settings().transparency, 50);
    }

    #[test]
    fn test_overlay_content_management() {
        let mut manager = OverlayManager::new();
        
        let status = StatusInfo {
            program_status: "运行中".to_string(),
            game_status: "已连接".to_string(),
            current_mode: "宏模式".to_string(),
            extra_info: HashMap::new(),
        };
        
        manager.update_status(status);
        assert_eq!(manager.content.len(), 1);
        
        manager.clear_content();
        assert_eq!(manager.content.len(), 0);
    }

    #[test]
    fn test_overlay_position() {
        let mut manager = OverlayManager::new();
        let position = OverlayPosition {
            x: 200,
            y: 300,
            width: 400,
            height: 500,
        };
        
        manager.set_position(position);
        let retrieved = manager.get_position();
        assert_eq!(retrieved.x, 200);
        assert_eq!(retrieved.y, 300);
        assert_eq!(retrieved.width, 400);
        assert_eq!(retrieved.height, 500);
    }

    #[test]
    fn test_should_show_logic() {
        let manager = OverlayManager::new();
        let window_info = WindowInfo {
            handle: WindowHandle::from(std::ptr::null_mut()),
            title: "Test".to_string(),
            position: (0, 0),
            size: (100, 100),
            process_id: 0,
            is_visible: true,
            is_foreground: true,
        };
        
        // 总是显示模式
        assert!(manager.should_show(&window_info, false));
        assert!(manager.should_show(&window_info, true));
    }
}