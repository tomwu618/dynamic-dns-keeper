use clap::{ArgMatches};
use serde_json::{json, Value};
use crate::Record::A;
use super::Record;
use super::Function;

use log::{info};
use crate::d2k_core::Record::AAAA;

pub struct Cloudflare {
    pub(crate) email: String,
    pub(crate) key: String,
    pub(crate) zones: String,
    pub(crate) type_: String,
    pub(crate) name: String,
    pub(crate) domain: String,
    pub(crate) ttl: String,
    pub(crate) proxied: String,
    pub(crate) recordType: String,
}

impl Function for Cloudflare {
    fn update(&self, record: Record) {
        let client = reqwest::blocking::Client::new();

        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records?type={}&name={}.{}&order=type&direction=desc&match=all",
                          self.zones, self.type_, self.name, self.domain);

        let http_response = match client.get(url)
            .header("X-Auth-Email", &self.email)
            .header("X-Auth-Key", &self.key)
            .header("Content-type", "application/json")
            .send() {
            Ok(response) => response,
            Err(e) => {
                info!("Error: {}", e);
                return;
            }
        };

        let response = http_response.text().unwrap();

        let json: Value = serde_json::from_str(&response).unwrap();

        if json["success"].as_bool().unwrap() && json["result_info"]["count"].as_u64().unwrap() == 1 {
            let result = json["result"].as_array().unwrap();

            let id = result[0]["id"].as_str().unwrap();
            let content_ip = result[0]["content"].as_str().unwrap();

            info!("Content Ip : {}",content_ip);

            let current_ip = match record {
                A(ip) => ip.to_string(),
                AAAA(ip) => ip.to_string(),
                _ => panic!("Not supported record type"),
            };

            if content_ip != current_ip {
                let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", self.zones, id);

                let http_response = match client.put(url)
                    .header("X-Auth-Email", &self.email)
                    .header("X-Auth-Key", &self.key)
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
                        return;
                    }
                };

                let response = http_response.text().unwrap();

                info!("Response: {}", &response);
            }
        }
    }
}
