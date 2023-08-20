use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDateTime, TimeZone, Timelike};

pub mod entity;

pub fn get_str_time(timestamp: i64) -> String {
    let datetime: DateTime<Local> = Local.timestamp_millis_opt(timestamp).unwrap();
    let current_time = datetime.with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
    //datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    format!(
        "{:04}/{:02}/{:02}/{:02}/{:02}/{:02}",
        current_time.year(),
        current_time.month(),
        current_time.day(),
        current_time.hour(),
        current_time.minute(),
        current_time.second(),
    )
}

pub fn get_timestamp_millis(time_str: impl AsRef<str>) -> i64 {
    let format_str = "%Y/%m/%d/%H/%M/%S"; // 时间字符串的格式
    let offset = FixedOffset::east_opt(9 * 3600).unwrap();
    // 解析时间字符串
    let naive_datetime = NaiveDateTime::parse_from_str(time_str.as_ref(), format_str).unwrap();

    // 使用固定偏移量构建日期时间对象
    let datetime = DateTime::<FixedOffset>::from_local(naive_datetime, offset);

    datetime.timestamp_millis()
}

pub fn get_str_time_in_original_url(original_url: impl AsRef<str>, pid: u64) -> String {
    let url = original_url.as_ref();
    let f_first = "img-original/img/";
    let first = url.find(f_first).unwrap() + f_first.len();
    let end = url.find(pid.to_string().as_str()).unwrap() - 1;
    url.get(first..end).unwrap().to_string()
}

pub fn get_ext_in_original_url(original_url: impl AsRef<str>) -> String {
    let ext = original_url.as_ref().split('.').last().unwrap();
    ext.to_string()
}

pub fn generate_original_url(upload_date: i64, pid: u64, p: u64, ext: &str) -> String {
    //https://i.pixiv.re/img-original/img/2022/07/13/12/07/27/99693065_p0.jpg
    format!(
        "https://i.pixiv.re/img-original/img/{}/{}_p{}.{}",
        get_str_time(upload_date),
        pid,
        p,
        ext
    )
}

pub fn generate_regular_url(upload_date: i64, pid: u64, p: u64) -> String {
    //https://i.pixiv.re/img-master/img/2022/07/13/12/07/27/99693065_p0_master1200.jpg
    format!(
        "https://i.pixiv.re/img-master/img/{}/{}_p{}_master1200.jpg",
        get_str_time(upload_date),
        pid,
        p
    )
}

pub fn generate_small_url(upload_date: i64, pid: u64, p: u64) -> String {
    //https://i.pixiv.re/c/540x540_70/img-master/img/2022/07/13/12/07/27/99693065_p0_master1200.jpg
    format!(
        "https://i.pixiv.re/c/540x540_70/img-master/img/{}/{}_p{}_master1200.jpg",
        get_str_time(upload_date),
        pid,
        p
    )
}

pub fn generate_thumb_url(upload_date: i64, pid: u64, p: u64) -> String {
    //https://i.pixiv.re/c/250x250_80_a2/img-master/img/2022/07/13/12/07/27/99693065_p0_square1200.jpg
    format!(
        "https://i.pixiv.re/c/250x250_80_a2/img-master/img/{}/{}_p{}_square1200.jpg",
        get_str_time(upload_date),
        pid,
        p
    )
}

pub fn generate_mini_url(upload_date: i64, pid: u64, p: u64) -> String {
    //https://i.pixiv.re/c/48x48/img-master/img/2022/07/13/12/07/27/99693065_p0_square1200.jpg
    format!(
        "https://i.pixiv.re/c/48x48/img-master/img/{}/{}_p{}_square1200.jpg",
        get_str_time(upload_date),
        pid,
        p
    )
}
