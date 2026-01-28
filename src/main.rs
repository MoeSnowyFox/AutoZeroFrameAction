mod config;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // 创建窗口实例
    let ui = AppWindow::new()?;
    
    // 设置按钮点击的回调函数
    ui.on_button_clicked({
        move || {
            println!("按钮被点击了！");
        }
    });
    
    // 显示窗口
    ui.show()?;
    
    // 运行事件循环 - 这会让窗口保持打开状态并响应用户交互
    slint::run_event_loop()
}