//! StateManager 演示程序
//! 
//! 展示状态管理器的基本功能，包括状态转换、观察者模式和事件广播

use arknights_macro::services::{StateManager, StateObserver, StatePersistenceConfig};
use arknights_macro::models::{ProgramState, GameState};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;

// 演示用的观察者实现
struct DemoObserver {
    name: String,
    program_state_changes: Arc<AtomicUsize>,
    game_state_changes: Arc<AtomicUsize>,
}

impl DemoObserver {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            program_state_changes: Arc::new(AtomicUsize::new(0)),
            game_state_changes: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    fn get_program_changes(&self) -> usize {
        self.program_state_changes.load(Ordering::SeqCst)
    }
    
    fn get_game_changes(&self) -> usize {
        self.game_state_changes.load(Ordering::SeqCst)
    }
}

impl StateObserver for DemoObserver {
    fn on_program_state_changed(&self, old_state: ProgramState, new_state: ProgramState) {
        self.program_state_changes.fetch_add(1, Ordering::SeqCst);
        println!("[{}] 程序状态变更: {} -> {}", self.name, old_state, new_state);
    }

    fn on_game_state_changed(&self, old_state: GameState, new_state: GameState) {
        self.game_state_changes.fetch_add(1, Ordering::SeqCst);
        println!("[{}] 游戏状态变更: {} -> {}", self.name, old_state, new_state);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    println!("=== StateManager 演示程序 ===\n");
    
    // 演示1: 基本状态管理
    println!("1. 基本状态管理演示");
    demo_basic_state_management().await?;
    
    println!("\n{}\n", "=".repeat(50));
    
    // 演示2: 观察者模式
    println!("2. 观察者模式演示");
    demo_observer_pattern().await?;
    
    println!("\n{}\n", "=".repeat(50));
    
    // 演示3: 事件广播
    println!("3. 事件广播演示");
    demo_event_broadcasting().await?;
    
    println!("\n{}\n", "=".repeat(50));
    
    // 演示4: 状态持久化
    println!("4. 状态持久化演示");
    demo_state_persistence().await?;
    
    println!("\n演示完成！");
    Ok(())
}

async fn demo_basic_state_management() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = StateManager::new();
    
    // 检查初始状态
    let initial_state = manager.get_state().await;
    println!("初始状态: 程序={}, 游戏={}", initial_state.program_state, initial_state.game_state);
    
    // 尝试启动核心功能（应该失败，因为游戏未检测到）
    println!("尝试启动核心功能（游戏未检测）...");
    match manager.start_core().await {
        Ok(_) => println!("启动成功"),
        Err(e) => println!("启动失败: {}", e),
    }
    
    // 更新游戏状态为已检测
    println!("更新游戏状态为已检测...");
    manager.update_game_state(GameState::Detected).await;
    
    // 现在应该可以启动核心功能
    println!("再次尝试启动核心功能...");
    match manager.start_core().await {
        Ok(_) => println!("启动成功！"),
        Err(e) => println!("启动失败: {}", e),
    }
    
    // 检查当前状态
    let current_state = manager.get_state().await;
    println!("当前状态: 程序={}, 游戏={}", current_state.program_state, current_state.game_state);
    
    // 停止核心功能
    println!("停止核心功能...");
    match manager.stop_core().await {
        Ok(_) => println!("停止成功！"),
        Err(e) => println!("停止失败: {}", e),
    }
    
    // 检查最终状态
    let final_state = manager.get_state().await;
    println!("最终状态: 程序={}, 游戏={}", final_state.program_state, final_state.game_state);
    
    Ok(())
}

async fn demo_observer_pattern() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = StateManager::new();
    
    // 创建观察者
    let observer1 = Arc::new(DemoObserver::new("观察者1"));
    let observer2 = Arc::new(DemoObserver::new("观察者2"));
    
    // 添加观察者
    manager.add_observer(observer1.clone());
    manager.add_observer(observer2.clone());
    
    println!("已添加2个观察者");
    
    // 触发状态变更
    println!("触发游戏状态变更...");
    manager.update_game_state(GameState::Detected).await;
    manager.update_game_state(GameState::InBattle).await;
    
    println!("触发程序状态变更...");
    manager.start_core().await?;
    manager.stop_core().await?;
    
    // 检查观察者收到的通知数量
    println!("观察者1收到的通知: 程序状态变更={}, 游戏状态变更={}", 
             observer1.get_program_changes(), observer1.get_game_changes());
    println!("观察者2收到的通知: 程序状态变更={}, 游戏状态变更={}", 
             observer2.get_program_changes(), observer2.get_game_changes());
    
    Ok(())
}

async fn demo_event_broadcasting() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = StateManager::new();
    let mut event_receiver = manager.subscribe_events();
    
    // 在后台任务中监听事件
    let event_count = Arc::new(AtomicUsize::new(0));
    let event_count_clone = Arc::clone(&event_count);
    
    let listener_handle = tokio::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            let count = event_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
            match event {
                arknights_macro::services::StateChangeEvent::ProgramStateChanged { old_state, new_state, .. } => {
                    println!("[事件监听器] 程序状态变更事件 #{}: {} -> {}", count, old_state, new_state);
                }
                arknights_macro::services::StateChangeEvent::GameStateChanged { old_state, new_state, .. } => {
                    println!("[事件监听器] 游戏状态变更事件 #{}: {} -> {}", count, old_state, new_state);
                }
            }
        }
    });
    
    // 触发一些状态变更
    println!("触发状态变更事件...");
    manager.update_game_state(GameState::Detected).await;
    manager.start_core().await?;
    manager.update_game_state(GameState::InBattle).await;
    manager.stop_core().await?;
    
    // 等待事件处理
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // 停止监听器
    listener_handle.abort();
    
    println!("总共收到 {} 个事件", event_count.load(Ordering::SeqCst));
    
    Ok(())
}

async fn demo_state_persistence() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let state_file = temp_dir.path().join("demo_state.json");
    
    println!("状态文件路径: {:?}", state_file);
    
    // 创建带持久化的状态管理器
    let persistence_config = StatePersistenceConfig::new(state_file.clone())
        .with_auto_save_interval(2); // 2秒自动保存
    
    let mut manager = StateManager::with_persistence(persistence_config);
    manager.initialize().await?;
    
    // 更改状态
    println!("更改状态...");
    manager.update_game_state(GameState::Detected).await;
    manager.start_core().await?;
    manager.update_game_state(GameState::InBattle).await;
    
    // 强制保存状态
    println!("强制保存状态...");
    manager.force_save_state().await?;
    
    // 验证文件存在
    if state_file.exists() {
        println!("状态文件已创建");
        let content = tokio::fs::read_to_string(&state_file).await?;
        println!("状态文件内容:\n{}", content);
    } else {
        println!("状态文件未创建");
    }
    
    // 关闭管理器
    manager.shutdown().await?;
    
    // 创建新的管理器并加载状态
    println!("创建新的管理器并加载状态...");
    let mut new_manager = StateManager::with_persistence(
        StatePersistenceConfig::new(state_file)
    );
    new_manager.initialize().await?;
    
    // 验证状态被正确加载
    let loaded_state = new_manager.get_state().await;
    println!("加载的状态: 程序={}, 游戏={}", loaded_state.program_state, loaded_state.game_state);
    
    // 清理
    new_manager.shutdown().await?;
    
    Ok(())
}