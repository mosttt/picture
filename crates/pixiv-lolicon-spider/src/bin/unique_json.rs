use std::path::Path;

use anyhow::Result;
use tokio::fs;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

use picture_core::pixiv::{PixivData, PixivFile};

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = Instant::now();
    picture_core::init_log();

    run(
        r"D:\Desktop\picture\save-spider-json-bin",
        "pixiv_merge_2023-08-18_20-07-55.json",
    )
    .await?;

    let end = start_time.elapsed().as_secs();
    println!("耗时: {}秒", end);
    Ok(())
}

async fn run(root_path: impl AsRef<Path>, filename: impl AsRef<str>) -> Result<()> {
    let root_path = root_path.as_ref();
    let filename = filename.as_ref();
    let json_filename = root_path.join("done").join(filename);

    let json_string = fs::read_to_string(json_filename).await?;
    let json: PixivFile = serde_json::from_str(json_string.as_str())?;

    let pixiv = json.data;
    let copy_pixiv = pixiv.clone();

    info!("pixiv count: {}", pixiv.len());

    let mut need_process = Vec::new();
    let mut unneeded_process = Vec::new();

    copy_pixiv.iter().for_each(|x| {
        let v: Vec<_> = pixiv
            .iter()
            .filter(|y| y.pid == x.pid && y.p == x.p)
            .collect();
        if v.len() == 2 {
            assert_eq!(v.len(), 2);
            assert_eq!(v[0].uid, v[1].uid);
            //有两个相同的PixivData，防止重复添加一对，每对只添加一次
            if !need_process.contains(&v) {
                need_process.push(v);
            }
        } else if v.len() == 1 {
            unneeded_process.push(x);
        } else {
            error!("v.len() != 1 && v.len() != 2 v.len(): {}", v.len());
            panic!("v.len() != 1 && v.len() != 2 v.len(): {}", v.len());
        }
    });

    info!("need_process count: {}", need_process.len() * 2);
    info!("unneeded_process count: {}", unneeded_process.len());

    assert_eq!(unneeded_process.len() + need_process.len() * 2, pixiv.len());

    if need_process.len() == 0 {
        warn!("no same data");
        return Ok(());
    }

    let need_process_count = need_process.len();

    //title和tag 为""的那个删除，如果两个都为空，均删除,更新json ...ok
    //之后merge的用uid,pid,p来判断是否重复。发现重复时比对title和tag，留下不空的那个；若都不空，留下upload_date最新的那个。更新json ...ok
    //aml-picture中通过uid,pid,p来获取图片，理论上来说不会重复，因为json文件中已经去重了。获取时检查len，如果不为1，打印错误日志
    let finish_process: Vec<_> = need_process
        .iter_mut()
        .enumerate()
        .flat_map(|(_index, v)| {
            process(v).unwrap()
            // println!("count: {}", x.0 + 1);
            // x.1.iter().for_each(|y| {
            //     println!("data: {:?}\n\n", y);
            // });
        })
        .cloned()
        .collect();

    info!("finish_process count: {}", finish_process.len());
    info!(
        "remove count: {}",
        need_process_count * 2 - finish_process.len()
    );

    unneeded_process.extend(finish_process);
    info!("final count: {}", unneeded_process.len());

    save_json_file(unneeded_process.into_iter().cloned().collect(), root_path).await?;
    Ok(())
}

fn process<'a, 'b>(v: &'a mut Vec<&'b PixivData>) -> Result<&'a Vec<&'b PixivData>>
where
    'b: 'a,
{
    let one = v[0];
    let two = v[1];
    let one_flag = is_delete_by_title_tag(one);
    let two_flag = is_delete_by_title_tag(two);

    if two_flag {
        debug!("remove two: {:?}", two);
        v.remove(1);
    }
    if one_flag {
        debug!("remove one: {:?}", one);
        v.remove(0);
    }

    if !one_flag && !two_flag {
        debug!("all is not empty:");
        if one.upload_date > two.upload_date {
            debug!("so remove two: {:?}", two);
            v.remove(1);
        } else if two.upload_date > one.upload_date {
            debug!("so remove one: {:?}", one);
            v.remove(0);
        } else {
            error!("upload_date相同，无法判断删除哪个");
            panic!("upload_date相同，无法判断删除哪个");
        }
    }
    Ok(v)
}

fn is_delete_by_title_tag(pixiv_data: &PixivData) -> bool {
    pixiv_data.title.is_empty() || pixiv_data.tags.is_empty()
}

async fn save_json_file(pixiv: Vec<PixivData>, root_path: impl AsRef<Path>) -> Result<()> {
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
        "pixiv_filter_{}.json",
        now.format("%Y-%m-%d_%H-%M-%S")
    ));
    info!("save path: {:?}", path.as_path());
    fs::write(path.as_path(), string).await?;
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn t() {
        let i = "a_b_c_".find('_').unwrap();
        println!("{}", i)
    }
}
