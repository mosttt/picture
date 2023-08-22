use anyhow::Result;
use tokio::time::Instant;
use tracing::info;

use pixiv_lolicon_spider::utils::delete_empty_json;

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Instant::now();

    picture_core::init_log();

    delete_empty_json::run(
        r"D:\Desktop\picture\save-spider-json-bin",
        "pixiv_merge_2023-08-18_20-07-55.json",
    )
    .await?;

    let end = start_time.elapsed().as_secs();
    info!("耗时: {}秒", end);
    Ok(())
}
