use std::path::{Path, PathBuf};

use anyhow::Result;
use tokio::time::Instant;
use tracing::{info, warn};

use pixiv_lolicon_spider::utils::{
    delete_empty_json, generate_bin_from_json, merge_pixiv_to_json,
    modify_valid_field_from_bad_picture, unique_json,
};

#[tokio::main]
async fn main() -> Result<()> {
    picture_core::init_log();

    let root_path = Path::new(r"D:\Desktop\picture\save-spider-json-bin");
    let mut current_json_file = PathBuf::from("pixiv_merge_2023-08-30_19-32-18.json");

    let initial_json_file_flag = current_json_file.clone();

    /////////////////generate merge file/////////////////////
    info!("-------------generate merge file----------------");
    let generate_merge_file =
        merge_pixiv_to_json(root_path, Some(current_json_file.as_path())).await?;
    if generate_merge_file.is_some() {
        current_json_file = generate_merge_file.unwrap();
    }

    ///////////////////generate modify valid field file//////////////////
    info!("-------------generate modify valid field file----------------");
    let generate_modify_valid_field_file = modify_valid_field_from_bad_picture(
        root_path,
        current_json_file.as_path(),
        root_path.join("bad_picture"),
    )
    .await?;
    if generate_modify_valid_field_file.is_some() {
        current_json_file = generate_modify_valid_field_file.unwrap();
    }

    ///////////////////generate filter file//////////////////
    info!("-------------generate filter file----------------");
    let generate_filter_file = unique_json(root_path, current_json_file.as_path()).await?;
    if generate_filter_file.is_some() {
        current_json_file = generate_filter_file.unwrap();
    }

    //////////////////generate delete file////////////////////
    info!("-------------generate delete file----------------");
    let generate_delete_file = delete_empty_json(root_path, current_json_file.as_path()).await?;
    if generate_delete_file.is_some() {
        current_json_file = generate_delete_file.unwrap();
    }

    ///////////////////generate bin file//////////////////
    info!("-------------generate bin file----------------");
    if initial_json_file_flag == current_json_file {
        warn!("没有生成新的json文件");
        warn!("没有生成新的json文件, 故不需要生成bin文件");
        return Ok(());
    } else {
        let generate_bin_file =
            generate_bin_from_json(root_path.join("done").join(current_json_file.as_path()))
                .await?;
        if generate_bin_file.is_some() {
            current_json_file = generate_bin_file.unwrap();
        }
    }

    ///////////////////final json file//////////////////
    info!("-------------final file----------------");
    info!("\nfinal json file: {}", current_json_file.display());

    Ok(())
}

async fn merge_pixiv_to_json(
    root_path: impl AsRef<Path>,
    filename: Option<impl AsRef<Path>>,
) -> Result<Option<PathBuf>> {
    info!("开始 merge_pixiv_to_json");
    let start_time = Instant::now();

    let generate_file = merge_pixiv_to_json::run(root_path.as_ref(), filename).await?;
    let end = start_time.elapsed().as_secs();
    info!("结束 merge_pixiv_to_json 耗时: {}秒", end);
    Ok(generate_file)
}

async fn modify_valid_field_from_bad_picture(
    root_path: impl AsRef<Path>,
    filename: impl AsRef<Path>,
    bad_picture_dir: impl AsRef<Path>,
) -> Result<Option<PathBuf>> {
    info!("开始 modify_valid_filed");
    let start_time = Instant::now();

    let generate_file = modify_valid_field_from_bad_picture::run(
        root_path.as_ref(),
        filename.as_ref(),
        bad_picture_dir.as_ref(),
    )
    .await?;

    let end = start_time.elapsed().as_secs();
    info!("结束 modify_valid_filed 耗时: {}秒", end);
    Ok(generate_file)
}

async fn unique_json(
    root_path: impl AsRef<Path>,
    filename: impl AsRef<Path>,
) -> Result<Option<PathBuf>> {
    info!("开始 unique_json");
    let start_time = Instant::now();

    let generate_file = unique_json::run(root_path.as_ref(), filename.as_ref()).await?;

    let end = start_time.elapsed().as_secs();
    info!("结束 unique_json 耗时: {}秒", end);
    Ok(generate_file)
}

async fn delete_empty_json(
    root_path: impl AsRef<Path>,
    filename: impl AsRef<Path>,
) -> Result<Option<PathBuf>> {
    info!("开始 delete_empty_json");
    let start_time = Instant::now();

    let generate_file = delete_empty_json::run(root_path.as_ref(), filename.as_ref()).await?;

    let end = start_time.elapsed().as_secs();
    info!("结束 delete_empty_json 耗时: {}秒", end);
    Ok(generate_file)
}

async fn generate_bin_from_json(filepath: impl AsRef<Path>) -> Result<Option<PathBuf>> {
    info!("开始 generate_bin_from_json");
    let start_time = Instant::now();

    let generate_file = generate_bin_from_json::run(filepath.as_ref()).await?;

    let end = start_time.elapsed().as_secs();
    info!("结束 generate_bin_from_json 耗时: {}秒", end);
    Ok(generate_file)
}
