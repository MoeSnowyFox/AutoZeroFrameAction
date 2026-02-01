//! 窗口管理服务
//! 
//! 负责检测和管理明日方舟游戏窗口

use crate::models::{WindowInfo, WindowDetectionConfig, WindowHandle};
use crate::utils::{WindowError, WindowResult};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam_channel::{Receiver, Sender, unbounded};
use log::{debug, info, warn};

#[cfg(windows)]
use winapi::{
    shared::windef::{HWND, RECT},
    um::{
        winuser::{
            EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
            GetWindowRect, GetForegroundWindow, GetDC, ReleaseDC,
        },
        wingdi::{
            BitBlt, SRCCOPY, CreateCompatibleDC, CreateCompatibleBitmap, SelectObject, 
            GetDIBits, BITMAPINFOHEADER, BITMAPINFO, DIB_RGB_COLORS, DeleteDC, DeleteObject,
        },
        processthreadsapi::{OpenProcess},
        psapi::GetModuleBaseNameW,
        handleapi::CloseHandle,
    },
};

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

/// 窗口检测事件
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// 窗口被检测到
    WindowFound(WindowInfo),
    /// 窗口丢失
    WindowLost,
    /// 窗口信息更新
    WindowUpdated(WindowInfo),
}

/// 窗口检测回调
pub type WindowCallback = Box<dyn Fn(WindowEvent) + Send + Sync>;

/// 用于枚举窗口的上下文结构
#[cfg(windows)]
struct EnumWindowsContext<'a> {
    config: &'a WindowDetectionConfig,
    found_window: &'a mut Option<WindowInfo>,
}

/// 窗口管理服务
pub struct WindowService {
    /// 当前锁定的目标窗口
    target_window: Arc<Mutex<Option<WindowInfo>>>,
    /// 窗口检测配置
    detection_config: WindowDetectionConfig,
    /// 检测是否正在运行
    detection_running: Arc<Mutex<bool>>,
    /// 检测线程句柄
    detection_thread: Option<thread::JoinHandle<()>>,
    /// 事件发送器
    event_sender: Option<Sender<WindowEvent>>,
    /// 事件接收器
    event_receiver: Option<Receiver<WindowEvent>>,
    /// 回调函数列表
    callbacks: Arc<Mutex<Vec<WindowCallback>>>,
    /// 最后一次截图时间（用于节流）
    last_capture_time: Arc<Mutex<Option<Instant>>>,
}

impl WindowService {
    /// 创建新的窗口服务
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        
        Self {
            target_window: Arc::new(Mutex::new(None)),
            detection_config: WindowDetectionConfig::default(),
            detection_running: Arc::new(Mutex::new(false)),
            detection_thread: None,
            event_sender: Some(sender),
            event_receiver: Some(receiver),
            callbacks: Arc::new(Mutex::new(Vec::new())),
            last_capture_time: Arc::new(Mutex::new(None)),
        }
    }
    
    /// 使用自定义配置创建窗口服务
    pub fn with_config(config: WindowDetectionConfig) -> Self {
        let mut service = Self::new();
        service.detection_config = config;
        service
    }
    
    /// 添加窗口事件回调
    pub fn add_callback(&mut self, callback: WindowCallback) -> WindowResult<()> {
        let mut callbacks = self.callbacks.lock()
            .map_err(|_| WindowError::SystemApiError("回调锁定失败".to_string()))?;
        callbacks.push(callback);
        Ok(())
    }
    
    /// 开始窗口检测
    pub fn start_detection(&mut self) -> WindowResult<()> {
        let mut running = self.detection_running.lock()
            .map_err(|_| WindowError::SystemApiError("状态锁定失败".to_string()))?;
        
        if *running {
            return Ok(()); // 已经在运行
        }
        
        *running = true;
        drop(running);
        
        let target_window = Arc::clone(&self.target_window);
        let detection_running = Arc::clone(&self.detection_running);
        let callbacks = Arc::clone(&self.callbacks);
        let config = self.detection_config.clone();
        let sender = self.event_sender.as_ref()
            .ok_or_else(|| WindowError::SystemApiError("事件发送器未初始化".to_string()))?
            .clone();
        
        let handle = thread::spawn(move || {
            Self::detection_loop(target_window, detection_running, callbacks, config, sender);
        });
        
        self.detection_thread = Some(handle);
        info!("窗口检测已启动");
        Ok(())
    }
    
    /// 停止窗口检测
    pub fn stop_detection(&mut self) {
        if let Ok(mut running) = self.detection_running.lock() {
            *running = false;
        }
        
        if let Some(handle) = self.detection_thread.take() {
            let _ = handle.join();
            info!("窗口检测已停止");
        }
    }
    
    /// 检测循环（在独立线程中运行）
    fn detection_loop(
        target_window: Arc<Mutex<Option<WindowInfo>>>,
        detection_running: Arc<Mutex<bool>>,
        callbacks: Arc<Mutex<Vec<WindowCallback>>>,
        config: WindowDetectionConfig,
        sender: Sender<WindowEvent>,
    ) {
        let mut last_window: Option<WindowInfo> = None;
        let detection_interval = Duration::from_millis(config.detection_interval_ms);
        
        while Self::is_detection_running(&detection_running) {
            match Self::find_target_window(&config) {
                Ok(Some(window_info)) => {
                    let window_changed = match &last_window {
                        Some(last) => last.handle != window_info.handle || 
                                     last.position != window_info.position ||
                                     last.size != window_info.size,
                        None => true,
                    };
                    
                    if window_changed {
                        // 更新目标窗口
                        if let Ok(mut target) = target_window.lock() {
                            *target = Some(window_info.clone());
                        }
                        
                        let event = if last_window.is_none() {
                            debug!("检测到明日方舟窗口: {}", window_info.title);
                            WindowEvent::WindowFound(window_info.clone())
                        } else {
                            debug!("窗口信息已更新");
                            WindowEvent::WindowUpdated(window_info.clone())
                        };
                        
                        // 发送事件
                        let _ = sender.send(event.clone());
                        
                        // 调用回调函数
                        if let Ok(callbacks_guard) = callbacks.lock() {
                            for callback in callbacks_guard.iter() {
                                callback(event.clone());
                            }
                        }
                        
                        last_window = Some(window_info);
                    }
                }
                Ok(None) => {
                    if last_window.is_some() {
                        debug!("明日方舟窗口已丢失");
                        
                        // 清除目标窗口
                        if let Ok(mut target) = target_window.lock() {
                            *target = None;
                        }
                        
                        let event = WindowEvent::WindowLost;
                        
                        // 发送事件
                        let _ = sender.send(event.clone());
                        
                        // 调用回调函数
                        if let Ok(callbacks_guard) = callbacks.lock() {
                            for callback in callbacks_guard.iter() {
                                callback(event.clone());
                            }
                        }
                        
                        last_window = None;
                    }
                }
                Err(e) => {
                    warn!("窗口检测出错: {}", e);
                }
            }
            
            thread::sleep(detection_interval);
        }
        
        debug!("窗口检测循环已退出");
    }
    
    /// 检查检测是否正在运行
    fn is_detection_running(detection_running: &Arc<Mutex<bool>>) -> bool {
        detection_running.lock().map(|r| *r).unwrap_or(false)
    }
    
    /// 查找目标窗口
    #[cfg(windows)]
    fn find_target_window(config: &WindowDetectionConfig) -> WindowResult<Option<WindowInfo>> {
        let mut found_window: Option<WindowInfo> = None;
        
        unsafe {
            let mut context = EnumWindowsContext {
                config,
                found_window: &mut found_window,
            };
            let context_ptr = &mut context as *mut EnumWindowsContext;
            
            EnumWindows(
                Some(Self::enum_windows_proc),
                context_ptr as isize,
            );
        }
        
        Ok(found_window)
    }
    
    /// 窗口枚举回调函数
    #[cfg(windows)]
    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: isize) -> i32 {
        let context = &mut *(lparam as *mut EnumWindowsContext);
        let config = context.config;
        let found_window = &mut *context.found_window;
        
        // 检查窗口是否可见
        if config.visible_only && IsWindowVisible(hwnd) == 0 {
            return 1; // 继续枚举
        }
        
        // 检查是否为前台窗口
        if config.foreground_only && GetForegroundWindow() != hwnd {
            return 1; // 继续枚举
        }
        
        // 获取窗口标题
        let mut title_buffer = [0u16; 256];
        let title_len = GetWindowTextW(hwnd, title_buffer.as_mut_ptr(), title_buffer.len() as i32);
        
        if title_len > 0 {
            let title = OsString::from_wide(&title_buffer[..title_len as usize])
                .to_string_lossy()
                .to_string();
            
            // 检查标题是否匹配
            if title.contains(&config.target_window_title) {
                // 获取进程ID和进程名
                let mut process_id = 0;
                GetWindowThreadProcessId(hwnd, &mut process_id);
                
                if let Ok(process_name) = Self::get_process_name(process_id) {
                    if process_name.to_lowercase().contains(&config.target_process_name.to_lowercase()) {
                        // 获取窗口位置和大小
                        let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                        if GetWindowRect(hwnd, &mut rect) != 0 {
                            let window_info = WindowInfo {
                                handle: WindowHandle::from(hwnd),
                                position: (rect.left, rect.top),
                                size: ((rect.right - rect.left) as u32, (rect.bottom - rect.top) as u32),
                                title,
                                process_id,
                                is_visible: IsWindowVisible(hwnd) != 0,
                                is_foreground: GetForegroundWindow() == hwnd,
                            };
                            
                            *found_window = Some(window_info);
                            return 0; // 停止枚举
                        }
                    }
                }
            }
        }
        
        1 // 继续枚举
    }
    
    /// 获取进程名称
    #[cfg(windows)]
    fn get_process_name(process_id: u32) -> WindowResult<String> {
        unsafe {
            // 使用数值常量而不是符号常量
            let process_handle = OpenProcess(0x1000, 0, process_id); // PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
            if process_handle.is_null() {
                return Err(WindowError::SystemApiError("无法打开进程".to_string()));
            }
            
            let mut module_name = [0u16; 256];
            let name_len = GetModuleBaseNameW(
                process_handle,
                std::ptr::null_mut(),
                module_name.as_mut_ptr(),
                module_name.len() as u32,
            );
            
            CloseHandle(process_handle);
            
            if name_len > 0 {
                let name = OsString::from_wide(&module_name[..name_len as usize])
                    .to_string_lossy()
                    .to_string();
                Ok(name)
            } else {
                Err(WindowError::SystemApiError("无法获取进程名称".to_string()))
            }
        }
    }
    
    /// 非Windows平台的窗口查找（占位符实现）
    #[cfg(not(windows))]
    fn find_target_window(_config: &WindowDetectionConfig) -> WindowResult<Option<WindowInfo>> {
        warn!("非Windows平台暂不支持窗口检测");
        Ok(None)
    }
    
    /// 获取当前锁定的窗口信息
    pub fn get_window_info(&self) -> Option<WindowInfo> {
        self.target_window.lock().ok()?.clone()
    }
    
    /// 检查是否有窗口被锁定
    pub fn has_window(&self) -> bool {
        self.target_window.lock()
            .map(|w| w.is_some())
            .unwrap_or(false)
    }
    
    /// 刷新窗口列表（重新检测目标窗口）
    pub fn refresh_window_list(&mut self) -> WindowResult<()> {
        // 清除当前窗口信息
        if let Ok(mut target) = self.target_window.lock() {
            *target = None;
        }
        
        // 立即尝试查找目标窗口
        if let Ok(Some(window_info)) = Self::find_target_window(&self.detection_config) {
            if let Ok(mut target) = self.target_window.lock() {
                *target = Some(window_info.clone());
            }
            
            // 发送窗口找到事件
            if let Some(sender) = &self.event_sender {
                let _ = sender.send(WindowEvent::WindowFound(window_info));
            }
        }
        
        info!("窗口列表已刷新");
        Ok(())
    }
    
    /// 按标题查找窗口
    pub fn find_window_by_title(&self, title: &str) -> Option<WindowInfo> {
        // 首先检查当前锁定的窗口
        if let Some(window) = self.get_window_info() {
            if window.title.contains(title) {
                return Some(window);
            }
        }
        
        // 创建临时配置用于搜索
        let mut search_config = self.detection_config.clone();
        search_config.target_window_title = title.to_string();
        
        // 尝试查找匹配的窗口
        Self::find_target_window(&search_config).ok().flatten()
    }
    
    /// 屏幕坐标转窗口相对坐标
    pub fn screen_to_window_coords(&self, screen_pos: (i32, i32)) -> Option<(i32, i32)> {
        let window = self.target_window.lock().ok()?.clone()?;
        window.screen_to_window_coords(screen_pos.0, screen_pos.1)
    }
    
    /// 窗口相对坐标转屏幕坐标
    pub fn window_to_screen_coords(&self, window_pos: (i32, i32)) -> Option<(i32, i32)> {
        let window = self.target_window.lock().ok()?.clone()?;
        Some(window.window_to_screen_coords(window_pos.0, window_pos.1))
    }
    
    /// 窗口截图
    pub fn capture_window(&self) -> WindowResult<opencv::core::Mat> {
        // 检查节流
        if let Ok(mut last_time) = self.last_capture_time.lock() {
            if let Some(last) = *last_time {
                let elapsed = last.elapsed();
                if elapsed < Duration::from_millis(50) { // 最小间隔50ms
                    return Err(WindowError::CaptureError("截图频率过高，请稍后重试".to_string()));
                }
            }
            *last_time = Some(Instant::now());
        }
        
        let window = self.target_window.lock()
            .map_err(|_| WindowError::SystemApiError("窗口锁定失败".to_string()))?
            .clone()
            .ok_or(WindowError::WindowNotFound)?;
        
        self.capture_window_internal(&window)
    }
    
    /// 内部窗口截图实现
    #[cfg(windows)]
    fn capture_window_internal(&self, window: &WindowInfo) -> WindowResult<opencv::core::Mat> {
        unsafe {
            let hwnd: HWND = window.handle.into();
            let window_dc = GetDC(hwnd);
            if window_dc.is_null() {
                return Err(WindowError::CaptureError("无法获取窗口设备上下文".to_string()));
            }
            
            let width = window.size.0 as i32;
            let height = window.size.1 as i32;
            
            // 创建兼容的设备上下文和位图
            let mem_dc = CreateCompatibleDC(window_dc);
            if mem_dc.is_null() {
                ReleaseDC(hwnd, window_dc);
                return Err(WindowError::CaptureError("无法创建兼容设备上下文".to_string()));
            }
            
            let bitmap = CreateCompatibleBitmap(window_dc, width, height);
            if bitmap.is_null() {
                DeleteDC(mem_dc);
                ReleaseDC(hwnd, window_dc);
                return Err(WindowError::CaptureError("无法创建兼容位图".to_string()));
            }
            
            let old_bitmap = SelectObject(mem_dc, bitmap as *mut _);
            
            // 复制窗口内容到位图
            let result = BitBlt(mem_dc, 0, 0, width, height, window_dc, 0, 0, SRCCOPY);
            if result == 0 {
                SelectObject(mem_dc, old_bitmap);
                DeleteObject(bitmap as *mut _);
                DeleteDC(mem_dc);
                ReleaseDC(hwnd, window_dc);
                return Err(WindowError::CaptureError("位图复制失败".to_string()));
            }
            
            // 创建位图信息结构
            let mut bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // 负值表示自上而下的位图
                    biPlanes: 1,
                    biBitCount: 24, // 24位RGB
                    biCompression: 0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [std::mem::zeroed(); 1],
            };
            
            // 计算图像数据大小
            let bytes_per_pixel = 3; // 24位RGB
            let row_size = ((width * bytes_per_pixel + 3) / 4) * 4; // 4字节对齐
            let image_size = (row_size * height) as usize;
            
            // 分配图像数据缓冲区
            let mut image_data = vec![0u8; image_size];
            
            // 获取位图数据
            let lines_copied = GetDIBits(
                mem_dc,
                bitmap,
                0,
                height as u32,
                image_data.as_mut_ptr() as *mut _,
                &mut bitmap_info,
                DIB_RGB_COLORS,
            );
            
            // 清理资源
            SelectObject(mem_dc, old_bitmap);
            DeleteObject(bitmap as *mut _);
            DeleteDC(mem_dc);
            ReleaseDC(hwnd, window_dc);
            
            if lines_copied == 0 {
                return Err(WindowError::CaptureError("获取位图数据失败".to_string()));
            }
            
            // 转换为OpenCV Mat
            self.convert_bgr_to_mat(image_data, width, height, row_size)
        }
    }
    
    /// 将BGR数据转换为OpenCV Mat
    #[cfg(windows)]
    fn convert_bgr_to_mat(&self, data: Vec<u8>, width: i32, height: i32, row_size: i32) -> WindowResult<opencv::core::Mat> {
        use opencv::core::{Mat, CV_8UC3};
        use opencv::prelude::*;
        
        // 直接创建RGB Mat，跳过颜色转换
        // Windows BitBlt实际上是BGR格式，但我们可以直接处理
        let mat = unsafe {
            Mat::new_rows_cols_with_data_unsafe(
                height,
                width,
                CV_8UC3,
                data.as_ptr() as *mut std::ffi::c_void,
                row_size as usize,
            ).map_err(|e| WindowError::CaptureError(format!("创建Mat失败: {}", e)))?
        };
        
        Ok(mat)
    }
    
    /// 非Windows平台的截图实现（占位符）
    #[cfg(not(windows))]
    fn capture_window_internal(&self, _window: &WindowInfo) -> WindowResult<opencv::core::Mat> {
        Err(WindowError::CaptureError("非Windows平台暂不支持窗口截图".to_string()))
    }
    
    /// 获取事件接收器
    pub fn get_event_receiver(&mut self) -> Option<Receiver<WindowEvent>> {
        self.event_receiver.take()
    }
    
    /// 手动刷新窗口信息
    pub fn refresh_window_info(&self) -> WindowResult<()> {
        if let Some(current_window) = self.get_window_info() {
            match Self::find_target_window(&self.detection_config)? {
                Some(updated_window) => {
                    if updated_window.handle == current_window.handle {
                        // 更新窗口信息
                        if let Ok(mut target) = self.target_window.lock() {
                            *target = Some(updated_window);
                        }
                    } else {
                        // 窗口句柄变化，可能是窗口重启
                        return Err(WindowError::WindowClosed);
                    }
                }
                None => {
                    return Err(WindowError::WindowNotFound);
                }
            }
        }
        Ok(())
    }
    
    /// 检查窗口是否仍然有效
    pub fn is_window_valid(&self) -> bool {
        if let Some(window) = self.get_window_info() {
            #[cfg(windows)]
            {
                unsafe {
                    let hwnd: winapi::shared::windef::HWND = window.handle.into();
                    winapi::um::winuser::IsWindow(hwnd) != 0
                }
            }
            #[cfg(not(windows))]
            {
                // 非Windows平台的占位符实现
                true
            }
        } else {
            false
        }
    }
    
    /// 获取窗口截图的缩略图
    pub fn capture_thumbnail(&self, max_width: u32, max_height: u32) -> WindowResult<opencv::core::Mat> {
        let original = self.capture_window()?;
        
        use opencv::imgproc::{resize, INTER_LINEAR};
        use opencv::core::Size;
        use opencv::prelude::*;
        
        let original_size = original.size().map_err(|e| 
            WindowError::CaptureError(format!("获取图像尺寸失败: {}", e)))?;
        
        let original_width = original_size.width as f64;
        let original_height = original_size.height as f64;
        
        // 计算缩放比例
        let scale_x = max_width as f64 / original_width;
        let scale_y = max_height as f64 / original_height;
        let scale = scale_x.min(scale_y).min(1.0); // 不放大，只缩小
        
        let new_width = (original_width * scale) as i32;
        let new_height = (original_height * scale) as i32;
        
        let mut thumbnail = opencv::core::Mat::default();
        resize(
            &original,
            &mut thumbnail,
            Size::new(new_width, new_height),
            0.0,
            0.0,
            INTER_LINEAR,
        ).map_err(|e| WindowError::CaptureError(format!("图像缩放失败: {}", e)))?;
        
        Ok(thumbnail)
    }
}

impl Drop for WindowService {
    fn drop(&mut self) {
        self.stop_detection();
    }
}

impl Default for WindowService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_window_service_creation() {
        let service = WindowService::new();
        assert!(!service.has_window());
        assert!(service.get_window_info().is_none());
    }
    
    #[test]
    fn test_window_service_with_config() {
        let config = WindowDetectionConfig {
            target_window_title: "Test Window".to_string(),
            target_process_name: "test.exe".to_string(),
            detection_interval_ms: 500,
            visible_only: false,
            foreground_only: true,
        };
        
        let service = WindowService::with_config(config.clone());
        assert_eq!(service.detection_config.target_window_title, "Test Window");
        assert_eq!(service.detection_config.detection_interval_ms, 500);
        assert!(!service.detection_config.visible_only);
        assert!(service.detection_config.foreground_only);
    }
    
    #[test]
    fn test_coordinate_conversion() {
        let mut service = WindowService::new();
        
        // 没有窗口时应该返回None
        assert!(service.screen_to_window_coords((100, 100)).is_none());
        assert!(service.window_to_screen_coords((50, 50)).is_none());
        
        // 模拟设置窗口信息
        let window_info = WindowInfo {
            #[cfg(windows)]
            handle: WindowHandle::from(std::ptr::null_mut()),
            #[cfg(not(windows))]
            handle: 0,
            position: (100, 100),
            size: (800, 600),
            title: "Test Window".to_string(),
            process_id: 1234,
            is_visible: true,
            is_foreground: false,
        };
        
        if let Ok(mut target) = service.target_window.lock() {
            *target = Some(window_info);
        }
        
        // 测试坐标转换
        assert_eq!(service.screen_to_window_coords((150, 150)), Some((50, 50)));
        assert_eq!(service.window_to_screen_coords((50, 50)), Some((150, 150)));
        
        // 测试边界情况
        assert!(service.screen_to_window_coords((50, 50)).is_none()); // 在窗口外
        assert_eq!(service.window_to_screen_coords((0, 0)), Some((100, 100))); // 窗口左上角
    }
    
    #[test]
    fn test_callback_management() {
        let mut service = WindowService::new();
        
        let callback_called = Arc::new(Mutex::new(false));
        let callback_called_clone = Arc::clone(&callback_called);
        
        let callback = Box::new(move |_event: WindowEvent| {
            if let Ok(mut called) = callback_called_clone.lock() {
                *called = true;
            }
        });
        
        assert!(service.add_callback(callback).is_ok());
    }
    
    #[test]
    fn test_capture_throttling() {
        let service = WindowService::new();
        
        // 没有窗口时应该返回错误
        assert!(service.capture_window().is_err());
        
        // 测试节流机制需要实际的窗口，这里只测试基本逻辑
        if let Ok(mut last_time) = service.last_capture_time.lock() {
            *last_time = Some(Instant::now());
        }
        
        // 立即再次尝试应该被节流
        assert!(service.capture_window().is_err());
    }
    
    #[cfg(windows)]
    #[test]
    fn test_process_name_extraction() {
        // 测试获取当前进程名称
        let current_process_id = std::process::id();
        if let Ok(process_name) = WindowService::get_process_name(current_process_id) {
            assert!(!process_name.is_empty());
            // 在测试环境中，进程名可能是cargo或测试相关的名称
            assert!(process_name.contains("cargo") || process_name.contains("test") || process_name.ends_with(".exe"));
        }
    }
    
    #[test]
    fn test_window_validation() {
        let service = WindowService::new();
        
        // 没有窗口时应该返回false
        assert!(!service.is_window_valid());
        
        // 有窗口但句柄无效时的测试需要实际的窗口句柄
        // 这里只测试基本逻辑
    }
}