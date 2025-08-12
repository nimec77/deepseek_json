use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    deepseek_json::cli::run_cli().await
}
