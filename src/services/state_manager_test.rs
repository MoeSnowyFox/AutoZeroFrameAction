//! StateManager 单元测试
//! 
//! 独立的测试文件，不依赖OpenCV，专门测试StateManager功能

#[cfg(test)]
mod tests {
    use super::super::state_manager::*;
    use crate::models::{ProgramState, GameState, AppState};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;
    use std::path::PathBuf;

    // 测试用的观察者实现
    struct TestObserver {
        program_state_changes: Arc<AtomicUsize>,
        game_state_changes: Arc<AtomicUsize>,
    }

    impl TestObserver {
        fn new() -> Self {
            Self {
                program_state_changes: Arc::new(AtomicUsize::new(0)),
                game_state_changes: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl StateObserver for TestObserver {
        fn on_program_state_changed(&self, _old_state: ProgramState, _new_state: ProgramState) {
            self.program_state_changes.fetch_add(1, Ordering::SeqCst);
        }

        fn on_game_state_changed(&self, _old_state: GameState, _new_state: GameState) {
            self.game_state_changes.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn test_state_manager_creation() {
        let manager = StateManager::new();
        let state = manager.get_state().await;
        
        assert_eq!(state.program_state, ProgramState::Stopped);
        assert_eq!(state.game_state, GameState::NotDetected);
    }

    #[tokio::test]
    async fn test_state_manager_with_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test_state.json");
        
        let persistence_config = StatePersistenceConfig::new(state_file.clone());
        let manager = StateManager::with_persistence(persistence_config);
        
        let state = manager.get_state().await;
        assert_eq!(state.program_state, ProgramState::Stopped);
        assert_eq!(state.game_state, GameState::NotDetected);
    }

    #[tokio::test]
    async fn test_basic_state_transitions() {
        let mut manager = StateManager::new();
        
        // 初始状态应该是停止状态
        assert_eq!(manager.get_program_state().await, ProgramState::Stopped);
        assert!(!manager.can_start_core().await); // 没有检测到游戏，不能启动
        
        // 更新游戏状态为已检测
        manager.update_game_state(GameState::Detected).await;
        assert!(manager.can_start_core().await); // 现在可以启动了
        
        // 启动核心功能
        manager.start_core().await.unwrap();
        assert_eq!(manager.get_program_state().await, ProgramState::Running);
        assert!(manager.can_stop_core().await);
        
        // 停止核心功能
        manager.stop_core().await.unwrap();
        assert_eq!(manager.get_program_state().await, ProgramState::Stopped);
    }

    #[tokio::test]
    async fn test_invalid_state_transitions() {
        let mut manager = StateManager::new();
        
        // 尝试在没有检测到游戏时启动核心功能
        let result = manager.start_core().await;
        assert!(result.is_err());
        
        // 尝试在停止状态时停止核心功能
        let result = manager.stop_core().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_observer_notifications() {
        let mut manager = StateManager::new();
        let observer = Arc::new(TestObserver::new());
        
        let program_changes = Arc::clone(&observer.program_state_changes);
        let game_changes = Arc::clone(&observer.game_state_changes);
        
        manager.add_observer(observer);
        
        // 更新游戏状态
        manager.update_game_state(GameState::Detected).await;
        assert_eq!(game_changes.load(Ordering::SeqCst), 1);
        
        // 启动和停止核心功能
        manager.start_core().await.unwrap();
        manager.stop_core().await.unwrap();
        
        // 应该有4次程序状态变更：Stopped->Starting, Starting->Running, Running->Stopping, Stopping->Stopped
        assert_eq!(program_changes.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn test_multiple_observers() {
        let mut manager = StateManager::new();
        let observer1 = Arc::new(TestObserver::new());
        let observer2 = Arc::new(TestObserver::new());
        
        let program_changes1 = Arc::clone(&observer1.program_state_changes);
        let program_changes2 = Arc::clone(&observer2.program_state_changes);
        
        manager.add_observer(observer1);
        manager.add_observer(observer2);
        
        // 触发状态变更
        manager.update_game_state(GameState::Detected).await;
        manager.start_core().await.unwrap();
        
        // 两个观察者都应该收到通知
        assert_eq!(program_changes1.load(Ordering::SeqCst), 2); // Starting + Running
        assert_eq!(program_changes2.load(Ordering::SeqCst), 2); // Starting + Running
    }

    #[tokio::test]
    async fn test_event_broadcasting() {
        let mut manager = StateManager::new();
        let mut event_receiver = manager.subscribe_events();
        
        // 在后台任务中监听事件
        let event_count = Arc::new(AtomicUsize::new(0));
        let event_count_clone = Arc::clone(&event_count);
        
        let listener_handle = tokio::spawn(async move {
            while let Ok(_event) = event_receiver.recv().await {
                event_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        
        // 触发一些状态变更
        manager.update_game_state(GameState::Detected).await;
        manager.start_core().await.unwrap();
        manager.stop_core().await.unwrap();
        
        // 等待事件处理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // 停止监听器
        listener_handle.abort();
        
        // 应该收到5个事件：1个游戏状态变更 + 4个程序状态变更
        assert_eq!(event_count.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_state_persistence_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("test_state.json");
        
        let persistence_config = StatePersistenceConfig::new(state_file.clone())
            .with_auto_save_interval(1); // 1秒自动保存
        
        // 创建带持久化的状态管理器
        let mut manager = StateManager::with_persistence(persistence_config);
        manager.initialize().await.unwrap();
        
        // 更改状态
        manager.update_game_state(GameState::Detected).await;
        manager.start_core().await.unwrap();
        
        // 强制保存状态
        manager.force_save_state().await.unwrap();
        
        // 验证文件存在
        assert!(state_file.exists());
        
        // 关闭管理器
        manager.shutdown().await.unwrap();
        
        // 创建新的管理器并加载状态
        let mut new_manager = StateManager::with_persistence(
            StatePersistenceConfig::new(state_file)
        );
        new_manager.initialize().await.unwrap();
        
        // 验证状态被正确加载
        assert_eq!(new_manager.get_game_state().await, GameState::Detected);
        assert_eq!(new_manager.get_program_state().await, ProgramState::Running);
        
        // 清理
        new_manager.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_config_disable_logic() {
        let mut manager = StateManager::new();
        
        // 停止状态时配置应该可用
        assert!(!manager.should_disable_config().await);
        
        // 启动后配置应该被禁用
        manager.update_game_state(GameState::Detected).await;
        manager.start_core().await.unwrap();
        assert!(manager.should_disable_config().await);
        
        // 停止后配置应该重新可用
        manager.stop_core().await.unwrap();
        assert!(!manager.should_disable_config().await);
    }

    #[tokio::test]
    async fn test_game_state_updates() {
        let mut manager = StateManager::new();
        
        // 测试各种游戏状态转换
        assert_eq!(manager.get_game_state().await, GameState::NotDetected);
        
        manager.update_game_state(GameState::Detected).await;
        assert_eq!(manager.get_game_state().await, GameState::Detected);
        
        manager.update_game_state(GameState::InBattle).await;
        assert_eq!(manager.get_game_state().await, GameState::InBattle);
        
        manager.update_game_state(GameState::NotDetected).await;
        assert_eq!(manager.get_game_state().await, GameState::NotDetected);
    }

    #[tokio::test]
    async fn test_persistence_config() {
        let config = StatePersistenceConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.auto_save_interval, 60);
        
        let custom_config = StatePersistenceConfig::new(PathBuf::from("custom.json"))
            .with_auto_save_interval(30)
            .without_auto_save();
        
        assert!(custom_config.enabled);
        assert_eq!(custom_config.auto_save_interval, 0);
        assert_eq!(custom_config.state_file_path, PathBuf::from("custom.json"));
    }

    #[tokio::test]
    async fn test_observer_removal() {
        let mut manager = StateManager::new();
        let observer1 = Arc::new(TestObserver::new());
        let observer2 = Arc::new(TestObserver::new());
        
        manager.add_observer(observer1);
        manager.add_observer(observer2);
        
        // 移除第一个观察者
        assert!(manager.remove_observer(0));
        
        // 尝试移除不存在的观察者
        assert!(!manager.remove_observer(10));
    }

    #[tokio::test]
    async fn test_state_manager_shutdown() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("shutdown_test.json");
        
        let persistence_config = StatePersistenceConfig::new(state_file.clone());
        let mut manager = StateManager::with_persistence(persistence_config);
        
        manager.initialize().await.unwrap();
        manager.update_game_state(GameState::Detected).await;
        
        // 关闭应该成功
        manager.shutdown().await.unwrap();
        
        // 状态文件应该被创建
        assert!(state_file.exists());
    }

    #[test]
    fn test_state_change_event_serialization() {
        let event = StateChangeEvent::ProgramStateChanged {
            old_state: ProgramState::Stopped,
            new_state: ProgramState::Running,
            timestamp: std::time::SystemTime::now(),
        };
        
        // 测试序列化
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(!serialized.is_empty());
        
        // 测试反序列化
        let deserialized: StateChangeEvent = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            StateChangeEvent::ProgramStateChanged { old_state, new_state, .. } => {
                assert_eq!(old_state, ProgramState::Stopped);
                assert_eq!(new_state, ProgramState::Running);
            }
            _ => panic!("Wrong event type"),
        }
    }
}