// use std::env;
//
// use sysinfo::{Disk, DiskExt, System, SystemExt};
//
// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let mut sys = System::new();
//     sys.refresh_disks_list();
//     sys.refresh_disks();
//     let mut disks = Vec::from_iter(sys.disks());
//
//     //把最长的挂在目录放在最前面，到时候优先匹配最满足的
//     disks.sort_by_key(|x| -(x.mount_point().to_str().unwrap().len() as i32));
//
//     println!("=> disks:");
//     for disk in disks.iter() {
//         println!(
//             "disk {:?} {} available_space/total_space: {:.2}GB/{:.2}GB",
//             disk.name(),
//             disk.mount_point().display(),
//             disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0,
//             disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0
//         );
//     }
//     println!();
//     let exe_file = env::current_exe().unwrap();
//     println!("exe_file: {}", exe_file.display());
//
//     let filter_fn = |x: &&&Disk| {
//         let mount_point = x.mount_point().to_str().unwrap();
//         let exe_file = exe_file.to_str().unwrap();
//         exe_file.contains(mount_point)
//     };
//
//     let disk = disks.iter().find(filter_fn).unwrap();
//
//     println!(
//         "disk mount_point: {} available_space/total_space: {:.2}GB/{:.2}GB",
//         disk.mount_point().display(),
//         disk.available_space() as f64 / 1024.0 / 1024.0 / 1024.0,
//         disk.total_space() as f64 / 1024.0 / 1024.0 / 1024.0
//     );
//     Ok(())
// }
//
// #[cfg(test)]
// mod test {
//     #[test]
//     fn t() {
//         let mut v = vec!["/", "/home", "/home/aml", "/root"];
//         let path = "/home/aml/exe/a";
//         v.sort_by_key(|x| -(x.len() as i32));
//         println!("{:?}", v);
//         let x = v.iter().find(|&&v| path.contains(v)).unwrap();
//         println!("{:?}", x);
//     }
// }
