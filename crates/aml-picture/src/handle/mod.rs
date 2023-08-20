use bytes::Bytes;
use salvo::{Piece, Response};

pub mod anime;
pub mod leg;
pub mod photo;
pub mod pixiv;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum Media {
    Image(Bytes),
    Video(Bytes),
}

impl Piece for Media {
    fn render(self, res: &mut Response) {
        match self {
            Media::Image(image) => {
                res.body(image.into());
            }
            Media::Video(video) => {
                res.body(video.into());
            }
        }
    }
}
