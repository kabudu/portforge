use rust_embed::Embed;

#[derive(Embed)]
#[folder = "src/web/static/"]
pub struct StaticAssets;
