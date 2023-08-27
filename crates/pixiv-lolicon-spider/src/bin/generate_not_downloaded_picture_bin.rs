use anyhow::Result;
use picture_core::pixiv::{PixivData, PixivFile};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    picture_core::init_log();
    if cfg!(windows) {
        run(
            r"D:\Desktop\picture\save-spider-json-bin\done\pixiv_2023-08-22_15-16-36.bin",
            r"D:\Desktop\picture\save-spider-json-bin\done",
            r"D:\Desktop\picture\crates\aml-picture\pixiv\picture",
        )
        .await?;
    } else {
        run(
            r"/mnt/usb/disk1/picture/pixiv/pixiv.bin",
            r"/mnt/usb/disk1/picture/pixiv/",
            r"/mnt/usb/disk1/picture/pixiv/picture",
        )
        .await?;
    }
    //86550-3787=82763
    Ok(())
}

pub async fn run<P: AsRef<Path>>(
    from_bin_filepath: P,
    directory_generated_bin: P,
    picture_directory: P,
) -> Result<Option<PathBuf>> {
    let from_bin_filepath = from_bin_filepath.as_ref();
    if from_bin_filepath.exists() {
        info!("bin filepath: {}", from_bin_filepath.display());
        let c = fs::read(from_bin_filepath).await?;
        let pixiv_json: PixivFile = bincode::deserialize_from(c.as_slice())?;

        info!("original count: {}", pixiv_json.len);

        let p = pixiv_json.data;

        let mut filepath = PathBuf::from(picture_directory.as_ref());
        filepath.push("test.jpg");

        let new_p: Vec<_> = p
            .into_iter()
            .filter(|x| {
                let filename = format!(
                    "{}@{}-{}_{}.{}",
                    allowed_file_name(x.title.as_str()),
                    x.uid,
                    x.pid,
                    x.p,
                    x.ext
                );
                filepath.set_file_name(filename);
                if filepath.exists() {
                    false
                } else {
                    true
                }
            })
            .filter(|p| p.valid)
            .collect();

        info!("from: {:?}", from_bin_filepath.file_name().unwrap());
        let save_name = save_bin_file(new_p, directory_generated_bin).await?;
        return Ok(Some(save_name));
    } else {
        error!("文件不存在: {}", from_bin_filepath.display());
    }
    Ok(None)
}

async fn save_bin_file(
    pixiv: Vec<PixivData>,
    directory_generated_bin: impl AsRef<Path>,
) -> Result<PathBuf> {
    info!("now count: {}", pixiv.len());

    let now = chrono::Local::now();

    let len = pixiv.len() as u64;
    let valid_len = pixiv.iter().filter(|p| p.valid).count() as u64;
    assert_eq!(len, valid_len);
    let save = PixivFile {
        len,
        valid_len,
        update_time: now.timestamp(),
        data: pixiv,
    };

    let directory_generated_bin = directory_generated_bin.as_ref();

    if !directory_generated_bin.exists() {
        fs::create_dir_all(directory_generated_bin).await?;
    }

    let save_name = directory_generated_bin.join(format!(
        "pixiv_not_download_{}.bin",
        now.format("%Y-%m-%d_%H-%M-%S")
    ));
    let bin_code = bincode::serialize(&save)?;
    info!("save to: {:?}", save_name.as_path());
    fs::write(save_name.as_path(), bin_code).await?;
    Ok(save_name)
}

fn allowed_file_name(title: &str) -> String {
    title.replace(
        [
            '#', '\'', '/', '\\', ':', '*', '?', '\"', '>', '<', '|', '&',
        ],
        "_",
    )
}
