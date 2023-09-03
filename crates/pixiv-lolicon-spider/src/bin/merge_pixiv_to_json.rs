use anyhow::Result;
use tokio::time::Instant;
use tracing::info;

use pixiv_lolicon_spider::utils::merge_pixiv_to_json;

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Instant::now();

    picture_core::init_log();

    merge_pixiv_to_json::run(
        r"D:\Desktop\picture\save-spider-json-bin",
        Some("pixiv_valid_2023-08-26_20-46-09.json"),
    )
    .await?;
    let end = start_time.elapsed().as_secs();
    info!("耗时: {}秒", end);

    Ok(())
}
