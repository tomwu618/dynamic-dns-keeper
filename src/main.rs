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

const DO_UPDATE_CMD_IP_LIST: Vec<String> = Vec::new();

fn main() {
    let menu = build_menu();
    let toml_config = config::read_config(menu);

    println!("waiting for network...");
    thread::sleep(Duration::from_secs(toml_config.global.post_up_wait));
    println!("start");
    cmd::run_array(toml_config.global.post_up_cmd.as_str());

    toml_config.record.iter().for_each(|r| {
        let record = r.clone();
        thread::spawn(move || {
            loop {
                let fun = function::create(&record);

                let ip_addr_result = cmd::run(record.get("ip_address_from").unwrap().as_str().unwrap());

                if !ip_addr_result.is_err() {
                    let ip_addr = ip_addr_result.unwrap();
                    let ip = IpAddr::from_str(&*ip_addr);
                    if !ip.is_err() {
                        let updated = fun.update(ip.unwrap());
                        if updated {
                            if !DO_UPDATE_CMD_IP_LIST.contains(&ip_addr) {
                                cmd::run_array(record.get("on_update_cmd").unwrap().as_str().unwrap());
                                DO_UPDATE_CMD_IP_LIST.push(ip_addr);
                            }

                            println!("updated");
                        }
                    }
                }
                thread::sleep(Duration::from_secs(60));
            }
        });
    });

    loop {
        thread::sleep(Duration::from_secs(18446744073709551615));
    }
}
