//! 明日方舟智能鼠标宏
//!
//! 基于 Rust + Slint + OpenCV 技术栈的桌面应用程序
//! 支持宏模式和智能模式两种工作方式

mod models;
mod services;
mod ui;
mod utils;

use ui::MainAppBuilder;
use utils::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    // 初始化日志系统
    env_logger::init();

    log::info!("正在启动明日方舟智能鼠标宏应用程序...");

    // 创建并运行应用程序
    let mut app = MainAppBuilder::new()
        .with_logging(false) // 已经初始化过了
        .build()?;

    log::info!("应用程序初始化完成，启动主循环");

    // 运行应用程序
    app.run().await?;

    log::info!("应用程序正常退出");
    Ok(())
}
