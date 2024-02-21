use anyhow::Result;
use rust_embed::RustEmbed;
use salvo::prelude::*;
use salvo::serve_static::static_embed;

use aml_picture::handle::anime::anime;
use aml_picture::handle::leg::leg;
use aml_picture::handle::photo::photo;
use aml_picture::handle::pixiv::{pixiv, pixiv_only_local};

// Use Jemalloc only for musl-64 bits platforms
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(RustEmbed)]
#[folder = "../../react-app/build"] //相对位置是按当前crate的*Cargo.toml*所在位置来算的，不是按执行命令的地方！
struct Assets;

#[tokio::main]
async fn main() -> Result<()> {
    picture_core::init_log();

    let router = Router::new()
        //放在前面的优先级高，不然"<**path>"会全部被匹配到
        .push(
            Router::new()
                .path("api")
                .push(Router::new().path("pixiv/v1").get(pixiv))
                .push(Router::new().path("anime/v1").get(anime))
                .push(Router::new().path("photo/v1").get(photo))
                .push(Router::new().path("leg/v1").get(leg))
                .push(Router::new().path("pixiv/v2").get(pixiv_only_local)),
        )
        //不会嵌入到二进制文件中，需要单独放置（相对路径是按执行命令的地方来算的）
        //找不到文件就是404，请求默认路径（/）就是.defaults("index.html")
        // .push(
        //     Router::new().path("<**path>").get(
        //         StaticDir::new([env::current_exe().unwrap().parent().unwrap().join("static")])
        //             .defaults("index.html")
        //             .listing(true),
        //     ),
        // );
        //会嵌入到二进制文件中，不需要单独放置（相对位置是按当前crate的*Cargo.toml*所在位置来算的）
        //找不到文件就用.fallback("index.html")，所以默认路径就是找不到的路径，就是.fallback("index.html")
        .push(Router::with_path("<**path>").get(static_embed::<Assets>().fallback("index.html")));
    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    Server::new(acceptor).serve(router).await;
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
