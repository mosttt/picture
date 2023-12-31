use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use bytes::Bytes;
use once_cell::sync::OnceCell;
use rand::seq::SliceRandom;
use rand::Rng;
use reqwest::Url;
use serde::Deserialize;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info, warn};

use picture_core::pixiv::{PixivData, PixivFile};

use crate::error::PError;
use crate::picture::{
    Picture, PictureData, PictureDescribe, PictureEntity, PictureLoad, PictureLoadType,
};
use crate::utils::local;
use crate::Result;

#[derive(Default, Debug, Clone)]
pub struct Pixiv {
    pub(crate) agent_domain: Option<Url>,
}

impl Pixiv {
    pub fn new() -> Self {
        Self {
            agent_domain: Some(Url::parse("https://pixiv.yuki.sh").unwrap()),
            //agent_domain: Some(Url::parse("https://i.pixiv.re").unwrap()),
        }
    }
}

impl PictureLoad for Pixiv {
    async fn load(&self, from: PictureLoadType) -> Box<dyn Picture> {
        match from {
            PictureLoadType::File(from) => Box::new(PixivLoadFromFile {
                root_file: from,
                agent_domain: self.agent_domain.clone(),
            }),
        }
    }
}

lazy_static::lazy_static! {static ref PIXIV:OnceCell<PixivFile> = OnceCell::new();}

pub struct PixivLoadFromFile {
    pub(crate) root_file: PathBuf,
    pub(crate) agent_domain: Option<Url>,
}

impl PixivLoadFromFile {
    fn get_pivix_file(&self) -> &PixivFile {
        let pivix = PIXIV.get_or_init(|| {
            info!("读取Pixiv文件: {:?}", self.root_file.as_path());
            let s = std::fs::read(self.root_file.as_path()).expect("读取文件失败");
            bincode::deserialize::<PixivFile>(&s[..]).unwrap()
        });
        pivix
    }

    ///根据传入的描述生成符合的PixivData的列表
    fn get_picture_data_list(&self, pixiv_describe: &PixivDescribe) -> Vec<&PixivData> {
        let pivix_file = self.get_pivix_file();
        let data_iter = pivix_file.data.iter().filter(|p| p.valid);

        return if pixiv_describe.is_empty_exclude_num() {
            let data = data_iter.filter(|p| p.valid).collect();
            data
        } else {
            let get_filter_flag = |data: &&PixivData| -> bool {
                //let  flag = true;
                if let Some(r18) = pixiv_describe.r18 {
                    if r18 == 0 && data.r18 != false {
                        return false;
                    } else if r18 == 1 && data.r18 != true {
                        return false;
                    }
                }

                if let Some(keyword) = &pixiv_describe.keyword {
                    for k in keyword {
                        if !data.tags.contains(k) {
                            return false;
                        }
                    }
                }

                if let Some(exclude_ai) = pixiv_describe.exclude_ai {
                    if exclude_ai == true && data.ai_type == 2 {
                        return false;
                    }
                }

                if let Some(date_after) = pixiv_describe.date_after {
                    if data.upload_date < date_after {
                        return false;
                    }
                }

                if let Some(date_before) = pixiv_describe.date_before {
                    if data.upload_date > date_before {
                        return false;
                    }
                }
                //return flag;
                return true;
            };

            let data: Vec<_> = data_iter.filter(get_filter_flag).collect();
            data
        };
    }

    ///记录bad picture到与pixiv文件同目录下的bad_picture.txt文件中
    async fn record_bad_picture(&self, pixiv_data: &PixivData, bytes_len: usize) -> Result<()> {
        let filename = self.root_file.parent().unwrap().join("bad_picture.txt");

        let mut file = fs::File::options()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(filename.as_path())
            .await?;
        let mut current_content = String::new();

        let s = format!(
            "title: {} uid: {} pid: {} p: {} upload_date: {} bytes_len: {} bytes\n",
            pixiv_data.title,
            pixiv_data.uid,
            pixiv_data.pid,
            pixiv_data.p,
            pixiv_data.upload_date,
            bytes_len
        );

        file.read_to_string(&mut current_content).await?;
        if !current_content.contains(s.as_str()) {
            file.write_all(s.as_bytes()).await?;
            warn!("write==> {}", s);
        }
        Ok(())
    }

    #[allow(unused)]
    async fn read_bad_picture(&self) -> Result<&Arc<Mutex<Vec<&'static PixivData>>>> {
        let _filename = self.root_file.parent().unwrap().join("bad_picture.txt");
        // static BAD_PICTURE_LIST:Arc<Mutex<Vec<&PixivData>>> = Arc::new(Mutex::new(Vec::new()));
        // Ok(&
        todo!()
    }

    ///根据设置的代理网址，生成original图片的地址
    fn get_original_url(&self, data: &PixivData) -> Url {
        let url = data.urls.original.as_str();

        if let Some(agent_domain) = &self.agent_domain {
            let url = if url.contains(agent_domain.as_str()) {
                url.parse().unwrap()
            } else {
                url.replace("https://i.pixiv.re", agent_domain.as_str())
                    .replace("https://i.pixiv.cat", agent_domain.as_str())
                    .parse()
                    .unwrap()
            };
            info!("代理地址: {:?}", url);
            url
        } else {
            Url::from_str(url).unwrap()
        }
    }

    async fn download_picture(
        &self,
        pixiv_data: &PixivData,
        filename: impl AsRef<Path>,
    ) -> Result<Bytes> {
        //检查存储空间是否足够
        //单位字节B
        let (mount_point, available_space, _total_space) = local::exe_disk_info().await?;
        //小于100MB不下载
        let need_space = 1024 * 1024 * 100; //100MB
        if available_space <= need_space {
            error!(
                "本地磁盘: {} ，空间不足: {}MB，当前可用: {}MB",
                mount_point.display(),
                need_space / 1024 / 1024,
                available_space / 1024 / 1024
            );
            return Err(PError::LocalDiskSpaceNotEnoughError(format!(
                "{}MB",
                need_space / 1024 / 1024
            )));
        }

        //下载
        let url = self.get_original_url(pixiv_data);
        let bytes = reqwest::get(url.as_ref()).await?.bytes().await?;
        //10KB
        if bytes.len() < 10 * 1024 {
            self.record_bad_picture(pixiv_data, bytes.len()).await?;
            return Err(PError::ReqwestBadPictureError(url.to_string()));
        }
        fs::write(filename.as_ref(), bytes.as_ref()).await?;

        info!(
            "下载图片: {:?}\n大小: {:?}KB",
            filename.as_ref(),
            bytes.len() as i64 / 1024
        );
        Ok(bytes)
    }
    ///拿的是original的地址，其他地址需要更改 pixiv_data.ext（图片后缀名）
    async fn get_picture_entity(&self, pixiv_data: &PixivData) -> Result<PictureEntity> {
        let title = Self::allowed_file_name(pixiv_data.title.as_str());
        //( _｀ω´)ゞ@1234-82355973_1.jpg
        let title = format!(
            "{}@{}-{}_{}.{}",
            title, pixiv_data.uid, pixiv_data.pid, pixiv_data.p, pixiv_data.ext
        );

        let mut path = local::exe_directory()?;
        path.push("pixiv");
        path.push("picture");
        //创建pixiv\picture文件夹
        if !path.exists() {
            fs::create_dir_all(path.as_path()).await?;
        }
        //构建pixiv\picture\title.jpg文件路径
        let file_name = path.join(title.as_str());
        if !file_name.exists() {
            //下载图片
            self.download_picture(pixiv_data, file_name.as_path())
                .await?;
        }
        Ok(PictureEntity {
            data: PictureData::PixivDataType(pixiv_data.clone()),
            filepath: Cow::from(file_name),
        })
    }
}

impl PixivLoadFromFile {
    pub async fn get_only_local(&self) -> Result<PictureEntity> {
        let mut path = PathBuf::from(env::current_exe()?.parent().unwrap());
        path.push("pixiv");
        path.push("picture");

        if !path.exists() {
            fs::create_dir_all(path.as_path()).await?;
        }

        let mut files = Vec::with_capacity(10000);
        let mut dir = fs::read_dir(path).await?;
        while let Some(res) = dir.next_entry().await? {
            files.push(res);
        }
        let file = if let Some(random_file) = files.choose(&mut rand::thread_rng()) {
            random_file
        } else {
            error!("No files found in the directory");
            panic!("No files found in the directory")
        };
        Ok(PictureEntity {
            data: PictureData::PixivDataType(PixivData::default()),
            filepath: Cow::from(file.path()),
        })
    }
}

#[async_trait]
impl Picture for PixivLoadFromFile {
    async fn get(&self, picture_describe: &PictureDescribe) -> Result<PictureEntity> {
        let pixiv_describe =
            if let PictureDescribe::PixivDescribeType(picture_describe) = picture_describe {
                picture_describe
            } else {
                panic!("传入的参数不是PixivDescribeType");
            };
        info!(
            "获取PictureDescribe: DescribeIsEmpty *{}* {:?}",
            pixiv_describe.is_empty_exclude_num(),
            pixiv_describe
        );
        let data = self.get_picture_data_list(pixiv_describe);

        if data.is_empty() {
            return Err(PError::PictureDataEmptyError(format!(
                "{:?}",
                picture_describe
            )));
        }
        let data = data
            .get(rand::thread_rng().gen_range(0..data.len()))
            .unwrap();

        self.get_picture_entity(data).await
        //}
    }

    async fn gets(&self, _picture_describe: &PictureDescribe) -> Result<Vec<PictureEntity>> {
        todo!()
    }
}

///暂时本地直接获取
pub struct PixivLoadFromFileGenerator;

///暂时本地直接获取实现
impl PixivLoadFromFileGenerator {
    pub fn generate() -> PixivLoadFromFile {
        PixivLoadFromFile {
            root_file: PathBuf::from(""),
            agent_domain: None,
        }
    }
}

///描述图片的规则
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PixivDescribe {
    r18: Option<u8>,
    num: Option<u8>,
    uid: Option<u64>,
    keyword: Option<Vec<String>>,
    size: Option<Size>,
    #[serde(rename = "dateAfter")]
    date_after: Option<i64>,
    #[serde(rename = "dateBefore")]
    date_before: Option<i64>,
    #[serde(rename = "excludeAI")]
    exclude_ai: Option<bool>,
}

impl PixivDescribe {
    pub fn is_empty(&self) -> bool {
        self.r18.is_none()
            && self.uid.is_none()
            && self.num.is_none()
            && self.keyword.is_none()
            && self.size.is_none()
            && self.date_after.is_none()
            && self.date_before.is_none()
            && self.exclude_ai.is_none()
    }
    pub fn is_empty_exclude_num(&self) -> bool {
        self.r18.is_none()
            && self.uid.is_none()
            && self.keyword.is_none()
            && self.size.is_none()
            && self.date_after.is_none()
            && self.date_before.is_none()
            && self.exclude_ai.is_none()
    }
}

#[derive(Debug, Clone, Deserialize)]
enum Size {
    #[serde(rename = "original")]
    Original,
    #[serde(rename = "regular")]
    Regular,
    #[serde(rename = "small")]
    Small,
    #[serde(rename = "thumb")]
    Thumb,
    #[serde(rename = "mini")]
    Mini,
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use tokio::fs;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn record_bad_picture() -> crate::Result<()> {
        let filename = Path::new(r"D:\Desktop\picture\crates\aml-picture\pixiv\bad_picture.txt");

        let mut file = fs::File::options()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(filename)
            .await?;
        let mut current_content = String::new();

        let s = String::from("title: 大神ミオ author: 無人 pid: 98701827 p: 1 bytes_len: 54d");

        file.read_to_string(&mut current_content).await?;
        if !current_content.contains(s.as_str()) {
            file.write_all(s.as_bytes()).await?;
            println!("write==> {}", s);
        } else {
            println!("包含")
        }
        Ok(())
    }
}
