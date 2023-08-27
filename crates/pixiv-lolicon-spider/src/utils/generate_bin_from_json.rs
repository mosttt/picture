use picture_core::pixiv::PixivFile;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{error, info};

pub async fn run(filepath: impl AsRef<Path>) -> anyhow::Result<Option<PathBuf>> {
    let filepath = filepath.as_ref();
    if filepath.exists() {
        info!("filepath: {:?}", filepath);
        let c = fs::read_to_string(filepath).await?;
        let p: PixivFile = serde_json::from_str(c.as_str())?;
        let bincode = bincode::serialize(&p)?;

        let now = chrono::Local::now();

        let save_name =
            filepath.with_file_name(format!("pixiv_{}.bin", now.format("%Y-%m-%d_%H-%M-%S")));
        info!("save name: {:?}", save_name.as_path());
        fs::write(save_name.as_path(), bincode).await?;
        return Ok(Some(save_name));
    } else {
        error!("文件不存在: {}", filepath.display());
    }
    Ok(None)
}
