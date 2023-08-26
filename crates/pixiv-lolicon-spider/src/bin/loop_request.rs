use anyhow::Result;
use reqwest::StatusCode;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    picture_core::init_log();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    loop {
        match client
            .get("http://127.0.0.1:5800/api/pixiv/v1")
            .send()
            .await
        {
            Ok(v) => {
                if v.status() != StatusCode::OK {
                    warn!("status: {:?}  text: {:?}", v.status(), v);
                } else {
                    info!("status: {:?}", v.status());
                }
            }
            Err(e) => {
                error!("e: {:?}", e);
            }
        };
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
