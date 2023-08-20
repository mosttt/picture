pub use picture_core::pixiv::{PixivData as Data, PixivFile as PixivJson, Urls};
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pixiv {
    pub error: String,
    pub data: Vec<Data>,
}
