extern crate clap;
extern crate core;

#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::net::IpAddr;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

mod config;
mod menu;
pub mod record;
mod cmd;
mod function;
mod cloudflare;

use crate::menu::build_menu;

fn main() {
    let menu = build_menu();
    let toml_config = config::read_config(menu);

    toml_config.record.iter().for_each(|c| {
       let fun = function::create(c);

        let ip_addr_result = cmd::run(c.get("ip_address_from").unwrap().as_str().unwrap());

        if !ip_addr_result.is_err() {
            let ip_addr = ip_addr_result.unwrap();
            let ip = IpAddr::from_str(&*ip_addr);
            if !ip.is_err() {
                fun.update(ip.unwrap());
            }
        }

        thread::sleep(Duration::from_secs(60));
    });


    loop{
        thread::sleep(Duration::from_secs(18446744073709551615));
    }
}
