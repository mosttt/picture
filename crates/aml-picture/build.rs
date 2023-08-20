fn main() {
    #[cfg(target_os = "windows")]
    embed_resource::compile("app-name-manifest.rc", embed_resource::NONE);
}
