//! 窗口相关数据模型

use serde::{Deserialize, Serialize};

/// 窗口句柄包装器（用于线程安全）
#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowHandle(pub winapi::shared::windef::HWND);

#[cfg(windows)]
unsafe impl Send for WindowHandle {}
#[cfg(windows)]
unsafe impl Sync for WindowHandle {}

#[cfg(windows)]
impl From<winapi::shared::windef::HWND> for WindowHandle {
    fn from(hwnd: winapi::shared::windef::HWND) -> Self {
        WindowHandle(hwnd)
    }
}

#[cfg(windows)]
impl From<WindowHandle> for winapi::shared::windef::HWND {
    fn from(handle: WindowHandle) -> Self {
        handle.0
    }
}

#[cfg(not(windows))]
pub type WindowHandle = u64;

/// 窗口信息
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    /// 窗口句柄
    pub handle: WindowHandle,
    /// 窗口位置 (x, y)
    pub position: (i32, i32),
    /// 窗口大小 (width, height)
    pub size: (u32, u32),
    /// 窗口标题
    pub title: String,
    /// 进程ID
    pub process_id: u32,
    /// 是否可见
    pub is_visible: bool,
    /// 是否在前台
    pub is_foreground: bool,
}

impl WindowInfo {
    /// 创建新的窗口信息
    #[cfg(windows)]
    pub fn new(handle: winapi::shared::windef::HWND, title: String) -> Self {
        Self {
            handle: WindowHandle::from(handle),
            position: (0, 0),
            size: (0, 0),
            title,
            process_id: 0,
            is_visible: false,
            is_foreground: false,
        }
    }
    
    /// 创建新的窗口信息（非Windows）
    #[cfg(not(windows))]
    pub fn new(handle: WindowHandle, title: String) -> Self {
        Self {
            handle,
            position: (0, 0),
            size: (0, 0),
            title,
            process_id: 0,
            is_visible: false,
            is_foreground: false,
        }
    }
    
    /// 获取窗口中心点坐标
    pub fn center(&self) -> (i32, i32) {
        (
            self.position.0 + (self.size.0 as i32) / 2,
            self.position.1 + (self.size.1 as i32) / 2,
        )
    }
    
    /// 检查点是否在窗口内
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.position.0
            && x < self.position.0 + self.size.0 as i32
            && y >= self.position.1
            && y < self.position.1 + self.size.1 as i32
    }
    
    /// 将屏幕坐标转换为窗口相对坐标
    pub fn screen_to_window_coords(&self, screen_x: i32, screen_y: i32) -> Option<(i32, i32)> {
        let window_x = screen_x - self.position.0;
        let window_y = screen_y - self.position.1;
        
        if window_x >= 0 && window_x < self.size.0 as i32
            && window_y >= 0 && window_y < self.size.1 as i32
        {
            Some((window_x, window_y))
        } else {
            None
        }
    }
    
    /// 将窗口相对坐标转换为屏幕坐标
    pub fn window_to_screen_coords(&self, window_x: i32, window_y: i32) -> (i32, i32) {
        (
            self.position.0 + window_x,
            self.position.1 + window_y,
        )
    }
}

/// 窗口检测配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowDetectionConfig {
    /// 目标窗口标题（支持部分匹配）
    pub target_window_title: String,
    /// 目标进程名
    pub target_process_name: String,
    /// 检测间隔（毫秒）
    pub detection_interval_ms: u64,
    /// 是否只检测可见窗口
    pub visible_only: bool,
    /// 是否只检测前台窗口
    pub foreground_only: bool,
}

impl Default for WindowDetectionConfig {
    fn default() -> Self {
        Self {
            target_window_title: "明日方舟".to_string(),
            target_process_name: "Arknights.exe".to_string(),
            detection_interval_ms: 1000,
            visible_only: true,
            foreground_only: false,
        }
    }
}

/// UI元素类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIElementType {
    /// 干员
    Operator,
    /// 技能
    Skill,
    /// 部署按钮
    DeployButton,
    /// 暂停按钮
    PauseButton,
    /// 撤退按钮
    RetreatButton,
    /// 未知元素
    Unknown,
}

/// UI元素信息
#[derive(Debug, Clone, PartialEq)]
pub struct UIElement {
    /// 元素类型
    pub element_type: UIElementType,
    /// 位置 (x, y)
    pub position: (i32, i32),
    /// 大小 (width, height)
    pub size: (u32, u32),
    /// 识别置信度 (0.0 - 1.0)
    pub confidence: f32,
    /// 额外数据
    pub data: Option<String>,
}

impl UIElement {
    /// 创建新的UI元素
    pub fn new(element_type: UIElementType, position: (i32, i32), size: (u32, u32)) -> Self {
        Self {
            element_type,
            position,
            size,
            confidence: 1.0,
            data: None,
        }
    }
    
    /// 获取元素中心点
    pub fn center(&self) -> (i32, i32) {
        (
            self.position.0 + (self.size.0 as i32) / 2,
            self.position.1 + (self.size.1 as i32) / 2,
        )
    }
    
    /// 检查点是否在元素内
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        x >= self.position.0
            && x < self.position.0 + self.size.0 as i32
            && y >= self.position.1
            && y < self.position.1 + self.size.1 as i32
    }
}