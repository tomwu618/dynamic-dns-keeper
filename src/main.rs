extern crate clap;
extern crate core;

#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::{thread, time, env};
use std::env::VarError;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use clap::{Arg, App, SubCommand};

mod d2k_core;

use d2k_core::{Cloudflare, Function, Record, Config};

use log::{info};

fn main() {
    let file_path = "config.toml";
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => panic!("no such file {} exception:{}", file_path, e)
    };
    let mut str_val = String::new();
    match file.read_to_string(&mut str_val) {
        Ok(s) => s
        ,
        Err(e) => panic!("Error Reading file: {}", e)
    };

    let config: Config = toml::from_str(&str_val).unwrap();

    let menu = App::new("Dynamic Dns Keeper")
        .version("0.0.1")
        .author("Tom Wu <luvnana618@gmail.com>")
        .about("An advanced DDNS tool for WEB3.")
        .arg(Arg::with_name("config")
            .short("c".parse().unwrap())
            .long("config")
            .value_name("config")
            .help("select a config file")
            .default_value("/etc/ddk/config.toml")
            .takes_value(true))
        .get_matches();

    let mut config_file = String::new();

    match env::var("DDK_CONFIG") {
        Ok(val) => {
            println!("DDK_CONFIG: {}", val);
            config_file = val;
        }
        Err(VarError::NotPresent) => {
            config_file = menu.value_of("config").unwrap().to_string();
        }
        _ => {}
    }

    println!("config_file: {}", config_file);


    // println!("DDR_BIND_IP {}", bind_ip);
    //
    // env_logger::init();
    //
    //
    //
    //
    // println!("config_file {}", config_file);

    // if let Some(matches) = matches.subcommand_matches("start") {
    //     if let Some(matches) = matches.subcommand_matches("cloudflare") {
    //         let cloudflare = Cloudflare::new(matches);
    //
    //         loop {
    //             match get_v4_addr(bind_ip.clone()) {
    //                 Ok(ip) => {
    //                     let record = Record::A(ip);
    //                     cloudflare.update(record);
    //                 }
    //                 Err(e) => {
    //                     info!("Error: {}", e);
    //                 }
    //             }
    //
    //             thread::sleep(time::Duration::from_secs(60));
    //         }
    //     }
    // }
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
