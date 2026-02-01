//! 操作执行服务
//! 
//! 负责执行各种操作，包括键盘按键、鼠标移动和点击等

use crate::models::{ActionType, ActionSequence, MouseButton};
use crate::utils::{ActionError, ActionResult};
use std::sync::Arc;
use std::collections::HashMap;

#[cfg(windows)]
use winapi::um::winuser::{
    SendInput, INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, MOUSEINPUT,
    KEYEVENTF_KEYUP, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_ABSOLUTE,
    VK_SPACE, VK_DELETE, VK_ESCAPE, VK_F1, VK_F2, VK_F3, VK_F4,
    GetCursorPos, SetCursorPos,
};

/// 操作执行服务
pub struct ActionService {
    /// 按键映射表
    key_map: HashMap<String, u16>,
}

impl ActionService {
    /// 创建新的操作服务
    pub fn new() -> Self {
        let mut key_map = HashMap::new();
        
        // 初始化按键映射
        key_map.insert("Space".to_string(), VK_SPACE as u16);
        key_map.insert("Delete".to_string(), VK_DELETE as u16);
        key_map.insert("Escape".to_string(), VK_ESCAPE as u16);
        key_map.insert("F".to_string(), 0x46); // F key
        key_map.insert("1".to_string(), 0x31); // 1 key
        key_map.insert("2".to_string(), 0x32); // 2 key
        key_map.insert("3".to_string(), 0x33); // 3 key
        key_map.insert("4".to_string(), 0x34); // 4 key
        
        Self { key_map }
    }
    
    /// 执行单个操作
    pub async fn execute_action(&self, action: &ActionType) -> ActionResult<()> {
        match action {
            ActionType::KeyPress(key) => {
                self.send_key_press(key).await
            }
            ActionType::MouseMove(x, y) => {
                self.send_mouse_move(*x, *y).await
            }
            ActionType::MouseClick(button, x, y) => {
                self.send_mouse_click(*button, *x, *y).await
            }
            ActionType::Wait(duration) => {
                log::info!("等待 {:?}", duration);
                tokio::time::sleep(*duration).await;
                Ok(())
            }
        }
    }
    
    /// 执行操作序列
    pub async fn execute_sequence(&self, sequence: &ActionSequence) -> ActionResult<()> {
        log::info!("开始执行操作序列: {}", sequence.name);
        
        for (index, action) in sequence.actions.iter().enumerate() {
            match self.execute_action(action).await {
                Ok(()) => {
                    log::debug!("操作 {} 执行成功", index + 1);
                }
                Err(e) => {
                    log::error!("操作 {} 执行失败: {}", index + 1, e);
                    return Err(e);
                }
            }
        }
        
        log::info!("操作序列执行完成: {}", sequence.name);
        Ok(())
    }
    
    /// 发送按键操作
    #[cfg(windows)]
    async fn send_key_press(&self, key: &str) -> ActionResult<()> {
        let vk_code = self.key_map.get(key)
            .ok_or_else(|| ActionError::InvalidKey(key.to_string()))?;
        
        log::info!("执行按键操作: {} (VK: 0x{:02X})", key, vk_code);
        
        unsafe {
            // 按下按键
            let mut input_down = INPUT {
                type_: INPUT_KEYBOARD,
                u: std::mem::zeroed(),
            };
            *input_down.u.ki_mut() = KEYBDINPUT {
                wVk: *vk_code,
                wScan: 0,
                dwFlags: 0,
                time: 0,
                dwExtraInfo: 0,
            };
            
            // 释放按键
            let mut input_up = INPUT {
                type_: INPUT_KEYBOARD,
                u: std::mem::zeroed(),
            };
            *input_up.u.ki_mut() = KEYBDINPUT {
                wVk: *vk_code,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            };
            
            let inputs = [input_down, input_up];
            let result = SendInput(
                inputs.len() as u32,
                inputs.as_ptr() as *mut INPUT,
                std::mem::size_of::<INPUT>() as i32,
            );
            
            if result != inputs.len() as u32 {
                return Err(ActionError::SystemCall(format!(
                    "SendInput failed for key: {}", key
                )));
            }
        }
        
        Ok(())
    }
    
    /// 发送鼠标移动操作
    #[cfg(windows)]
    async fn send_mouse_move(&self, x: i32, y: i32) -> ActionResult<()> {
        log::info!("执行鼠标移动: ({}, {})", x, y);
        
        unsafe {
            let result = SetCursorPos(x, y);
            if result == 0 {
                return Err(ActionError::SystemCall(format!(
                    "SetCursorPos failed for position: ({}, {})", x, y
                )));
            }
        }
        
        Ok(())
    }
    
    /// 发送鼠标点击操作
    #[cfg(windows)]
    async fn send_mouse_click(&self, button: MouseButton, x: i32, y: i32) -> ActionResult<()> {
        log::info!("执行鼠标点击: {:?} at ({}, {})", button, x, y);
        
        // 先移动鼠标到指定位置
        self.send_mouse_move(x, y).await?;
        
        // 等待一小段时间确保鼠标移动完成
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        
        let (down_flag, up_flag) = match button {
            MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
        };
        
        unsafe {
            // 按下鼠标按钮
            let mut input_down = INPUT {
                type_: INPUT_MOUSE,
                u: std::mem::zeroed(),
            };
            *input_down.u.mi_mut() = MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: down_flag,
                time: 0,
                dwExtraInfo: 0,
            };
            
            // 释放鼠标按钮
            let mut input_up = INPUT {
                type_: INPUT_MOUSE,
                u: std::mem::zeroed(),
            };
            *input_up.u.mi_mut() = MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: 0,
                dwFlags: up_flag,
                time: 0,
                dwExtraInfo: 0,
            };
            
            let inputs = [input_down, input_up];
            let result = SendInput(
                inputs.len() as u32,
                inputs.as_ptr() as *mut INPUT,
                std::mem::size_of::<INPUT>() as i32,
            );
            
            if result != inputs.len() as u32 {
                return Err(ActionError::SystemCall(format!(
                    "SendInput failed for mouse click: {:?}", button
                )));
            }
        }
        
        Ok(())
    }
    
    /// 非Windows平台的按键操作实现（占位符）
    #[cfg(not(windows))]
    async fn send_key_press(&self, key: &str) -> ActionResult<()> {
        log::warn!("按键操作在非Windows平台上不支持: {}", key);
        Err(ActionError::UnsupportedPlatform("Key press not supported on this platform".to_string()))
    }
    
    /// 非Windows平台的鼠标移动实现（占位符）
    #[cfg(not(windows))]
    async fn send_mouse_move(&self, x: i32, y: i32) -> ActionResult<()> {
        log::warn!("鼠标移动在非Windows平台上不支持: ({}, {})", x, y);
        Err(ActionError::UnsupportedPlatform("Mouse move not supported on this platform".to_string()))
    }
    
    /// 非Windows平台的鼠标点击实现（占位符）
    #[cfg(not(windows))]
    async fn send_mouse_click(&self, button: MouseButton, x: i32, y: i32) -> ActionResult<()> {
        log::warn!("鼠标点击在非Windows平台上不支持: {:?} at ({}, {})", button, x, y);
        Err(ActionError::UnsupportedPlatform("Mouse click not supported on this platform".to_string()))
    }
    
    /// 获取当前鼠标位置
    #[cfg(windows)]
    pub fn get_cursor_position(&self) -> ActionResult<(i32, i32)> {
        unsafe {
            let mut point = winapi::shared::windef::POINT { x: 0, y: 0 };
            let result = GetCursorPos(&mut point);
            if result == 0 {
                return Err(ActionError::SystemCall("GetCursorPos failed".to_string()));
            }
            Ok((point.x, point.y))
        }
    }
    
    /// 非Windows平台获取鼠标位置（占位符）
    #[cfg(not(windows))]
    pub fn get_cursor_position(&self) -> ActionResult<(i32, i32)> {
        Err(ActionError::UnsupportedPlatform("Get cursor position not supported on this platform".to_string()))
    }
    
    /// 添加自定义按键映射
    pub fn add_key_mapping(&mut self, key: String, vk_code: u16) {
        self.key_map.insert(key, vk_code);
    }
    
    /// 获取支持的按键列表
    pub fn get_supported_keys(&self) -> Vec<String> {
        self.key_map.keys().cloned().collect()
    }
}