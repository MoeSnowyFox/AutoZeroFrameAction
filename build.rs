fn main() {
    // 配置 vcpkg OpenCV
    #[cfg(target_os = "windows")]
    {
        // 使用已知的 vcpkg 路径
        let vcpkg_root = std::env::var("VCPKG_ROOT")
            .unwrap_or_else(|_| "F:/aaaa/github/MaaFramework/vcpkg".to_string());
        
        println!("cargo:rustc-link-search=native={}/installed/x64-windows/lib", vcpkg_root);
        println!("cargo:rustc-link-search=native={}/installed/x64-windows/bin", vcpkg_root);
        
        // 设置 OpenCV 环境变量来覆盖自动检测
        println!("cargo:rustc-link-lib=opencv_core4");
        println!("cargo:rustc-link-lib=opencv_imgcodecs4");
        println!("cargo:rustc-link-lib=opencv_imgproc4");
        
        // 设置 OpenCV 路径
        println!("cargo:rustc-env=OPENCV_LINK_PATHS={}/installed/x64-windows/lib", vcpkg_root);
    }
    
    // 编译 Slint UI
    slint_build::compile("ui/main.slint").unwrap();
}