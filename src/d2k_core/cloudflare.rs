use std::collections::HashMap;
use std::error::Error;
use std::net::Ipv4Addr;
use std::pin::Pin;
use clap::{App, Arg, ArgMatches, SubCommand};
use clap::builder::TypedValueParser;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use crate::Record::A;
use super::Record;
use super::Function;

use log::Level::Info;
use log::{info, log_enabled};

struct Context {
    email: String,
    key: String,
    zones: String,
    type_: String,
    name: String,
    domain: String,
    ttl: String,
    proxied: String,

    meta_data: HashMap<String, String>,
}

pub struct Cloudflare {
    context: Context,
}

impl Cloudflare {
    pub(crate) fn new(matches: &ArgMatches) -> Self {
        let email = matches.value_of("email").unwrap().to_string();
        let key = matches.value_of("key").unwrap().to_string();
        let zones = matches.value_of("zones").unwrap().to_string();
        let type_ = matches.value_of("type").unwrap().to_string();
        let name = matches.value_of("name").unwrap().to_string();
        let domain = matches.value_of("domain").unwrap().to_string();
        let ttl = matches.value_of("ttl").unwrap().to_string();
        let proxied = matches.value_of("proxied").unwrap().to_string();

        Cloudflare {
            context: Context {
                email,
                key,
                zones,
                type_,
                name,
                domain,
                ttl,
                proxied,
                meta_data: Default::default(),
            },
        }
    }
}

enum PokerCard {
    Clubs(u8),
    Spades(u8),
    Diamonds(char),
    Hearts(char),
}

impl Function for Cloudflare {
    fn update(&self, record: Record) {
        let client = reqwest::blocking::Client::new();

        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records?type={}&name={}.{}&order=type&direction=desc&match=all",
                          self.context.zones, self.context.type_, self.context.name, self.context.domain);

        let http_response = client.get(url)
            .header("X-Auth-Email", &self.context.email)
            .header("X-Auth-Key", &self.context.key)
            .header("Content-type", "application/json")
            .send();

        let response = http_response.unwrap().text().unwrap();

        let json: Value = serde_json::from_str(&response).unwrap();

        if json["success"].as_bool().unwrap() && json["result_info"]["count"].as_u64().unwrap() == 1 {
            let result = json["result"].as_array().unwrap();

            let id = result[0]["id"].as_str().unwrap();
            let content_ip = result[0]["content"].as_str().unwrap();

            info!("Content Ip : {}",content_ip);

            let current_ip = match record {
                A(ip) => ip.to_string(),
                _ => panic!("Not supported record type"),
            };

            if content_ip != current_ip {
                let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}", self.context.zones, id);

                let http_response = client.put(url)
                    .header("X-Auth-Email", &self.context.email)
                    .header("X-Auth-Key", &self.context.key)
                    .header("Content-type", "application/json")
                    .body(json!({
                        "type": self.context.type_,
                        "name": self.context.name,
                        "content":current_ip,
                        "ttl": self.context.ttl.parse::<u32>().unwrap(),
                        "proxied": self.context.proxied.parse::<bool>().unwrap(),
                    }).to_string())
                    .send();

                let response = http_response.unwrap().text().unwrap();

                info!("Response: {}", &response);
            }
        }
    }
}
