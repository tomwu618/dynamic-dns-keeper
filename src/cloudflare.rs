use std::net::IpAddr;
use serde_json::{json, Value};

use log::{info};
use crate::function::Function;

pub struct Cloudflare {
    api_key: String,
    email: String,
    zone_id: String,
    type_: String,
    name: String,
    domain: String,
    ttl: String,
    proxied: String,
}

impl Cloudflare {
    pub(crate) fn new(api_key: String, email: String, zone_id: String, type_: String, name: String, domain: String, ttl: String, proxied: String) -> Self {
        Cloudflare {
            api_key,
            email,
            zone_id,
            type_,
            name,
            domain,
            ttl,
            proxied,
        }
    }
}

impl Function for Cloudflare {
    fn update(&self, ip_addr: IpAddr) -> bool {
        let client = reqwest::blocking::Client::new();

        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records?type={}&name={}.{}&order=type&direction=desc&match=all",
                          self.zone_id, self.type_, self.name, self.domain);

        let http_response = match client.get(url)
            .header("X-Auth-Email", &self.email)
            .header("X-Auth-Key", &self.api_key)
            .header("Content-type", "application/json")
            .send() {
            Ok(response) => response,
            Err(e) => {
                info!("Error: {}", e);
                return false;
            }
        };

        let result_response = http_response.text();
        if result_response.is_err() {
            return false;
        }

        let response = result_response.unwrap();

        //Todo
        let json: Value = serde_json::from_str(&response).unwrap();

        if json["success"].as_bool().unwrap() && json["result_info"]["count"].as_u64().unwrap() == 1 {
            let result = json["result"].as_array().unwrap();

            let id = result[0]["id"].as_str().unwrap();
            let content_ip = result[0]["content"].as_str().unwrap();

            info!("Content Ip : {}",content_ip);

            let current_ip = ip_addr.to_string();

            if content_ip != current_ip {
                let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", self.zone_id, id);

                let http_response = match client.put(url)
                    .header("X-Auth-Email", &self.email)
                    .header("X-Auth-Key", &self.api_key)
                    .header("Content-type", "application/json")
                    .body(json!({
                        "type": self.type_,
                        "name": self.name,
                        "content":current_ip,
                        "ttl": self.ttl.parse::<u32>().unwrap(),
                        "proxied": self.proxied.parse::<bool>().unwrap(),
                    }).to_string())
                    .send() {
                    Ok(response) => response,
                    Err(e) => {
                        info!("Error: {}", e);
                        return false;
                    }
                };

                let response = http_response.text().unwrap();

                info!("Response: {}", &response);
                return true;
            } else { return false; }
        } else { return false; }
    }
}
