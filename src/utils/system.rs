//! 系统相关工具函数

use std::path::PathBuf;

/// 获取应用程序配置目录
pub fn get_app_config_dir() -> Result<PathBuf, std::io::Error> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "无法获取系统配置目录"
        ))?;
    
    let app_dir = config_dir.join("arknights-macro");
    
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }
    
    Ok(app_dir)
}

/// 获取应用程序数据目录
pub fn get_app_data_dir() -> Result<PathBuf, std::io::Error> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "无法获取系统数据目录"
        ))?;
    
    let app_dir = data_dir.join("arknights-macro");
    
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }
    
    Ok(app_dir)
}

/// 获取应用程序缓存目录
pub fn get_app_cache_dir() -> Result<PathBuf, std::io::Error> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "无法获取系统缓存目录"
        ))?;
    
    let app_dir = cache_dir.join("arknights-macro");
    
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir)?;
    }
    
    Ok(app_dir)
}

/// 检查是否为管理员权限（Windows）
#[cfg(windows)]
pub fn is_elevated() -> bool {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size = 0;
        
        let result = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        );
        
        CloseHandle(token);
        
        result != 0 && elevation.TokenIsElevated != 0
    }
}

/// 检查是否为管理员权限（非Windows）
#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}