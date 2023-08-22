use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::Regex;
use tokio::fs;
use tracing::{info, instrument, warn};

use picture_core::pixiv::PixivFile;

pub async fn run(
    json_filename: impl AsRef<Path>,
    picture_directory: impl AsRef<Path>,
) -> Result<()> {
    let bin_filename = json_filename.as_ref();
    let picture_directory = picture_directory.as_ref();

    let json_string = fs::read_to_string(bin_filename).await?;
    let json: PixivFile = serde_json::from_str(&json_string)?;

    let pixiv = json.data;
    let mut repetitive = Vec::new();
    let mut already_delete_in_json = Vec::new();
    let mut original_name = Vec::new();

    let mut dir = fs::read_dir(picture_directory).await?;

    let mut change_count = 0;
    //遍历图片文件
    while let Some(p) = dir.next_entry().await? {
        let mut path_buf = p.path();
        if path_buf.is_file() {
            info!("filename: {}", path_buf.display());

            if let Some(_) = match_new_filename(&path_buf)? {
                info!("{} is ok", path_buf.display());
                continue;
            }

            let filename = path_buf.file_stem().unwrap().to_str().unwrap();
            //println!("*filename: {}", filename);
            let pid_index = if let Some(pid_index) = filename.rfind('_') {
                pid_index
            } else {
                warn!("original name: {}", filename);
                original_name.push(path_buf);
                continue;
            };
            let p_index = if let Some(p_index) = filename.rfind('-') {
                p_index
            } else {
                warn!("original name: {}", filename);
                original_name.push(path_buf);
                continue;
            };
            //println!("filename: {filename} pid_index: {pid_index} p_index: {p_index} ");
            let pid = filename[pid_index + 1..p_index].parse::<u64>()?;
            let p = filename[p_index + 1..].parse::<u64>()?;
            //println!("pid: {} p: {}", pid, p);

            //去json文件里比对
            let v: Vec<_> = pixiv.iter().filter(|x| x.pid == pid && x.p == p).collect();
            //图片在，json文件里没了
            if v.len() == 0 {
                already_delete_in_json.push(path_buf);
                continue;
            }
            //图片在。json文件里也在，但是json文件中有两个相同的PixivData
            else if v.len() != 1 {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0].uid, v[1].uid);
                repetitive.push(v);
                continue;
            }
            //图片在，json文件里也在，且都只有一个
            let x = v[0];

            //( _｀ω´)ゞ@1234-82355973_1.jpg
            let new_filename = format!(
                "{}@{}-{}_{}.{}",
                allowed_file_name(x.title.as_str()),
                x.uid,
                x.pid,
                x.p,
                x.ext
            );

            let old_filename = path_buf.as_path().to_path_buf();
            info!("old_filename: {}", old_filename.display());

            path_buf.set_file_name(new_filename);
            let new_filename = path_buf.as_path();
            info!("new_filename: {}\n", new_filename.display());

            fs::rename(old_filename, new_filename).await?;

            change_count += 1;
        }
    }

    info!("change_count: {}", change_count);

    info!("repetitive: {}", repetitive.len());
    repetitive.iter().enumerate().for_each(|x| {
        println!("count: {}", x.0 + 1);
        x.1.iter().for_each(|y| {
            println!("data: {:?}\n\n", y);
        });
    });

    process_original_files(original_name, picture_directory.parent().unwrap()).await?;
    process_already_deleted_files(already_delete_in_json, picture_directory.parent().unwrap())
        .await?;

    Ok(())
}

#[instrument(skip_all)]
async fn process_original_files(
    original_name_files: Vec<PathBuf>,
    root_path: impl AsRef<Path>,
) -> Result<()> {
    info!("original_name_in_json: {}", original_name_files.len());
    if original_name_files.len() == 0 {
        return Ok(());
    }

    let mut original_name_directory = root_path.as_ref().join("original_name");

    if !original_name_directory.exists() {
        fs::create_dir_all(original_name_directory.as_path()).await?;
    }

    original_name_directory.push("test.jpg");

    let original_files: Vec<_> = original_name_files.iter().enumerate().collect();

    for (i, f) in original_files.iter() {
        info!("original_name count: {} filename: {}", i + 1, f.display());

        original_name_directory.set_file_name(f.file_name().unwrap());

        info!("move to:{}", original_name_directory.as_path().display());

        fs::rename(f.as_path(), original_name_directory.as_path()).await?;
    }
    Ok(())
}

#[instrument(skip_all)]
async fn process_already_deleted_files(
    deleted_files: Vec<PathBuf>,
    root_path: impl AsRef<Path>,
) -> Result<()> {
    info!("already_delete_in_json: {}", deleted_files.len());
    if deleted_files.len() == 0 {
        return Ok(());
    }

    let mut already_delete_directory = root_path.as_ref().join("already_delete");

    if !already_delete_directory.exists() {
        fs::create_dir_all(already_delete_directory.as_path()).await?;
    }

    already_delete_directory.push("test.jpg");

    let already_delete_files: Vec<_> = deleted_files.iter().enumerate().collect();

    for (i, f) in already_delete_files.iter() {
        info!("already_delete count: {} filename: {}", i + 1, f.display());

        already_delete_directory.set_file_name(f.file_name().unwrap());

        info!("move to:{}", already_delete_directory.as_path().display());

        fs::rename(f.as_path(), already_delete_directory.as_path()).await?;
    }
    Ok(())
}

fn allowed_file_name(title: &str) -> String {
    title.replace(
        [
            '#', '\'', '/', '\\', ':', '*', '?', '\"', '>', '<', '|', '&',
        ],
        "_",
    )
}

fn match_new_filename(
    filename: impl AsRef<Path>,
) -> Result<Option<(String, u64, u64, u64, String)>> {
    let pattern = r"(\d+)-(\d+)_(\d+)\.(\w+)";
    let text = filename.as_ref().to_str().unwrap();

    let title = if let Some(i) = text.rfind('@') {
        let title = &text[0..i];
        title
    } else {
        warn!("not changed name: {}", text);
        return Ok(None);
    };

    let text = text.replace(&(title.to_owned() + "@"), "");

    let re = Regex::new(pattern)?;
    if let Some(captures) = re.captures(&text) {
        //let title = captures.get(1).unwrap().as_str();
        let uid = captures.get(1).unwrap().as_str();
        let pid = captures.get(2).unwrap().as_str();
        let p = captures.get(3).unwrap().as_str();
        let ext = captures.get(4).unwrap().as_str();

        Ok(Some((
            title.to_owned(),
            uid.parse::<u64>()?,
            pid.parse::<u64>()?,
            p.parse::<u64>()?,
            ext.to_owned(),
        )))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use crate::match_new_filename;

    #[test]
    fn t() {
        let filename = match_new_filename("水も滴るいい狐@6247414-93245877_0.jpg").unwrap();
        match filename {
            None => {
                println!("None")
            }
            Some((title, uid, pid, p, ext)) => {
                println!("title: {}", title);
                println!("uid: {}", uid);
                println!("pid: {}", pid);
                println!("p: {}", p);
                println!("ext: {}", ext);
            }
        }
    }
}
