use p4_controller::cli::{Cli, CliHandler};
use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // ログ設定を初期化
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
    
    info!("Starting P4 Controller...");
    
    // CLIを解析
    let cli = Cli::parse();
    
    // CLIハンドラーを作成して実行
    let handler = CliHandler::new();
    handler.run(cli).await?;
    
    info!("P4 Controller finished");
    Ok(())
}
