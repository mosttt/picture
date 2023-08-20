use pixiv_lolicon_spider::entity::PixivJson;
use std::path::Path;
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filename = Path::new(
        "D:/Desktop/picture/save-spider-json-bin/done/pixiv_merge_2023-08-18_20-07-55.json",
    );
    if filename.exists() {
        println!("filename: {:?}", filename);
        let c = fs::read_to_string(filename).await?;
        let p: PixivJson = serde_json::from_str(c.as_str())?;
        let bincode = bincode::serialize(&p)?;

        let now = chrono::Local::now();

        let save_name =
            filename.with_file_name(format!("pixiv_{}.bin", now.format("%Y-%m-%d_%H-%M-%S")));
        println!("save name: {:?}", save_name.as_path());
        fs::write(save_name.as_path(), bincode).await?;
    }
    Ok(())
}
