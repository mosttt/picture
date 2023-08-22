use std::path::{Path, PathBuf};

use anyhow::Result;
use tokio::fs;
use tracing::{info, warn};

use picture_core::pixiv::{PixivData, PixivFile};

pub async fn run(
    root_path: impl AsRef<Path>,
    filename: impl AsRef<Path>,
) -> Result<Option<PathBuf>> {
    let root_path = root_path.as_ref();
    let filename = filename.as_ref();
    let json_filename = root_path.join("done").join(filename);

    let json_string = fs::read_to_string(json_filename.as_path()).await?;
    let json: PixivFile = serde_json::from_str(json_string.as_str())?;

    let pixiv = json.data;

    info!("pixiv count: {}", pixiv.len());

    let new_pixiv: Vec<_> = pixiv
        .iter()
        .filter(|x| {
            if is_delete_by_title_tag(x) {
                false
            } else {
                true
            }
        })
        .collect();

    let change_pixiv = pixiv.len() - new_pixiv.len();
    if change_pixiv == 0 {
        warn!("no needed deleted pixiv");
        return Ok(None);
    }

    info!("new pixiv count: {}", new_pixiv.len());

    let generate_file = save_json_file(new_pixiv.into_iter().cloned().collect(), root_path).await?;

    Ok(Some(generate_file))
}

fn is_delete_by_title_tag(pixiv_data: &PixivData) -> bool {
    pixiv_data.title.is_empty() || pixiv_data.tags.is_empty()
}

async fn save_json_file(pixiv: Vec<PixivData>, root_path: impl AsRef<Path>) -> Result<PathBuf> {
    let now = chrono::Local::now();
    let save = PixivFile {
        len: pixiv.len() as u64,
        update_time: now.timestamp(),
        data: pixiv,
    };
    let string = serde_json::to_string_pretty(&save)?;

    let mut path = root_path.as_ref().join("done");
    if !path.exists() {
        fs::create_dir_all(path.as_path()).await?;
    }
    path.push(format!(
        "pixiv_deleted_{}.json",
        now.format("%Y-%m-%d_%H-%M-%S")
    ));
    info!("save path: {:?}", path.as_path());
    fs::write(path.as_path(), string).await?;
    Ok(path)
}

#[cfg(test)]
mod test {
    #[test]
    fn t() {}
}
