use anyhow::Result;
use picture_core::pixiv::{PixivData, PixivFile};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tokio::fs;
use tracing::{debug, info, warn};

pub async fn run(
    root_path: impl AsRef<Path>,
    filename: impl AsRef<Path>,
    bad_picture_dir: impl AsRef<Path>,
) -> Result<Option<PathBuf>> {
    let mut change_to_false_count = 0;
    let mut content = String::new();

    let mut dir = fs::read_dir(bad_picture_dir.as_ref()).await?;
    while let Some(p) = dir.next_entry().await? {
        let filename = p.path();
        if filename.is_file()
            && filename
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("bad_picture")
            && filename.extension().unwrap().eq("txt")
        {
            debug!("filename: {}", filename.display());
            let s = fs::read_to_string(filename.as_path()).await?;
            content.push_str(s.as_str());
        }
    }

    let mut bad_pixiv: Vec<_> = Vec::new();
    content
        .split("\n")
        .filter(|p| *p != "")
        .map(|p| get_bad_pixiv_data(p).unwrap())
        .for_each(|p| {
            if !bad_pixiv.contains(&p) {
                bad_pixiv.push(p);
            }
        });

    if bad_pixiv.len() == 0 {
        warn!("bad picture count is 0,so not generate file");
        return Ok(None);
    }

    info!("bad picture count: {}", bad_pixiv.len());

    let pixiv_file: PixivFile = serde_json::from_str(
        fs::read_to_string(root_path.as_ref().join("done").join(filename.as_ref()))
            .await?
            .as_str(),
    )?;

    let pixiv = pixiv_file.data;

    let all_pixiv_count = pixiv.len();

    info!("pixiv picture count: {}", pixiv.len());

    let x: Vec<_> = pixiv
        .into_iter()
        .map(|mut p| {
            if bad_pixiv.contains(&p) {
                if p.valid {
                    change_to_false_count += 1;
                    //设置为无效
                    p.valid = false;
                }
            }
            p
        })
        .collect();

    if change_to_false_count == 0 {
        warn!("change_to_false_count is 0,so not generate file");
        return Ok(None);
    }

    info!("valid picture count: {}",all_pixiv_count - bad_pixiv.len());
    let generate_file = save_json_file(x, root_path.as_ref()).await?;

    Ok(Some(generate_file))
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
        "pixiv_valid_{}.json",
        now.format("%Y-%m-%d_%H-%M-%S")
    ));
    info!("save path: {:?}", path.as_path());
    fs::write(path.as_path(), string).await?;
    Ok(path)
}

fn get_bad_pixiv_data(s: &str) -> Result<PixivData> {
    Ok(s.parse::<MyPixivData>()?.0)
}

struct MyPixivData(PixivData);

impl FromStr for MyPixivData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let find_title = "title: ";
        let title_next_index = s.find(find_title).unwrap() + find_title.len();

        let find_uid = " uid: ";
        let uid_index = s.find(find_uid).unwrap();

        let find_pid = " pid: ";
        let pid_index = s.find(find_pid).unwrap();

        let find_p = " p: ";
        let p_index = s.find(find_p).unwrap();

        let find_bytes = " bytes_len: ";
        let bytes_index = s.find(find_bytes).unwrap();

        let title = &s[title_next_index..uid_index];
        let uid: u64 = s[uid_index + find_uid.len()..pid_index].parse()?;
        let pid: u64 = s[pid_index + find_pid.len()..p_index].parse()?;
        let p: u64 = s[p_index + find_p.len()..bytes_index].parse()?;
        let bytes: u64 = s[bytes_index + find_bytes.len()..s.len() - 6].parse()?;
        if bytes != 54 && bytes != 155 {
            panic!("s: {}", s)
        }

        Ok(MyPixivData(PixivData {
            title: title.to_owned(),
            uid,
            pid,
            p,
            ..Default::default()
        }))
    }
}
