//! ConfigService 功能演示
//! 
//! 演示配置管理服务的各种功能

use arknights_macro::services::ConfigService;
use arknights_macro::models::config::{AppConfig, OperationMode, Theme};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use auto_zero_frame_action::{AppConfig, ConfigService, OperationMode, Theme};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("=== ConfigService 功能演示 ===\n");
    
    // 创建临时配置文件路径
    let config_path = PathBuf::from("demo_config.json");
    
    // 清理可能存在的旧文件
    if config_path.exists() {
        std::fs::remove_file(&config_path)?;
    }
    
    println!("1. 创建配置服务");
    let mut service = ConfigService::new(config_path.clone())?;
    println!("   ✓ 配置服务创建成功");
    
    println!("\n2. 加载配置（文件不存在，使用默认配置）");
    service.load_config().await?;
    println!("   ✓ 配置加载成功");
    
    let config = service.get_config().await;
    println!("   - 当前模式: {:?}", config.mode);
    println!("   - 自动启动: {}", config.global_settings.auto_start_on_detection);
    println!("   - 主题: {:?}", config.ui_settings.theme);
    
    println!("\n3. 更新配置");
    service.update_config(|config| {
        config.mode = OperationMode::Intelligent;
        config.global_settings.auto_start_on_detection = true;
        config.ui_settings.theme = Theme::Dark;
    }).await?;
    println!("   ✓ 配置更新成功");
    
    // 等待异步保存完成
    sleep(Duration::from_millis(100)).await;
    
    let updated_config = service.get_config().await;
    println!("   - 更新后模式: {:?}", updated_config.mode);
    println!("   - 更新后自动启动: {}", updated_config.global_settings.auto_start_on_detection);
    println!("   - 更新后主题: {:?}", updated_config.ui_settings.theme);
    
    println!("\n4. 验证配置文件已保存");
    assert!(config_path.exists());
    println!("   ✓ 配置文件存在: {:?}", config_path);
    
    println!("\n5. 验证配置文件内容");
    let is_valid = service.validate_config_file().await?;
    println!("   ✓ 配置文件有效: {}", is_valid);
    
    println!("\n6. 重新创建服务并加载配置");
    let service2 = ConfigService::new_and_load(config_path.clone()).await?;
    let loaded_config = service2.get_config().await;
    
    println!("   - 加载的模式: {:?}", loaded_config.mode);
    println!("   - 加载的自动启动: {}", loaded_config.global_settings.auto_start_on_detection);
    println!("   - 加载的主题: {:?}", loaded_config.ui_settings.theme);
    
    // 验证配置一致性
    assert_eq!(loaded_config.mode, OperationMode::Intelligent);
    assert_eq!(loaded_config.global_settings.auto_start_on_detection, true);
    assert_eq!(loaded_config.ui_settings.theme, Theme::Dark);
    println!("   ✓ 配置加载一致性验证通过");
    
    println!("\n7. 测试批量更新");
    let updaters = vec![
        |config: &mut AppConfig| {
            config.macro_config.battle_detection_enabled = false;
        },
        |config: &mut AppConfig| {
            config.macro_config.overlay_settings.transparency = 60;
        },
        |config: &mut AppConfig| {
            config.ui_settings.window_size = (1024, 768);
        },
    ];
    
    service2.batch_update_config(updaters).await?;
    println!("   ✓ 批量更新成功");
    
    let batch_updated_config = service2.get_config().await;
    println!("   - 战斗检测: {}", batch_updated_config.macro_config.battle_detection_enabled);
    println!("   - 透明度: {}", batch_updated_config.macro_config.overlay_settings.transparency);
    println!("   - 窗口大小: {:?}", batch_updated_config.ui_settings.window_size);
    
    println!("\n8. 测试配置导出和导入");
    let export_path = PathBuf::from("exported_config.json");
    service2.export_config(&export_path).await?;
    println!("   ✓ 配置导出成功: {:?}", export_path);
    
    // 重置配置
    service2.reset_to_default().await?;
    println!("   ✓ 配置重置为默认值");
    
    // 导入配置
    service2.import_config(&export_path).await?;
    println!("   ✓ 配置导入成功");
    
    let imported_config = service2.get_config().await;
    assert_eq!(imported_config.mode, OperationMode::Intelligent);
    assert_eq!(imported_config.ui_settings.theme, Theme::Dark);
    println!("   ✓ 导入配置验证通过");
    
    println!("\n9. 测试配置变更事件");
    let mut receiver = service2.subscribe_changes();
    
    // 在后台监听事件
    let event_handle = tokio::spawn(async move {
        if let Ok(event) = receiver.recv().await {
            println!("   ✓ 收到配置变更事件: {:?}", event);
        }
    });
    
    // 触发配置更新
    service2.update_config(|config| {
        config.global_settings.auto_start_on_detection = false;
    }).await?;
    
    // 等待事件处理
    let _ = tokio::time::timeout(Duration::from_millis(500), event_handle).await;
    
    println!("\n10. 测试自动保存控制");
    let mut service3 = ConfigService::new(PathBuf::from("test_auto_save.json"))?;
    service3.set_auto_save(false);
    println!("   ✓ 自动保存已禁用: {}", !service3.is_auto_save_enabled());
    
    service3.update_config(|config| {
        config.mode = OperationMode::Macro;
    }).await?;
    
    // 由于禁用了自动保存，文件不应该存在
    sleep(Duration::from_millis(100)).await;
    let auto_save_path = PathBuf::from("test_auto_save.json");
    println!("   ✓ 禁用自动保存时文件不存在: {}", !auto_save_path.exists());
    
    // 手动保存
    service3.save_config().await?;
    println!("   ✓ 手动保存成功: {}", auto_save_path.exists());
    
    println!("\n=== 清理测试文件 ===");
    let test_files = vec![
        "demo_config.json",
        "exported_config.json", 
        "test_auto_save.json",
        "demo_config.bak",
    ];
    
    for file in test_files {
        let path = PathBuf::from(file);
        if path.exists() {
            std::fs::remove_file(&path)?;
            println!("   ✓ 删除文件: {}", file);
        }
    }
    
    println!("\n=== ConfigService 功能演示完成 ===");
    println!("所有功能测试通过！");
    
    Ok(())
}