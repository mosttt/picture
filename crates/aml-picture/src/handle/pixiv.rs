use std::io;

use salvo::{handler, FlowCtrl, Request, Response};
use tracing::{error, info};

use crate::error::PError;
use crate::handle::Media;
use crate::picture::pixiv::{Pixiv, PixivDescribe, PixivLoadFromFileGenerator};
use crate::picture::{PictureDescribe, PictureLoad, PictureLoadType};
use crate::utils::local;
use crate::Result;

#[allow(unused_variables)] //req在release下没用到
#[handler]
pub async fn pixiv(req: &mut Request, res: &mut Response, ctrl: &mut FlowCtrl) -> Result<Media> {
    //debug模式下允许跨域
    #[cfg(debug_assertions)]
    {
        use salvo::http::HeaderValue;
        res.headers_mut()
            .insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    }

    let mut path = local::exe_directory()?;
    path.push("pixiv");
    path.push("pixiv.bin");

    if !path.exists() {
        error!("pixiv.bin not found");
        return Err(PError::IOError(io::Error::from(io::ErrorKind::NotFound)));
    }
    let pivix_describe: PixivDescribe = req.parse_queries()?;

    let load = Pixiv::new().load(PictureLoadType::File(path)).await;

    let picture_entity = load
        .get(&PictureDescribe::PixivDescribeType(pivix_describe))
        .await?;
    info!("获取PictureEntity filepath: {:?}\n", picture_entity.filepath);
    ctrl.cease();
    Ok(Media::Image(picture_entity.bytes().await?))
}

#[allow(unused_variables)] //req在release下没用到
#[handler]
pub async fn pixiv_only_local(res: &mut Response, ctrl: &mut FlowCtrl) -> Result<Media> {
    //debug模式下允许跨域
    #[cfg(debug_assertions)]
    {
        use salvo::http::HeaderValue;
        res.headers_mut()
            .insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    }

    let p = PixivLoadFromFileGenerator::generate();
    let picture_entity = p.get_only_local().await?;
    info!("获取OnlyLocal PictureEntity: {:?}\n", picture_entity.filepath);
    ctrl.cease();
    Ok(Media::Image(picture_entity.bytes().await?))
}
