use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use bytes::Bytes;

use picture_core::anime::AnimeData;
use picture_core::leg::LegData;
use picture_core::photo::PhotoData;
use picture_core::pixiv::PixivData;

use crate::error::PError;
use crate::picture::anime::AnimeDescribe;
use crate::picture::leg::LegDescribe;
use crate::picture::photo::PhotoDescribe;
use crate::picture::pixiv::PixivDescribe;
use crate::Result;

pub mod anime;
pub mod leg;
pub mod photo;
pub mod pixiv;

///图片的抽象
#[async_trait]
pub trait Picture: Send + Sync {
    async fn get(&self, picture_describe: &PictureDescribe) -> Result<PictureEntity>;
    async fn gets(&self, picture_describe: &PictureDescribe) -> Result<Vec<PictureEntity>>;
    fn allowed_file_name(title: &str) -> String
    where
        Self: Sized,
    {
        title.replace(
            [
                '#', '\'', '/', '\\', ':', '*', '?', '\"', '>', '<', '|', '&',
            ],
            "_",
        )
    }
}

///图片的加载方式
pub trait PictureLoad {
    async fn load(&self, from: PictureLoadType) -> Box<dyn Picture>;
}

//#[non_exhaustive]
pub enum PictureLoadType {
    File(PathBuf),
    //ConfigFile(String),
    //SQLLite(String),
}

///描述图片的规则
#[derive(Debug, Clone)]
pub enum PictureDescribe {
    PixivDescribeType(PixivDescribe),
    AnimeDescribeType(AnimeDescribe),
    PhotoDescribeType(PhotoDescribe),
    LegDescribeType(LegDescribe),
}

///获得的图片实体
#[derive(Debug, Clone)]
pub struct PictureEntity<'a> {
    pub data: PictureData,
    pub(crate) filepath: Cow<'a, Path>,
}

impl PictureEntity<'_> {
    ///懒加载图片的字节
    pub async fn bytes(&self) -> Result<Bytes> {
        let bytes = fs::read(self.filepath.as_ref())?;
        Ok::<Bytes, PError>(Bytes::from(bytes))
    }
}

///图片的数据
#[derive(Debug, Clone)]
pub enum PictureData {
    PixivDataType(PixivData),
    AnimeDataType(AnimeData),
    PhotoDataType(PhotoData),
    LegDataType(LegData),
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let x = "tit/le#".replace(
            [
                '#', '\'', '/', '\\', ':', '*', '?', '\"', '>', '<', '|', '&',
            ],
            "_",
        );
        println!("{}", x);
    }
}
