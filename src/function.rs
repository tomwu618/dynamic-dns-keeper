use std::net::IpAddr;
use toml::Value;
use crate::cloudflare;

pub trait Function {
    fn update(&self, record: IpAddr);
}

pub fn create(c: &Value) -> Box<dyn Function> {
    let domain_registrar = c.get("domain_registrar").unwrap().as_str().unwrap();

    let api_param_table = c.get("api_param").unwrap().as_table().unwrap();

    match domain_registrar {
        "cloudflare" => {
            let api_key = api_param_table["api_key"].as_str().unwrap().to_string();
            let email = api_param_table["email"].as_str().unwrap().to_string();
            let zone_id = api_param_table["zone_id"].as_str().unwrap().to_string();
            let type_ = api_param_table["record_type"].as_str().unwrap().to_string();
            let name = api_param_table["record_name"].as_str().unwrap().to_string();
            let domain = api_param_table["domain"].as_str().unwrap().to_string();
            let ttl = api_param_table["record_ttl"].as_integer().unwrap().to_string();
            let proxied = api_param_table["record_proxied"].as_bool().unwrap().to_string();

            let cloudflare = cloudflare::Cloudflare::new(api_key, email, zone_id, type_, name, domain, ttl, proxied);
            Box::new(cloudflare)
        }
        _ => panic!("Unknown function type: {}", domain_registrar),
    }
}
