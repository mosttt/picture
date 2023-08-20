use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixivFile {
    pub len: u64,
    pub update_time: i64,
    pub data: Vec<PixivData>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PixivData {
    pub pid: u64,
    pub p: u64,
    pub uid: u64,
    pub title: String,
    pub author: String,
    pub r18: bool,
    pub width: u64,
    pub height: u64,
    pub tags: Vec<String>,
    pub ext: String,
    pub ai_type: i64,
    pub upload_date: i64,
    pub urls: Urls,
}

impl PartialEq for PixivData {
    fn eq(&self, other: &Self) -> bool {
        self.pid == other.pid && self.p == other.p && self.uid == other.uid
        //&& self.upload_date == other.upload_date
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Urls {
    pub original: String,
    pub regular: String,
    pub small: String,
    pub thumb: String,
    pub mini: String,
}
