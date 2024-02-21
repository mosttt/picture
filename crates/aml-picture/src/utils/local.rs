use std::env;
use std::path::PathBuf;

use sysinfo::{Disk, DiskExt, System, SystemExt};

pub async fn exe_disk_info() -> crate::Result<(PathBuf, u64, u64)> {
    let mut sys = System::new();
    sys.refresh_disks_list();
    sys.refresh_disks();

    let mut disks = Vec::from_iter(sys.disks());

    //把最长的挂在目录放在最前面，到时候优先匹配最满足的
    disks.sort_by_key(|x| -(x.mount_point().to_str().unwrap().len() as i32));

    let exe_file = env::current_exe().unwrap();

    let filter_fn = |x: &&&Disk| {
        let mount_point = x.mount_point().to_str().unwrap();
        let exe_file = exe_file.to_str().unwrap();
        exe_file.contains(mount_point)
    };

    let disk = disks.iter().find(filter_fn).unwrap();

    Ok((
        disk.mount_point().to_path_buf(),
        disk.available_space(),
        disk.total_space(),
    ))
}

///debug_assertions下为相对位置crates/aml-picture，release下为exe所在目录
pub fn exe_directory() -> crate::Result<PathBuf> {
    let current_exe_directory = if cfg!(debug_assertions) {
        PathBuf::from("crates/aml-picture")
    } else {
        let exe_file = env::current_exe()?;
        exe_file.parent().unwrap().to_path_buf()
    };
    //到时候删除，零时设置一下
    let current_exe_directory = PathBuf::from(r"F:\picture");
    Ok(current_exe_directory)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_get_exe_disk_info() {
        let (mount_point, available_space, total_space) = exe_disk_info().await.unwrap();
        println!(
            "disk {:?} available_space/total_space: {:.2}GB/{:.2}GB",
            mount_point,
            available_space as f64 / 1024.0 / 1024.0 / 1024.0,
            total_space as f64 / 1024.0 / 1024.0 / 1024.0
        );
    }
}
