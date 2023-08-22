use std::path::PathBuf;
use std::{env, io};

use salvo::{handler, FlowCtrl, Request, Response};
use tracing::{error, info};

use crate::error::PError;
use crate::handle::Media;
use crate::picture::photo::{Photo, PhotoDescribe};
use crate::picture::{PictureDescribe, PictureLoad, PictureLoadType};
use crate::Result;

#[allow(unused_variables)] //req在release下没用到
#[handler]
pub async fn photo(req: &mut Request, res: &mut Response, ctrl: &mut FlowCtrl) -> Result<Media> {
    //debug模式下允许跨域
    #[cfg(debug_assertions)]
    {
        use salvo::http::HeaderValue;
        res.headers_mut()
            .insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    }
    let mut path: PathBuf;
    if cfg!(debug_assertions) {
        path = PathBuf::from("E:/spider/photos");
    } else {
        path = env::current_exe().unwrap().parent().unwrap().join("photo");
        path.push("picture");
    }
    if !path.exists() {
        error!("photo: {} not exists", path.display());
        return Err(PError::IOError(io::Error::from(io::ErrorKind::NotFound)));
    }
    let load = Photo::new().load(PictureLoadType::File(path)).await;
    let photo_describe: PhotoDescribe = req.parse_queries()?;

    let picture_entity = load
        .get(&PictureDescribe::PhotoDescribeType(photo_describe))
        .await?;
    info!("获取PictureEntity: {:?}\n", picture_entity);
    ctrl.cease();
    Ok(Media::Image(picture_entity.bytes().await?))
}
