#![feature(async_closure)]

use std::path::{Path, PathBuf};

use tokio::fs;

use picture_core::pixiv::PixivFile;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    run("pixiv/pixiv.bin").await?;
    Ok(())
}

async fn run(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let s = tokio::fs::read(path.as_ref()).await.expect("读取文件失败");
    let v = bincode::deserialize::<PixivFile>(&s[..]).unwrap();

    for x in &v.data {
        let title = allowed_file_name(x.title.as_str());
        let filename = PathBuf::from(format!("crates/aml-picture/pixiv/picture/{}.jpg", title));
        if filename.exists() {
            println!("exists: {:?}", filename);
            let mut new_filename = filename.to_path_buf();
            new_filename.set_file_name(format!("{}_{}-{}.jpg", title, x.pid, x.p));
            println!("rename: {:?}", new_filename);
            fs::rename(filename, &new_filename).await?;
            println!("rename completed: {:?}", new_filename);
        }
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
