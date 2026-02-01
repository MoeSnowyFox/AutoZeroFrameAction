//! 图像识别服务
//! 
//! 负责使用OpenCV进行游戏状态识别和图像分析

use crate::models::{UIElement, UIElementType};
use crate::utils::{VisionError, VisionResult};
use opencv::core::Mat;
use std::time::{Duration, Instant};
use std::sync::Arc;

/// 战斗状态检测配置
#[derive(Debug, Clone)]
pub struct BattleDetectionConfig {
    /// 检测区域 (x, y, width, height)
    pub detection_region: (i32, i32, u32, u32),
    /// 匹配阈值
    pub match_threshold: f32,
    /// 模板图片路径
    pub template_paths: Vec<String>,
}

impl Default for BattleDetectionConfig {
    fn default() -> Self {
        Self {
            detection_region: (0, 0, 1920, 1080),
            match_threshold: 0.8,
            template_paths: vec![
                "templates/battle_ui.png".to_string(),
                "templates/pause_button.png".to_string(),
            ],
        }
    }
}

/// UI元素检测配置
#[derive(Debug, Clone)]
pub struct UIDetectionConfig {
    /// 干员检测区域
    pub operator_region: (i32, i32, u32, u32),
    /// 技能检测区域
    pub skill_region: (i32, i32, u32, u32),
    /// 检测阈值
    pub detection_threshold: f32,
}

impl Default for UIDetectionConfig {
    fn default() -> Self {
        Self {
            operator_region: (100, 800, 800, 200),
            skill_region: (1000, 600, 300, 200),
            detection_threshold: 0.7,
        }
    }
}

/// 图像识别服务
pub struct VisionService {
    /// 上次捕获时间
    last_capture_time: Instant,
    /// 节流间隔
    throttle_interval: Duration,
    /// 战斗检测配置
    battle_config: BattleDetectionConfig,
    /// UI检测配置
    ui_config: UIDetectionConfig,
    /// 是否启用节流
    throttle_enabled: bool,
    /// 缓存的屏幕截图
    cached_screenshot: Option<Mat>,
    /// 缓存时间
    cache_time: Option<Instant>,
    /// 缓存有效期
    cache_duration: Duration,
}

impl VisionService {
    /// 创建新的图像识别服务
    pub fn new() -> Self {
        Self {
            last_capture_time: Instant::now() - Duration::from_secs(1),
            throttle_interval: Duration::from_millis(100),
            battle_config: BattleDetectionConfig::default(),
            ui_config: UIDetectionConfig::default(),
            throttle_enabled: true,
            cached_screenshot: None,
            cache_time: None,
            cache_duration: Duration::from_millis(50),
        }
    }
    
    /// 使用自定义配置创建服务
    pub fn with_config(
        battle_config: BattleDetectionConfig,
        ui_config: UIDetectionConfig,
    ) -> Self {
        Self {
            last_capture_time: Instant::now() - Duration::from_secs(1),
            throttle_interval: Duration::from_millis(100),
            battle_config,
            ui_config,
            throttle_enabled: true,
            cached_screenshot: None,
            cache_time: None,
            cache_duration: Duration::from_millis(50),
        }
    }
    
    /// 设置节流间隔
    pub fn set_throttle_interval(&mut self, interval: Duration) {
        self.throttle_interval = interval;
    }
    
    /// 启用或禁用节流
    pub fn set_throttle_enabled(&mut self, enabled: bool) {
        self.throttle_enabled = enabled;
    }
    
    /// 检查是否在战斗状态
    pub fn is_in_battle(&mut self) -> VisionResult<bool> {
        // 节流检查
        if self.throttle_enabled {
            let now = Instant::now();
            if now.duration_since(self.last_capture_time) < self.throttle_interval {
                return Ok(false); // 返回默认状态，避免过度检测
            }
            self.last_capture_time = now;
        }
        
        log::debug!("开始检测战斗状态");
        
        // 获取屏幕截图
        let screenshot = self.capture_game_screen_cached()?;
        
        // 检测战斗UI元素
        let battle_detected = self.detect_battle_ui(&screenshot)?;
        
        log::debug!("战斗状态检测结果: {}", battle_detected);
        Ok(battle_detected)
    }
    
    /// 捕获游戏屏幕
    pub fn capture_game_screen(&mut self) -> VisionResult<Mat> {
        log::debug!("捕获游戏屏幕");
        
        // 由于OpenCV链接问题，这里提供一个占位符实现
        // 在实际部署时，这里应该调用WindowService的截图功能
        self.create_placeholder_image()
    }
    
    /// 捕获游戏屏幕（带缓存）
    pub fn capture_game_screen_cached(&mut self) -> VisionResult<Mat> {
        let now = Instant::now();
        
        // 检查缓存是否有效
        if let (Some(cached), Some(cache_time)) = (&self.cached_screenshot, self.cache_time) {
            if now.duration_since(cache_time) < self.cache_duration {
                log::debug!("使用缓存的屏幕截图");
                return Ok(cached.clone());
            }
        }
        
        // 捕获新的截图
        let screenshot = self.capture_game_screen()?;
        
        // 更新缓存
        self.cached_screenshot = Some(screenshot.clone());
        self.cache_time = Some(now);
        
        Ok(screenshot)
    }
    
    /// 检测UI元素
    pub fn detect_ui_elements(&mut self) -> VisionResult<Vec<UIElement>> {
        log::debug!("开始检测UI元素");
        
        let screenshot = self.capture_game_screen_cached()?;
        let mut elements = Vec::new();
        
        // 检测干员
        let operators = self.detect_operators(&screenshot)?;
        elements.extend(operators);
        
        // 检测技能
        let skills = self.detect_skills(&screenshot)?;
        elements.extend(skills);
        
        log::debug!("检测到 {} 个UI元素", elements.len());
        Ok(elements)
    }
    
    /// 检测战斗UI
    fn detect_battle_ui(&self, screenshot: &Mat) -> VisionResult<bool> {
        // 由于OpenCV链接问题，这里提供一个占位符实现
        // 在实际部署时，这里应该使用模板匹配来检测战斗UI
        
        log::debug!("检测战斗UI（占位符实现）");
        
        // 模拟检测逻辑：随机返回结果用于测试
        // 在实际实现中，这里会：
        // 1. 在指定区域内搜索战斗UI模板
        // 2. 使用模板匹配算法
        // 3. 根据匹配度判断是否在战斗中
        
        Ok(false) // 默认返回非战斗状态
    }
    
    /// 检测干员
    fn detect_operators(&self, screenshot: &Mat) -> VisionResult<Vec<UIElement>> {
        log::debug!("检测干员UI元素");
        
        let mut operators = Vec::new();
        
        // 占位符实现：在干员区域内模拟检测到一些干员
        let (x, y, width, height) = self.ui_config.operator_region;
        
        // 模拟检测到3个干员
        for i in 0..3 {
            let operator_x = x + (i * (width as i32 / 3));
            let operator_y = y + height as i32 / 2;
            
            operators.push(UIElement::new(
                UIElementType::Operator,
                (operator_x, operator_y),
                (80, 80),
            ));
        }
        
        log::debug!("检测到 {} 个干员", operators.len());
        Ok(operators)
    }
    
    /// 检测技能
    fn detect_skills(&self, screenshot: &Mat) -> VisionResult<Vec<UIElement>> {
        log::debug!("检测技能UI元素");
        
        let mut skills = Vec::new();
        
        // 占位符实现：在技能区域内模拟检测到一些技能
        let (x, y, width, height) = self.ui_config.skill_region;
        
        // 模拟检测到2个技能
        for i in 0..2 {
            let skill_x = x + (i * (width as i32 / 2));
            let skill_y = y + height as i32 / 2;
            
            skills.push(UIElement::new(
                UIElementType::Skill,
                (skill_x, skill_y),
                (60, 60),
            ));
        }
        
        log::debug!("检测到 {} 个技能", skills.len());
        Ok(skills)
    }
    
    /// 创建占位符图像（用于测试）
    fn create_placeholder_image(&self) -> VisionResult<Mat> {
        // 由于OpenCV链接问题，这里创建一个空的Mat作为占位符
        // 在实际部署时，这里应该返回真实的屏幕截图
        
        #[cfg(feature = "opencv-working")]
        {
            use opencv::core::{Size, CV_8UC3};
            use opencv::imgproc;
            
            let mut img = Mat::zeros(1080, 1920, CV_8UC3)?.to_mat()?;
            
            // 添加一些测试内容
            imgproc::rectangle(
                &mut img,
                opencv::core::Rect::new(100, 100, 200, 100),
                opencv::core::Scalar::new(0.0, 255.0, 0.0, 0.0),
                2,
                imgproc::LINE_8,
                0,
            )?;
            
            Ok(img)
        }
        
        #[cfg(not(feature = "opencv-working"))]
        {
            // 占位符实现：创建一个空的Mat
            log::warn!("OpenCV功能不可用，使用占位符图像");
            
            // 由于OpenCV链接问题，我们无法创建真实的Mat
            // 这里返回一个错误，表示功能暂时不可用
            Err(VisionError::CaptureError(
                "OpenCV链接问题，屏幕捕获功能暂时不可用".to_string()
            ))
        }
    }
    
    /// 清除缓存
    pub fn clear_cache(&mut self) {
        self.cached_screenshot = None;
        self.cache_time = None;
        log::debug!("已清除图像缓存");
    }
    
    /// 获取战斗检测配置
    pub fn get_battle_config(&self) -> &BattleDetectionConfig {
        &self.battle_config
    }
    
    /// 设置战斗检测配置
    pub fn set_battle_config(&mut self, config: BattleDetectionConfig) {
        self.battle_config = config;
        log::debug!("已更新战斗检测配置");
    }
    
    /// 获取UI检测配置
    pub fn get_ui_config(&self) -> &UIDetectionConfig {
        &self.ui_config
    }
    
    /// 设置UI检测配置
    pub fn set_ui_config(&mut self, config: UIDetectionConfig) {
        self.ui_config = config;
        log::debug!("已更新UI检测配置");
    }
    
    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> (bool, Option<Duration>) {
        let has_cache = self.cached_screenshot.is_some();
        let cache_age = self.cache_time.map(|t| Instant::now().duration_since(t));
        (has_cache, cache_age)
    }
    
    /// 设置缓存有效期
    pub fn set_cache_duration(&mut self, duration: Duration) {
        self.cache_duration = duration;
        log::debug!("已设置缓存有效期为 {:?}", duration);
    }
}

impl Default for VisionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_service_creation() {
        let service = VisionService::new();
        assert!(service.throttle_enabled);
        assert_eq!(service.throttle_interval, Duration::from_millis(100));
    }

    #[test]
    fn test_throttle_configuration() {
        let mut service = VisionService::new();
        
        service.set_throttle_interval(Duration::from_millis(200));
        assert_eq!(service.throttle_interval, Duration::from_millis(200));
        
        service.set_throttle_enabled(false);
        assert!(!service.throttle_enabled);
    }

    #[test]
    fn test_config_updates() {
        let mut service = VisionService::new();
        
        let battle_config = BattleDetectionConfig {
            detection_region: (0, 0, 800, 600),
            match_threshold: 0.9,
            template_paths: vec!["test.png".to_string()],
        };
        
        service.set_battle_config(battle_config.clone());
        assert_eq!(service.get_battle_config().match_threshold, 0.9);
    }

    #[test]
    fn test_cache_management() {
        let mut service = VisionService::new();
        
        let (has_cache, _) = service.get_cache_stats();
        assert!(!has_cache);
        
        service.set_cache_duration(Duration::from_millis(100));
        assert_eq!(service.cache_duration, Duration::from_millis(100));
        
        service.clear_cache();
        let (has_cache_after_clear, _) = service.get_cache_stats();
        assert!(!has_cache_after_clear);
    }
}