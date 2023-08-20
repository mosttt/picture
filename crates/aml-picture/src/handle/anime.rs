use std::env;
use std::path::PathBuf;

use salvo::{handler, FlowCtrl, Request, Response};
use tokio::io;
use tracing::{error, info};

use crate::error::PError;
use crate::handle::Media;
use crate::picture::anime::{Anime, AnimeDescribe};
use crate::picture::{PictureDescribe, PictureLoad, PictureLoadType};
use crate::Result;

#[allow(unused_variables)] //req在release下没用到
#[handler]
pub async fn anime(req: &mut Request, res: &mut Response, ctrl: &mut FlowCtrl) -> Result<Media> {
    //debug模式下允许跨域
    #[cfg(debug_assertions)]
    {
        use salvo::http::HeaderValue;
        res.headers_mut()
            .insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    }
    let mut path: PathBuf;
    if cfg!(debug_assertions) {
        path = PathBuf::from("E:/spider/anime");
    } else {
        path = env::current_exe().unwrap().parent().unwrap().join("anime");
        path.push("picture");
    }
    if !path.exists() {
        error!("anime: {} not exists",path.display());
        return Err(PError::IOError(io::Error::from(io::ErrorKind::NotFound)));
    }
    let load = Anime::new().load(PictureLoadType::File(path)).await;
    let anime_describe: AnimeDescribe = req.parse_queries()?;

    let picture_entity = load
        .get(&PictureDescribe::AnimeDescribeType(anime_describe))
        .await?;
    info!("获取PictureEntity: {:?}\n", picture_entity);
    ctrl.cease();
    Ok(Media::Image(picture_entity.bytes().await?))
}
