extern crate clap;
extern crate core;

#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::borrow::Borrow;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

mod d2k_core;
mod config;
mod menu;

use d2k_core::{Cloudflare, Config, Function, Record};

use log::info;
use toml::Value;
use toml::value::{Array, Table};

use crate::menu::build_menu;
use std::process::Command;

fn cmd(command: &str) -> Result<String, bool> {
    info!("Run Command : {}",command);

    let split_command = command.split(" ").collect::<Vec<&str>>();

    let mut cmd = Command::new(split_command[0]);
    for i in 1..split_command.len() {
        cmd.arg(split_command[i]);
    }

    let output = cmd.output().expect("failed to execute process");
    let output_str = String::from_utf8_lossy(&output.stdout).to_string();
    println!("{}", output_str);

    if output.status.success() {
        Ok(output_str)
    } else {
        Err(false)
    }
}

fn cmds(command: &str) {
    let split_command_row = command.split(";").collect::<Vec<&str>>();
    split_command_row.iter().for_each(|command| {
        cmd(command);
    });
}

fn start_v4_thread(toml_config: &Config) -> JoinHandle<()> {
    if false == toml_config.global.ipv4.enabled {
        return thread::spawn(|| {});
    }

    let get_ip_cmd = toml_config.global.ipv4.get_ip_cmd.clone();
    let mut tables: Vec<Table> = Vec::new();

    toml_config.cloudflare.to_vec().iter().for_each(|v| {
        let table = v.as_table().unwrap();

        if table["recordType"].as_str().unwrap() == "A" {
            tables.push(table.clone());
        }
    });

    thread::spawn(move || {
        loop {
            let ip_addr_result = cmd(get_ip_cmd.as_str());
            if !ip_addr_result.is_err() {
                let ip_addr = ip_addr_result.unwrap();

                tables.iter().for_each(|t| {
                    let Ipv4Addr = Ipv4Addr::from_str(&*ip_addr);
                    if !Ipv4Addr.is_err() {
                        let record = Record::A(Ipv4Addr::from_str(&*ip_addr).unwrap());

                        let cloudflare = Cloudflare {
                            email: t["Email"].as_str().unwrap().to_string(),
                            key: t["ApiKey"].as_str().unwrap().to_string(),
                            zones: t["zoneID"].as_str().unwrap().to_string(),
                            type_: t["recordType"].as_str().unwrap().to_string(),
                            name: t["recordName"].as_str().unwrap().to_string(),
                            domain: t["domain"].as_str().unwrap().to_string(),
                            recordType: t["recordType"].as_str().unwrap().to_string(),
                            ttl: t["recordTTL"].as_integer().unwrap().to_string(),
                            proxied: t["recordProxied"].as_bool().unwrap().to_string(),
                        };

                        cloudflare.update(record);
                    }



                });
            }

            thread::sleep(Duration::from_secs(60));
        }
    })
}

fn start_v6_thread(toml_config: &Config) -> JoinHandle<()> {
    if false == toml_config.global.ipv6.enabled {
        return thread::spawn(|| {});
    }

    let get_ip_cmd = toml_config.global.ipv6.get_ip_cmd.clone();
    let mut tables: Vec<Table> = Vec::new();

    toml_config.cloudflare.to_vec().iter().for_each(|v| {
        let table = v.as_table().unwrap();

        if table["recordType"].as_str().unwrap() == "AAAA" {
            tables.push(table.clone());
        }
    });

    thread::spawn(move || {
        loop {
            let ip_addr_result = cmd(get_ip_cmd.as_str());
            if !ip_addr_result.is_err() {
                let ip_addr = ip_addr_result.unwrap();

                tables.iter().for_each(|t| {
                    let Ipv6Addr = Ipv6Addr::from_str(&*ip_addr);
                    if !Ipv6Addr.is_err() {
                        let record = Record::AAAA(Ipv6Addr.unwrap());

                        let cloudflare = Cloudflare {
                            email: t["Email"].as_str().unwrap().to_string(),
                            key: t["ApiKey"].as_str().unwrap().to_string(),
                            zones: t["zoneID"].as_str().unwrap().to_string(),
                            type_: t["recordType"].as_str().unwrap().to_string(),
                            name: t["recordName"].as_str().unwrap().to_string(),
                            domain: t["domain"].as_str().unwrap().to_string(),
                            recordType: t["recordType"].as_str().unwrap().to_string(),
                            ttl: t["recordTTL"].as_integer().unwrap().to_string(),
                            proxied: t["recordProxied"].as_bool().unwrap().to_string(),
                        };

                        cloudflare.update(record);
                    }
                });

                thread::sleep(Duration::from_secs(60));
            }
        }
    })
}

fn main() {
    let menu = build_menu();
    let toml_config = config::read_config(menu);
    cmds(toml_config.global.post_up.as_str());

    let v4_thread = start_v4_thread(&toml_config);
    let v6_thread = start_v6_thread(&toml_config);

    v4_thread.join().unwrap();
    v6_thread.join().unwrap();
}
