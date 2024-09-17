use anyhow::Result;
use api::start_server;

#[tokio::main]
async fn main() -> Result<()>{
    start_server().await
}
