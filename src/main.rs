extern crate clap;
extern crate core;

#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::net::IpAddr;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use crate::init::init;

mod config;
mod menu;
pub mod record;
mod cmd;
mod function;
mod cloudflare;
mod init;

use crate::menu::build_menu;
use std::env;

const DO_UPDATE_CMD_IP_LIST: Vec<String> = Vec::new();

fn main() {
    let menu = build_menu();
    let toml_config = config::read_config(menu);
    init(&toml_config);

    toml_config.record.iter().for_each(|r| {
        let record = r.clone();
        thread::spawn(move || {
            loop {
                let fun = function::create(&record);
                let ip_addr_result = cmd::run(record.get("ip_address_from_cmd").unwrap().as_str().unwrap());

                if !ip_addr_result.is_err() {
                    let ip_addr = ip_addr_result.unwrap();
                    let ip = IpAddr::from_str(&*ip_addr);
                    if !ip.is_err() {
                        let updated = fun.update(ip.unwrap());
                        if updated {
                            let on_update_cmd = record.get("ip_address_on_update_cmd");
                            if on_update_cmd.is_some() && !DO_UPDATE_CMD_IP_LIST.contains(&ip_addr) {
                                let cmd_str= on_update_cmd.unwrap().as_str().unwrap().replace("${IP_ADDRESS}", &*ip_addr);
                                cmd::run_array(cmd_str.to_string().as_str());

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
