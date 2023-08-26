use pixiv_lolicon_spider::utils::generate_bin_from_json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    picture_core::init_log();

    // generate_bin_from_json::run(
    //     "D:/Desktop/picture/save-spider-json-bin/done/pixiv_merge_2023-08-18_20-07-55.json",
    // )
    // .await?;
    generate_bin_from_json::run(
        r"D:\Desktop\picture\save-spider-json-bin\done\pixiv_valid_2023-08-26_20-46-09.json",
    )
    .await?;
    Ok(())
}
