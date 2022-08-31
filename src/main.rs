extern crate clap;

use std::{thread, time};
use std::collections::HashMap;
use clap::{Arg, App, SubCommand};
use reqwest::{Client, Error, Response};
use reqwest::header::HeaderMap;
use serde_json::Value;

mod d2k_core;

use d2k_core::{Cloudflare, Function};


#[tokio::main]
async fn main() {
    let matches = App::new("Dynamic Dns Keeper")
        .version("0.0.1")
        .author("Tom Wu <luvnana618@gmail.com>")
        .about("An advanced DDNS tool for WEB3.")
        .subcommand(SubCommand::with_name("start")
            .about("Start the DDNS service.")
            .subcommand(Cloudflare::buildCliCommand())
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("start") {
        if let Some(matches) = matches.subcommand_matches("cloudflare") {
            println!("Start the DDNS service with keeper.cloudflare.");
            println!("Email: {}", matches.value_of("email").unwrap());
            println!("Key: {}", matches.value_of("key").unwrap());
            println!("Type: {}", matches.value_of("type").unwrap());
            println!("Name: {}", matches.value_of("name").unwrap());
            println!("Content: {}", matches.value_of("content").unwrap());
            println!("TTL: {}", matches.value_of("ttl").unwrap());
            println!("Proxied: {}", matches.value_of("proxied").unwrap());
            println!("Zone: {}", matches.value_of("zones").unwrap());
            println!("Domain: {}", matches.value_of("domain").unwrap());

            let mail = matches.value_of("email").unwrap();
            let key = matches.value_of("key").unwrap();
            let type_ = matches.value_of("type").unwrap();
            let name = matches.value_of("name").unwrap();
            let content = matches.value_of("content").unwrap();
            let ttl = matches.value_of("ttl").unwrap();
            let proxied = matches.value_of("proxied").unwrap();
            let zone = matches.value_of("zones").unwrap();
            let domain = matches.value_of("domain").unwrap();

            let client = reqwest::Client::new();

            loop {
                let my_ip = get_my_ip(&client).await.unwrap();
                println!("My Ip: {}", my_ip);

                let dns_record_result = get_dns_record(&client, mail, key, zone, type_, name).await.unwrap();

                if dns_record_result != my_ip {
                    println!("Update DNS record.");
                    // update_dns_record(mail, key, zone, type_, name, my_ip, ttl, proxied, domain).await.unwrap();
                } else {
                    println!("DNS record is up to date.");
                }

                thread::sleep(time::Duration::from_secs(60));
            }
        }
    }
}

async fn get_my_ip(client: &Client) -> Result<String, reqwest::Error> {
    let my_ip = client.get("https://ip.yan-yun.com")
        .send()
        .await?
        .text()
        .await?;

    Ok(my_ip)
}


async fn get_dns_record(client: &Client, email: &str, key: &str, zones: &str, record_type: &str, name: &str) -> Result<String, Error> {
    let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records?type={}&name={}&order=type&direction=desc&match=all", zones, record_type, name);

    let http_response = client.get(url)
        .header("X-Auth-Email", email)
        .header("X-Auth-Key", key)
        .header("Content-type", "application/json")
        .send()
        .await?;

    let response = http_response.text().await?;
    println!("Response: {}", &response);

    let json: Value = serde_json::from_str(&response).unwrap();

    if json["success"].as_bool().unwrap() && json["result_info"]["count"].as_u64().unwrap() == 1 {
        let result = json["result"].as_array().unwrap();

        let id = result[0]["id"].as_str().unwrap();
        let content = result[0]["content"].as_str().unwrap();

        Ok(content.to_string())
    } else {
        Ok("OH NO!".to_string())
    }

}
