use std::borrow::Cow;
use std::path::PathBuf;

use async_trait::async_trait;
use once_cell::sync::OnceCell;
use rand::Rng;
use serde::Deserialize;
use tracing::info;

use picture_core::photo::PhotoData;

use crate::picture::{
    Picture, PictureData, PictureDescribe, PictureEntity, PictureLoad, PictureLoadType,
};

#[derive(Default, Debug, Clone)]
pub struct Photo;

impl Photo {
    pub fn new() -> Self {
        Self
    }
}

impl PictureLoad for Photo {
    async fn load(&self, from: PictureLoadType) -> Box<dyn Picture> {
        match from {
            PictureLoadType::File(path) => Box::new(PhotoLoadFromFile { root_path: path }),
        }
    }
}
pub(crate) struct PhotoLoadFromFile {
    pub(crate) root_path: PathBuf,
}

lazy_static::lazy_static! {static ref PHOTO:OnceCell<Vec<std::fs::DirEntry>> = OnceCell::new();}
impl PhotoLoadFromFile {
    fn get_file_list(&self) -> &Vec<std::fs::DirEntry> {
        let photo = PHOTO.get_or_init(|| {
            info!("读取Photo文件: {:?}", self.root_path.as_path());
            let mut file_list = Vec::with_capacity(10000);
            let mut dir = std::fs::read_dir(self.root_path.as_path()).unwrap();
            while let Some(Ok(file)) = dir.next() {
                if file.path().is_file() {
                    file_list.push(file);
                }
            }
            file_list
        });
        photo
    }
}

#[async_trait]
impl Picture for PhotoLoadFromFile {
    async fn get(&self, picture_describe: &PictureDescribe) -> crate::Result<PictureEntity> {
        let photo_describe =
            if let PictureDescribe::PhotoDescribeType(picture_describe) = picture_describe {
                picture_describe
            } else {
                panic!("传入的参数不是PhotoDescribeType");
            };
        info!(
            "获取PictureDescribe: DescribeIsEmpty *{}* {:?}",
            photo_describe.is_empty_exclude_num(),
            photo_describe
        );
        if photo_describe.is_empty_exclude_num() {
            let file_list = self.get_file_list();

            let i = rand::thread_rng().gen_range(0..file_list.len());
            let file = file_list.get(i).unwrap();
            Ok(PictureEntity {
                data: PictureData::PhotoDataType(PhotoData::default()),
                filepath: Cow::Owned(file.path()),
            })
        } else {
            todo!()
        }
    }

    async fn gets(&self, _picture_describe: &PictureDescribe) -> crate::Result<Vec<PictureEntity>> {
        todo!()
    }
}

///描述图片的规则
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PhotoDescribe {
    num: Option<u8>,
    size: Option<u32>,
}

impl PhotoDescribe {
    pub fn is_empty(&self) -> bool {
        self.num.is_none() && self.size.is_none()
    }
    pub fn is_empty_exclude_num(&self) -> bool {
        self.size.is_none()
    }
}
