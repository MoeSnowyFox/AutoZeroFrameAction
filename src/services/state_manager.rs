//! 状态管理器
//! 
//! 负责管理程序的运行状态，包括核心功能的启动和停止，
//! 以及状态变更通知和状态持久化功能

use crate::models::{AppState, ProgramState, GameState};
use crate::utils::{StateError, StateResult};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// 状态变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateChangeEvent {
    /// 程序状态变更
    ProgramStateChanged {
        old_state: ProgramState,
        new_state: ProgramState,
        timestamp: std::time::SystemTime,
    },
    /// 游戏状态变更
    GameStateChanged {
        old_state: GameState,
        new_state: GameState,
        timestamp: std::time::SystemTime,
    },
}

/// 状态观察者trait
pub trait StateObserver: Send + Sync {
    fn on_program_state_changed(&self, old_state: ProgramState, new_state: ProgramState);
    fn on_game_state_changed(&self, old_state: GameState, new_state: GameState);
}

/// 状态持久化配置
#[derive(Debug, Clone)]
pub struct StatePersistenceConfig {
    /// 是否启用状态持久化
    pub enabled: bool,
    /// 状态文件路径
    pub state_file_path: PathBuf,
    /// 自动保存间隔（秒）
    pub auto_save_interval: u64,
}

/// 状态管理器
pub struct StateManager {
    /// 应用程序状态
    state: Arc<RwLock<AppState>>,
    /// 状态观察者列表
    observers: Vec<Arc<dyn StateObserver>>,
    /// 状态变更事件广播器
    event_sender: broadcast::Sender<StateChangeEvent>,
    /// 状态持久化配置
    persistence_config: Option<StatePersistenceConfig>,
    /// 自动保存任务句柄
    auto_save_handle: Option<tokio::task::JoinHandle<()>>,
}

impl StateManager {
    /// 创建新的状态管理器
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        
        Self {
            state: Arc::new(RwLock::new(AppState::new())),
            observers: Vec::new(),
            event_sender,
            persistence_config: None,
            auto_save_handle: None,
        }
    }
    
    /// 创建带持久化配置的状态管理器
    pub fn with_persistence(persistence_config: StatePersistenceConfig) -> Self {
        let (event_sender, _) = broadcast::channel(100);
        
        Self {
            state: Arc::new(RwLock::new(AppState::new())),
            observers: Vec::new(),
            event_sender,
            persistence_config: Some(persistence_config),
            auto_save_handle: None,
        }
    }
    
    /// 初始化状态管理器（加载持久化状态）
    pub async fn initialize(&mut self) -> StateResult<()> {
        if let Some(config) = &self.persistence_config {
            if config.enabled {
                self.load_state().await?;
                self.start_auto_save().await?;
            }
        }
        
        log::info!("状态管理器初始化完成");
        Ok(())
    }
    
    /// 关闭状态管理器
    pub async fn shutdown(&mut self) -> StateResult<()> {
        // 停止自动保存任务
        if let Some(handle) = self.auto_save_handle.take() {
            handle.abort();
        }
        
        // 保存最终状态
        if let Some(config) = &self.persistence_config {
            if config.enabled {
                self.save_state().await?;
            }
        }
        
        log::info!("状态管理器已关闭");
        Ok(())
    }
    
    /// 启动核心功能
    pub async fn start_core(&mut self) -> StateResult<()> {
        let mut state = self.state.write().await;
        
        if !state.can_start_core() {
            return Err(StateError::InvalidTransition {
                from: state.program_state.to_string(),
                to: ProgramState::Starting.to_string(),
            });
        }
        
        let old_state = state.program_state;
        state.update_program_state(ProgramState::Starting);
        
        // 发送状态变更事件
        let event = StateChangeEvent::ProgramStateChanged {
            old_state,
            new_state: ProgramState::Starting,
            timestamp: std::time::SystemTime::now(),
        };
        let _ = self.event_sender.send(event);
        
        // 通知观察者
        for observer in &self.observers {
            observer.on_program_state_changed(old_state, ProgramState::Starting);
        }
        
        // TODO: 实际启动核心功能的逻辑
        // 这里应该调用其他服务来启动核心功能
        
        // 模拟启动过程
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        state.update_program_state(ProgramState::Running);
        
        // 发送运行状态事件
        let event = StateChangeEvent::ProgramStateChanged {
            old_state: ProgramState::Starting,
            new_state: ProgramState::Running,
            timestamp: std::time::SystemTime::now(),
        };
        let _ = self.event_sender.send(event);
        
        // 通知观察者
        for observer in &self.observers {
            observer.on_program_state_changed(ProgramState::Starting, ProgramState::Running);
        }
        
        log::info!("核心功能已启动");
        Ok(())
    }
    
    /// 停止核心功能
    pub async fn stop_core(&mut self) -> StateResult<()> {
        let mut state = self.state.write().await;
        
        if !state.can_stop_core() {
            return Err(StateError::InvalidTransition {
                from: state.program_state.to_string(),
                to: ProgramState::Stopping.to_string(),
            });
        }
        
        let old_state = state.program_state;
        state.update_program_state(ProgramState::Stopping);
        
        // 发送状态变更事件
        let event = StateChangeEvent::ProgramStateChanged {
            old_state,
            new_state: ProgramState::Stopping,
            timestamp: std::time::SystemTime::now(),
        };
        let _ = self.event_sender.send(event);
        
        // 通知观察者
        for observer in &self.observers {
            observer.on_program_state_changed(old_state, ProgramState::Stopping);
        }
        
        // TODO: 实际停止核心功能的逻辑
        // 这里应该调用其他服务来停止核心功能
        
        // 模拟停止过程
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        state.update_program_state(ProgramState::Stopped);
        
        // 发送停止状态事件
        let event = StateChangeEvent::ProgramStateChanged {
            old_state: ProgramState::Stopping,
            new_state: ProgramState::Stopped,
            timestamp: std::time::SystemTime::now(),
        };
        let _ = self.event_sender.send(event);
        
        // 通知观察者
        for observer in &self.observers {
            observer.on_program_state_changed(ProgramState::Stopping, ProgramState::Stopped);
        }
        
        log::info!("核心功能已停止");
        Ok(())
    }
    
    /// 暂停核心功能
    pub async fn pause_core(&mut self) -> StateResult<()> {
        let state = self.state.read().await;
        
        if state.program_state != ProgramState::Running {
            return Err(StateError::InvalidTransition {
                from: state.program_state.to_string(),
                to: "Paused".to_string(),
            });
        }
        
        // TODO: 实际暂停核心功能的逻辑
        log::info!("核心功能已暂停（注意：暂停功能尚未完全实现）");
        Ok(())
    }
    
    /// 更新游戏状态
    pub async fn update_game_state(&mut self, new_state: GameState) {
        let mut state = self.state.write().await;
        let old_state = state.game_state;
        
        if old_state != new_state {
            state.update_game_state(new_state);
            
            // 发送状态变更事件
            let event = StateChangeEvent::GameStateChanged {
                old_state,
                new_state,
                timestamp: std::time::SystemTime::now(),
            };
            let _ = self.event_sender.send(event);
            
            // 通知观察者
            for observer in &self.observers {
                observer.on_game_state_changed(old_state, new_state);
            }
            
            log::info!("游戏状态更新: {} -> {}", old_state, new_state);
        }
    }
    
    /// 获取当前状态
    pub async fn get_state(&self) -> AppState {
        self.state.read().await.clone()
    }
    
    /// 获取程序状态
    pub async fn get_program_state(&self) -> ProgramState {
        self.state.read().await.program_state
    }
    
    /// 获取游戏状态
    pub async fn get_game_state(&self) -> GameState {
        self.state.read().await.game_state
    }
    
    /// 检查是否可以启动核心功能
    pub async fn can_start_core(&self) -> bool {
        self.state.read().await.can_start_core()
    }
    
    /// 检查是否可以停止核心功能
    pub async fn can_stop_core(&self) -> bool {
        self.state.read().await.can_stop_core()
    }
    
    /// 检查配置是否应该被禁用
    pub async fn should_disable_config(&self) -> bool {
        self.state.read().await.should_disable_config()
    }
    
    /// 添加状态观察者
    pub fn add_observer(&mut self, observer: Arc<dyn StateObserver>) {
        self.observers.push(observer);
        log::debug!("添加状态观察者，当前观察者数量: {}", self.observers.len());
    }
    
    /// 移除状态观察者
    pub fn remove_observer(&mut self, observer_id: usize) -> bool {
        if observer_id < self.observers.len() {
            self.observers.remove(observer_id);
            log::debug!("移除状态观察者，当前观察者数量: {}", self.observers.len());
            true
        } else {
            false
        }
    }
    
    /// 获取状态变更事件接收器
    pub fn subscribe_events(&self) -> broadcast::Receiver<StateChangeEvent> {
        self.event_sender.subscribe()
    }
    
    /// 强制触发状态保存
    pub async fn force_save_state(&self) -> StateResult<()> {
        if let Some(config) = &self.persistence_config {
            if config.enabled {
                self.save_state().await?;
                log::info!("强制保存状态完成");
            }
        }
        Ok(())
    }

    /// 私有方法：加载持久化状态
    async fn load_state(&self) -> StateResult<()> {
        if let Some(config) = &self.persistence_config {
            if config.state_file_path.exists() {
                match tokio::fs::read_to_string(&config.state_file_path).await {
                    Ok(content) => {
                        match serde_json::from_str::<AppState>(&content) {
                            Ok(loaded_state) => {
                                let mut state = self.state.write().await;
                                *state = loaded_state;
                                log::info!("成功加载持久化状态");
                            }
                            Err(e) => {
                                log::warn!("解析状态文件失败: {}, 使用默认状态", e);
                                return Err(StateError::PersistenceError(format!("解析状态文件失败: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("读取状态文件失败: {}, 使用默认状态", e);
                        return Err(StateError::PersistenceError(format!("读取状态文件失败: {}", e)));
                    }
                }
            } else {
                log::info!("状态文件不存在，使用默认状态");
            }
        }
        Ok(())
    }
    
    /// 私有方法：保存状态到文件
    async fn save_state(&self) -> StateResult<()> {
        if let Some(config) = &self.persistence_config {
            let state = self.state.read().await;
            
            // 创建状态文件的父目录
            if let Some(parent) = config.state_file_path.parent() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    return Err(StateError::PersistenceError(format!("创建状态目录失败: {}", e)));
                }
            }
            
            match serde_json::to_string_pretty(&*state) {
                Ok(content) => {
                    if let Err(e) = tokio::fs::write(&config.state_file_path, content).await {
                        return Err(StateError::PersistenceError(format!("写入状态文件失败: {}", e)));
                    }
                    log::debug!("状态已保存到文件: {:?}", config.state_file_path);
                }
                Err(e) => {
                    return Err(StateError::PersistenceError(format!("序列化状态失败: {}", e)));
                }
            }
        }
        Ok(())
    }
    
    /// 私有方法：启动自动保存任务
    async fn start_auto_save(&mut self) -> StateResult<()> {
        if let Some(config) = &self.persistence_config {
            if config.auto_save_interval > 0 {
                let state_clone = Arc::clone(&self.state);
                let config_clone = config.clone();
                let interval = tokio::time::Duration::from_secs(config.auto_save_interval);
                
                let handle = tokio::spawn(async move {
                    let mut interval_timer = tokio::time::interval(interval);
                    
                    loop {
                        interval_timer.tick().await;
                        
                        // 创建临时StateManager实例来调用save_state
                        let temp_manager = StateManager {
                            state: state_clone.clone(),
                            observers: Vec::new(),
                            event_sender: broadcast::channel(1).0,
                            persistence_config: Some(config_clone.clone()),
                            auto_save_handle: None,
                        };
                        
                        if let Err(e) = temp_manager.save_state().await {
                            log::error!("自动保存状态失败: {}", e);
                        } else {
                            log::debug!("自动保存状态成功");
                        }
                    }
                });
                
                self.auto_save_handle = Some(handle);
                log::info!("自动保存任务已启动，间隔: {}秒", config.auto_save_interval);
            }
        }
        Ok(())
    }
}
impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for StatePersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            state_file_path: PathBuf::from("state.json"),
            auto_save_interval: 60, // 默认60秒自动保存一次
        }
    }
}

impl StatePersistenceConfig {
    /// 创建新的持久化配置
    pub fn new(state_file_path: PathBuf) -> Self {
        Self {
            enabled: true,
            state_file_path,
            auto_save_interval: 60,
        }
    }
    
    /// 设置自动保存间隔
    pub fn with_auto_save_interval(mut self, interval_seconds: u64) -> Self {
        self.auto_save_interval = interval_seconds;
        self
    }
    
    /// 禁用自动保存
    pub fn without_auto_save(mut self) -> Self {
        self.auto_save_interval = 0;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;

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
    async fn test_state_transitions() {
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
    async fn test_event_broadcasting() {
        let mut manager = StateManager::new();
        let mut event_receiver = manager.subscribe_events();
        
        // 在后台任务中监听事件
        let event_count = Arc::new(AtomicUsize::new(0));
        let event_count_clone = Arc::clone(&event_count);
        
        tokio::spawn(async move {
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
        
        // 应该收到5个事件：1个游戏状态变更 + 4个程序状态变更
        assert_eq!(event_count.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_state_persistence() {
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
}