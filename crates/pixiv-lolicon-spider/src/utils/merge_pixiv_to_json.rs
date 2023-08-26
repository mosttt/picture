use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use anyhow::Result;
use tokio::fs;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use picture_core::pixiv::{PixivData, PixivFile};

static PROCESS_NEW_DATA: AtomicU32 = AtomicU32::new(0);
static ADD_DATA: AtomicU32 = AtomicU32::new(0);
static DISCARD_DATA: AtomicU32 = AtomicU32::new(0);
static SAME_DATA: AtomicU32 = AtomicU32::new(0);
static REPLACE_DATA: AtomicU32 = AtomicU32::new(0);
static DELETE_DATA: AtomicU32 = AtomicU32::new(0);

pub async fn run(
    root_path: impl AsRef<Path>,
    filename: impl AsRef<Path>,
) -> Result<Option<PathBuf>> {
    let root_path = root_path.as_ref();
    let filename = filename.as_ref();

    let mut dir = fs::read_dir(root_path.join("temporary")).await?;

    let pixiv = Arc::new(Mutex::new(Vec::with_capacity(20000)));
    let move_files = Arc::new(Mutex::new(Vec::with_capacity(1000)));

    let mut join_handles = Vec::with_capacity(1000);

    info!("read pixiv json...");
    read_pixiv_json_to_vec(root_path.join("done").join(filename), pixiv.clone()).await?;

    while let Some(d) = dir.next_entry().await? {
        let pixiv = pixiv.clone();
        let move_files = move_files.clone();
        let join_handle = tokio::spawn(async move {
            let path = d.path();
            if path.is_file() {
                debug!("path: {:?}", path);
                let c = fs::read_to_string(path.as_path()).await?;
                let p: PixivFile = serde_json::from_str(c.as_str())?;
                for x in p.data {
                    PROCESS_NEW_DATA.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                    let pixiv = &mut *pixiv.lock().await;
                    if !pixiv.contains(&x) {
                        if !is_empty_by_title_tag(&x) {
                            ADD_DATA.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            pixiv.push(x);
                        } else {
                            DISCARD_DATA.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    } else {
                        process(pixiv, &x).await?;
                    }
                }
                move_files.lock().await.push(path);
            }
            Ok::<(), anyhow::Error>(())
        });
        join_handles.push(join_handle);
    }

    for join_handle in join_handles {
        join_handle.await??;
    }

    info!("pixiv json len: {}", pixiv.lock().await.len());

    if move_files.lock().await.len() == 0 {
        warn!("no needed merged files");
        return Ok(None);
    }

    info!(
        "process new data: {}",
        PROCESS_NEW_DATA.load(std::sync::atomic::Ordering::SeqCst)
    );
    info!(
        "add data: {}",
        ADD_DATA.load(std::sync::atomic::Ordering::SeqCst)
    );
    info!(
        "discard data: {}",
        DISCARD_DATA.load(std::sync::atomic::Ordering::SeqCst)
    );
    info!(
        "same data: {}",
        SAME_DATA.load(std::sync::atomic::Ordering::SeqCst)
    );
    info!(
        "replace data: {}",
        REPLACE_DATA.load(std::sync::atomic::Ordering::SeqCst)
    );
    info!(
        "delete data: {}",
        DELETE_DATA.load(std::sync::atomic::Ordering::SeqCst)
    );
    let actually_add_and_update_data = ADD_DATA.load(std::sync::atomic::Ordering::SeqCst)
        + REPLACE_DATA.load(std::sync::atomic::Ordering::SeqCst);
    info!(
        "actually add and replace data: {}",
        actually_add_and_update_data
    );
    assert_eq!(
        PROCESS_NEW_DATA.load(std::sync::atomic::Ordering::SeqCst),
        ADD_DATA.load(std::sync::atomic::Ordering::SeqCst)
            + DISCARD_DATA.load(std::sync::atomic::Ordering::SeqCst)
            + SAME_DATA.load(std::sync::atomic::Ordering::SeqCst)
    );
    // process new data = add data + same data
    // replace data are included in same data

    if actually_add_and_update_data == 0 {
        move_file(&*move_files.lock().await, root_path).await?;
        warn!("no new data");
        return Ok(None);
    }
    let now = chrono::Local::now();

    let lock = pixiv.lock().await;
    let save = PixivFile {
        len: lock.len() as u64,
        update_time: now.timestamp(),
        data: Vec::from_iter(lock.iter().cloned()),
    };
    info!("save pixiv len: {}", lock.len());
    drop(lock);

    let string = serde_json::to_string_pretty(&save)?;

    let mut path = root_path.join("done");
    if !path.exists() {
        fs::create_dir_all(path.as_path()).await?;
    }
    path.push(format!(
        "pixiv_merge_{}.json",
        now.format("%Y-%m-%d_%H-%M-%S")
    ));
    info!("save path: {:?}", path.as_path());
    fs::write(path.as_path(), string).await?;

    move_file(&*move_files.lock().await, root_path).await?;

    Ok(Some(path))
}

/// 碰到相同的数据
/// 先看current的title和tags是否为空，如果为空则不替换
/// 如果不为空，检查pixiv中相同的那个数据的title和tags是否为空，如果为空则替换
/// 如果pixiv和current中的title和tags都不为空，检查upload_date，如果current的upload_date大于pixiv中相同的那个数据的upload_date，则替换
#[tracing::instrument(skip_all)]
async fn process(pixiv: &mut Vec<PixivData>, current: &PixivData) -> anyhow::Result<()> {
    SAME_DATA.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let mut same: Vec<_> = pixiv.iter_mut().filter(|x| *x == current).collect();

    assert_eq!(same.len(), 1);

    let p = &mut **same.get_mut(0).unwrap();
    debug!(
        "have same one\np title: {} pid&p: {}\ncurrent title: {} pid&p: {}",
        p.title,
        format!("{}&{}", p.pid, p.p),
        current.title,
        format!("{}&{}", current.pid, current.p)
    );

    let p_is_empty = is_empty_by_title_tag(p);
    let current_is_empty = is_empty_by_title_tag(current);

    //如果都不为空，则比较upload_date
    if !p_is_empty && !current_is_empty {
        //如果current的upload_date大于pixiv中相同的那个数据的upload_date，则替换
        //否则不处理，及不添加进去
        if current.upload_date > p.upload_date {
            REPLACE_DATA.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            debug!(
                r"all not empty
            replace p title: {} pid&p: {}
            with current title: {} pid&p: {}",
                p.title,
                format!("{}&{}", p.pid, p.p),
                current.title,
                format!("{}&{}", current.pid, current.p)
            );
            *p = current.clone();
        } else if current.upload_date == p.upload_date {
            debug!("一模一样，原封不动")
        } else {
            debug!("current的upload_date小于pixiv中的，不处理")
        }
    }
    //如果都为空，则删除pixiv中的并丢弃current
    else if p_is_empty && current_is_empty {
        DELETE_DATA.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        debug!(
            "all empty 要个鸡毛
            remove p title: {} pid&p: {}",
            p.title,
            format!("{}&{}", p.pid, p.p),
        );
        let same_position = pixiv.iter_mut().position(|x| x == current).unwrap();

        pixiv.remove(same_position);
    } else if p_is_empty {
        REPLACE_DATA.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        //如果pixiv中相同的那个数据的title和tags为空，则替换
        debug!(
            r"pixiv is empty
            replace p title: {} pid&p: {}
            with current title: {} pid&p: {}",
            p.title,
            format!("{}&{}", p.pid, p.p),
            current.title,
            format!("{}&{}", current.pid, current.p)
        );
        *p = current.clone();
    } else {
        //剩下的就是current_is_empty，不处理，及不添加进去
        debug!(
            "什么垃圾，current为空\ncurrent title: {} pid&p: {}",
            current.title,
            format!("{}&{}", current.pid, current.p)
        );
    }
    Ok(())
}

async fn move_file(move_files: &Vec<PathBuf>, root_path: impl AsRef<Path>) -> Result<()> {
    info!("move files: {:?}", move_files.len());
    for p in move_files {
        fs::rename(
            p.as_path(),
            root_path
                .as_ref()
                .join("backup")
                .join(p.file_name().unwrap()),
        )
        .await?;
    }
    info!("move files completed");
    Ok(())
}

fn is_empty_by_title_tag(pixiv_data: &PixivData) -> bool {
    pixiv_data.title.is_empty() || pixiv_data.tags.is_empty()
}

async fn read_pixiv_json_to_vec(
    file: impl AsRef<Path>,
    vec: Arc<Mutex<Vec<PixivData>>>,
) -> Result<()> {
    let string = fs::read_to_string(file.as_ref()).await?;
    let x: PixivFile = serde_json::from_str(string.as_str())?;
    vec.lock().await.extend(x.data);
    Ok(())
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    #[tokio::test]
    async fn t() -> Result<()> {
        let mut v = vec![1, 2, 3, 3, 4, 5];
        let mut x: Vec<_> = v.iter_mut().filter(|x| **x == 3).collect();
        if let Some(elem) = x.get_mut(0) {
            **elem = 42;
        }
        println!("{:?}", v);
        Ok(())
    }
}
