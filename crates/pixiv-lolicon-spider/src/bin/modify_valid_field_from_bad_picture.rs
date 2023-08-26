use anyhow::Result;

use pixiv_lolicon_spider::utils::modify_valid_field_from_bad_picture;

#[tokio::main]
async fn main() -> Result<()> {
    picture_core::init_log();
    modify_valid_field_from_bad_picture::run(
        r"D:\Desktop\picture\save-spider-json-bin",
        "pixiv_valid_2023-08-26_20-46-09.json",
        r"D:\Desktop\picture\save-spider-json-bin\bad_picture",
    )
    .await?;
    Ok(())
}
