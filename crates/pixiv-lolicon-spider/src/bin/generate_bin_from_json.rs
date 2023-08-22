use pixiv_lolicon_spider::utils::generate_bin_from_json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    picture_core::init_log();

    generate_bin_from_json::run(
        "D:/Desktop/picture/save-spider-json-bin/done/pixiv_merge_2023-08-18_20-07-55.json",
    )
    .await?;
    Ok(())
}
