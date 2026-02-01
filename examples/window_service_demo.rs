//! WindowService 演示程序
//! 
//! 演示窗口检测、坐标转换和截图功能

use arknights_macro::services::WindowService;
use arknights_macro::models::WindowDetectionConfig;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("=== WindowService 演示程序 ===");
    
    // 创建窗口检测配置
    let config = WindowDetectionConfig {
        target_window_title: "明日方舟".to_string(),
        target_process_name: "Arknights.exe".to_string(),
        detection_interval_ms: 1000,
        visible_only: true,
        foreground_only: false,
    };
    
    // 创建窗口服务
    let mut window_service = WindowService::with_config(config);
    
    // 添加窗口事件回调
    window_service.add_callback(Box::new(|event| {
        match event {
            arknights_macro::services::WindowEvent::WindowFound(window) => {
                println!("✅ 检测到明日方舟窗口:");
                println!("   标题: {}", window.title);
                println!("   位置: ({}, {})", window.position.0, window.position.1);
                println!("   大小: {}x{}", window.size.0, window.size.1);
                println!("   进程ID: {}", window.process_id);
                println!("   可见: {}", window.is_visible);
                println!("   前台: {}", window.is_foreground);
            }
            arknights_macro::services::WindowEvent::WindowLost => {
                println!("❌ 明日方舟窗口已丢失");
            }
            arknights_macro::services::WindowEvent::WindowUpdated(window) => {
                println!("🔄 窗口信息已更新:");
                println!("   位置: ({}, {})", window.position.0, window.position.1);
                println!("   大小: {}x{}", window.size.0, window.size.1);
            }
        }
    }))?;
    
    // 启动窗口检测
    println!("🔍 启动窗口检测...");
    window_service.start_detection()?;
    
    // 等待一段时间让检测运行
    println!("⏳ 等待窗口检测结果...");
    sleep(Duration::from_secs(5)).await;
    
    // 检查是否检测到窗口
    if window_service.has_window() {
        println!("\n=== 窗口功能测试 ===");
        
        if let Some(window_info) = window_service.get_window_info() {
            println!("📋 当前窗口信息:");
            println!("   标题: {}", window_info.title);
            println!("   位置: ({}, {})", window_info.position.0, window_info.position.1);
            println!("   大小: {}x{}", window_info.size.0, window_info.size.1);
            
            // 测试坐标转换
            println!("\n🎯 坐标转换测试:");
            let screen_pos = (window_info.position.0 + 100, window_info.position.1 + 100);
            if let Some(window_pos) = window_service.screen_to_window_coords(screen_pos) {
                println!("   屏幕坐标 ({}, {}) -> 窗口坐标 ({}, {})", 
                    screen_pos.0, screen_pos.1, window_pos.0, window_pos.1);
                
                if let Some(back_to_screen) = window_service.window_to_screen_coords(window_pos) {
                    println!("   窗口坐标 ({}, {}) -> 屏幕坐标 ({}, {})", 
                        window_pos.0, window_pos.1, back_to_screen.0, back_to_screen.1);
                }
            }
            
            // 测试窗口截图
            println!("\n📸 窗口截图测试:");
            match window_service.capture_window() {
                Ok(screenshot) => {
                    use opencv::prelude::*;
                    let size = screenshot.size().unwrap();
                    println!("   截图成功! 尺寸: {}x{}", size.width, size.height);
                    
                    // 测试缩略图
                    match window_service.capture_thumbnail(200, 150) {
                        Ok(thumbnail) => {
                            let thumb_size = thumbnail.size().unwrap();
                            println!("   缩略图成功! 尺寸: {}x{}", thumb_size.width, thumb_size.height);
                        }
                        Err(e) => {
                            println!("   缩略图失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("   截图失败: {}", e);
                }
            }
            
            // 测试窗口有效性
            println!("\n✅ 窗口有效性检查:");
            println!("   窗口有效: {}", window_service.is_window_valid());
        }
    } else {
        println!("❌ 未检测到明日方舟窗口");
        println!("   请确保明日方舟游戏已启动");
    }
    
    // 停止窗口检测
    println!("\n🛑 停止窗口检测...");
    window_service.stop_detection();
    
    println!("✨ 演示完成!");
    
    Ok(())
}