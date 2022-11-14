use toml::value::Array;

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Config {
    api_version: String,
    cloudflare: Array,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Cloudflare {
    api_key: String,
    email: String,
    zone_id: String,
    record_id: String,
    record_name: String,
    record_type: String,
    record_ttl: u32,
    record_content: Option<String>,
    record_priority: u32,
    record_proxied: bool,
}
