#![feature(async_closure)]

use std::env;
use std::path::PathBuf;

use anyhow::Result;

use pixiv_lolicon_spider::utils::modify_name;

#[tokio::main]
async fn main() -> Result<()> {
    picture_core::init_log();

    let json_filename: PathBuf;
    let picture_directory: PathBuf;

    if cfg!(windows) {
        json_filename = PathBuf::from(
            r"D:\Desktop\picture\save-spider-json-bin\done\pixiv_merge_2023-08-22_15-16-28.json",
        );
        picture_directory = PathBuf::from("crates/aml-picture/pixiv/picture");
        //picture_directory = PathBuf::from(r"F:\picture\pixiv\picture");
    } else {
        let pixiv_directory = env::current_exe()?.parent().unwrap().join("pixiv");
        json_filename = pixiv_directory.join("pixiv.json");
        picture_directory = pixiv_directory.join("picture");
    }
    modify_name::run(json_filename, picture_directory).await?;
    Ok(())
}
