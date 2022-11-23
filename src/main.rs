extern crate clap;
extern crate core;

#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

mod d2k_core;
mod config;
mod menu;

use d2k_core::{Cloudflare, Config, Function, Record};

use log::info;
use crate::menu::build_menu;

fn main() {
    let menu = build_menu();
    let toml_config = config::read_config(menu);

    println!("{:?}", toml_config);

}

fn get_v4_addr(bind_ip: String) -> Result<Ipv4Addr, reqwest::Error> {
    let client = reqwest::blocking::Client::builder().local_address(IpAddr::from_str(bind_ip.as_str()).unwrap()).build().unwrap();

    let my_ip = match client.get("https://ip.yan-yun.com")
        .send() {
        Ok(res) => {
            let body = res.text().unwrap();
            Ipv4Addr::from_str(&body).unwrap()
        }
        Err(err) => {
            return Err(err);
        }
    };

    info!("Current Ip : {}",my_ip.to_string());
    Ok(my_ip)
}
