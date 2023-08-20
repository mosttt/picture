pub mod anime;
pub mod leg;
pub mod photo;
pub mod pixiv;

#[inline]
pub fn init_log() {
    use time::{format_description, UtcOffset};
    use tracing_subscriber::fmt::time::OffsetTime;
    let format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]";
    let timer = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap(),
        format_description::parse(format).unwrap(),
    );
    tracing_subscriber::fmt()
        .with_max_level({
            if cfg!(debug_assertions) {
                tracing::Level::INFO
            } else {
                tracing::Level::INFO
            }
        })
        .with_timer(timer)
        .init();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
