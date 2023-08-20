use std::path::Path;

use anyhow::Result;
use async_recursion::async_recursion;
use tokio::fs;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<()> {
    run("pixiv/picture").await?;
    Ok(())
}

#[async_recursion]
async fn run<'a>(path: impl AsRef<Path> + Send + 'static) -> Result<()> {
    let path = path.as_ref();
    let mut dir = fs::read_dir(path).await?;
    while let Some(res) = dir.next_entry().await? {
        let path = res.path();
        if path.is_file() {
            let file = File::open(path.as_path()).await?;
            let len = file.metadata().await.unwrap().len();
            if len < 1024 {
                fs::remove_file(path.as_path()).await?;
                println!("delete: {:?}", path);
            }
        } else {
            run(path).await?;
        }
    }
    Ok(())
}
