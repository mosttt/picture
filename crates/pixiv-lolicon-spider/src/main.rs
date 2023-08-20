use std::env;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Local, Timelike};
use lazy_static::lazy_static;
use tokio::fs;
use tokio::task::JoinHandle;
use tokio::time::interval;

use pixiv_lolicon_spider::entity::{Pixiv, PixivJson};

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    env::set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();
    let spider: JoinHandle<Result<()>> = tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(60)); //一分钟
        loop {
            interval.tick().await;
            println!("start save");
            save_to_json("p").await?;
            println!("end save\n");
        }
        //Ok::<(),Error>(())
    });
    let merging_into_json: JoinHandle<Result<()>> = tokio::spawn(async {
        let mut interval = interval(Duration::from_secs(6 * 60 * 60)); //六个小时
        loop {
            interval.tick().await;
            println!("start merge");
            process_to_json_and_bin("processing", "completed").await?;
            println!("end merge\n");
        }
        //Ok::<(),Error>(())
    });

    spider.await??;
    merging_into_json.await??;
    Ok(())
}

async fn process_to_json_and_bin(
    processing: impl AsRef<Path>,
    completed: impl AsRef<Path>,
) -> Result<()> {
    //////////////////////////////////////////////////////////////////////////////////////////////////
    async fn move_directory_file_to_other_directory(
        from: impl AsRef<Path>,
        to: impl AsRef<Path>,
    ) -> Result<()> {
        println!("move_to_processing_directory");

        let to = to.as_ref();
        if !to.exists() {
            fs::create_dir_all(to).await?;
        } else if to.is_file() {
            return Err(anyhow!("{} is file", to.display()));
        }

        let mut dir = fs::read_dir(from).await?;
        while let Some(f) = dir.next_entry().await? {
            let path = f.path();
            if path.is_file() {
                fs::rename(path.as_path(), to.join(path.file_name().unwrap())).await?;
            }
        }
        Ok(())
    }

    //////////////////////////////////////////////////////////////////////////////////////////////////
    async fn process_file_to_json_and_bin(from: impl AsRef<Path>) -> Result<()> {
        println!("process_file_to_json_and_bin");

        let mut data = Vec::with_capacity(1000);
        let mut dir = fs::read_dir(from).await?;
        while let Some(f) = dir.next_entry().await? {
            let path = f.path();
            if path.is_file() {
                let pixiv = serde_json::from_str::<Pixiv>(
                    fs::read_to_string(path.as_path()).await?.as_str(),
                )?;
                for x in pixiv.data {
                    if !data.contains(&x) {
                        data.push(x);
                    }
                }
            }
        }
        let now = Local::now();
        let save = PixivJson {
            len: data.len() as u64,
            update_time: { now.timestamp() },
            data,
        };
        fs::write(
            format!("pixiv_{}.json", get_time(&now)),
            serde_json::to_string_pretty(&save)?,
        )
        .await?;
        //fs::write(format!("pixiv_{}.bin", get_time(&now)), bincode::serialize(&save)?).await?;
        Ok(())
    }
    //////////////////////////////////////////////////////////////////////////////////////////////////////
    let path = Path::new("p");
    if path.exists() {
        if fs::read_dir(path).await?.next_entry().await?.is_some() {
            move_directory_file_to_other_directory("p", processing.as_ref()).await?;
            process_file_to_json_and_bin(processing.as_ref()).await?;
            move_directory_file_to_other_directory(processing.as_ref(), completed.as_ref()).await?;
        } else {
            println!("no file in p");
        }
    } else {
        println!("p is not exist");
    }
    Ok(())
}

async fn save_to_json(path: impl AsRef<Path>) -> Result<()> {
    println!("save");
    let path = path.as_ref();
    if !path.exists() {
        fs::create_dir_all(path).await?;
    }
    match get_content().await {
        Ok(content) => {
            fs::write(path.join(get_time(&Local::now()) + ".json"), content).await?;
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
    Ok(())
}

async fn get_content() -> Result<String> {
    println!("get_content");
    let pixiv: Pixiv = CLIENT.get("https://api.lolicon.app/setu/v2?num=20&size=original&size=regular&size=small&size=thumb&size=mini&r18=2")
        .send().await?
        .json().await?;
    if pixiv.error.is_empty() {
        Ok(serde_json::to_string_pretty(&pixiv)?)
    } else {
        Err(anyhow!("请求限制"))
    }
}

fn get_time(now: &DateTime<Local>) -> String {
    println!("get_time");
    let current_time = now;
    format!(
        "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
        current_time.year(),
        current_time.month(),
        current_time.day(),
        current_time.hour(),
        current_time.minute(),
        current_time.second(),
    )
}

#[cfg(test)]
mod test {
    use crate::process_to_json_and_bin;

    #[tokio::test]
    async fn process_to_json_and_bin_should_work() {
        process_to_json_and_bin("processing", "completed")
            .await
            .unwrap();
    }
}
