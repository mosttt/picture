use anyhow::Result;
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

    info!("valid picture count: {}", all_pixiv_count - bad_pixiv.len());
    let generate_file = save_json_file(x, root_path.as_ref()).await?;

    Ok(Some(generate_file))
}

async fn save_json_file(pixiv: Vec<PixivData>, root_path: impl AsRef<Path>) -> Result<PathBuf> {
    let now = chrono::Local::now();

    let save = PixivFile {
        len: pixiv.len() as u64,
        valid_len: pixiv.iter().filter(|p| p.valid).count() as u64,
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
        //title: あけましてさかまたございます uid: 26644 pid: 95196993 p: 0 upload_date: 1111111 bytes_len: 54 bytes
        let find_title = "title: ";
        let title_next_index = s.find(find_title).unwrap() + find_title.len();

        let find_uid = " uid: ";
        let uid_index = s.find(find_uid).unwrap();

        let find_pid = " pid: ";
        let pid_index = s.find(find_pid).unwrap();

        let find_p = " p: ";
        let p_index = s.find(find_p).unwrap();

        let find_upload_date = " upload_date: ";
        let upload_date_index = s.find(find_upload_date).unwrap();

        let find_bytes = " bytes_len: ";
        let bytes_index = s.find(find_bytes).unwrap();

        let title = &s[title_next_index..uid_index];
        let uid: u64 = s[uid_index + find_uid.len()..pid_index].parse()?;
        let pid: u64 = s[pid_index + find_pid.len()..p_index].parse()?;
        let p: u64 = s[p_index + find_p.len()..upload_date_index].parse()?;
        let upload_date: i64 =
            s[upload_date_index + find_upload_date.len()..bytes_index].parse()?;
        let bytes: u64 = s[bytes_index + find_bytes.len()..s.len() - 6].parse()?;
        if bytes != 54 && bytes != 155 {
            panic!("s: {}", s)
        }

        Ok(MyPixivData(PixivData {
            title: title.to_owned(),
            uid,
            pid,
            p,
            upload_date,
            ..Default::default()
        }))
    }
}

use picture_core::pixiv::Urls;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PixivFile {
    pub len: u64,
    ///爬下来的数据中没有valid字段，所以默认为0_u64
    #[serde(default = "default_len")]
    pub valid_len: u64,
    pub update_time: i64,
    pub data: Vec<PixivData>,
}

fn default_len() -> u64 {
    0
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PixivData {
    ///爬下来的数据中没有valid字段，所以默认为true
    #[serde(default = "default_true")]
    pub valid: bool,
    pub pid: u64,
    pub p: u64,
    pub uid: u64,
    pub title: String,
    pub author: String,
    pub r18: bool,
    pub width: u64,
    pub height: u64,
    pub tags: Vec<String>,
    pub ext: String,
    pub ai_type: i64,
    pub upload_date: i64,
    pub urls: Urls,
}

fn default_true() -> bool {
    true
}

impl PartialEq for PixivData {
    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid
            && self.p == other.p
            && self.uid == other.uid
            && self.upload_date == other.upload_date
    }
}
