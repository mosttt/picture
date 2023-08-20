use reqwest::StatusCode;
use salvo::prelude::Text;
use salvo::{Piece, Response};
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum PError {
    #[error("reqwest error: `{0}`")]
    ReqwestOtherError(#[from] reqwest::Error),

    #[error("reqwest bad picture error, url: {0}")]
    ReqwestBadPictureError(String),

    #[error("io error: `{0}`")]
    IOError(#[from] std::io::Error),

    #[error("local disk space not enough < {0}")]
    LocalDiskSpaceNotEnoughError(String),

    #[error("picture data empty error, in this describe: {0}")]
    PictureDataEmptyError(String),

    #[error("ssh2 error: `{0}`")]
    SSH2Error(#[from] ssh2::Error),

    ///ssh2用户操作错误
    #[error("ssh2 error: {0}")]
    SSH2OPError(String),

    #[error("salvo http parse error: `{0}`")]
    SalvoHttpParseError(#[from] salvo::http::ParseError),
}

//unsafe impl Send for PError {}

impl Piece for PError {
    fn render(self, res: &mut Response) {
        error!("{:?}", self.to_string());
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(Text::Plain(self.to_string()));
    }
}
