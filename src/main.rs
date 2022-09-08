extern crate clap;

use std::{thread, time};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use clap::{Arg, App, SubCommand};

mod d2k_core;

use d2k_core::{Cloudflare, Function, Record};

use log::{info};


fn main() {
    env_logger::init();

    let matches = App::new("Dynamic Dns Keeper")
        .version("0.0.1")
        .author("Tom Wu <luvnana618@gmail.com>")
        .about("An advanced DDNS tool for WEB3.")
        .subcommand(SubCommand::with_name("start")
            .about("Start the DDNS service.")
            .subcommand(SubCommand::with_name("cloudflare")
                .about("https://www.cloudflare.com/")
                .arg(Arg::with_name("email")
                    .short('m')
                    .long("email")
                    .value_name("X-Auth-Email")
                    .help("Email address associated with your account")
                    .takes_value(true))
                .arg(Arg::with_name("key")
                    .short('k')
                    .long("key")
                    .value_name("X-Auth-Key")
                    .help("API key generated on the \"My Account\" page")
                    .takes_value(true))
                .arg(Arg::with_name("type")
                    .short('t')
                    .long("type")
                    .value_name("TYPE")
                    .help("DNS record type")
                    .takes_value(true)
                    .default_value("A"))
                .arg(Arg::with_name("name")
                    .short('n')
                    .long("name")
                    .value_name("NAME")
                    .help("DNS record name (or @ for the zone apex)")
                    .takes_value(true))
                .arg(Arg::with_name("ttl")
                    .short('l')
                    .long("ttl")
                    .value_name("TTL")
                    .help("Time to live, in seconds, of the DNS record. Must be between 60 and 86400, or 1 for 'automatic'")
                    .default_value("1")
                    .takes_value(true))
                .arg(Arg::with_name("proxied")
                    .short('p')
                    .long("proxied")
                    .value_name("PROXIED")
                    .help("Whether the record is receiving the performance and security benefits of Cloudflare")
                    .default_value("false")
                    .takes_value(true))
                .arg(Arg::with_name("zones")
                    .short('z')
                    .long("zones")
                    .value_name("ZONE ID")
                    .help("Specify the zone where the domain name to be modified")
                    .default_value("false")
                    .takes_value(true))
                .arg(Arg::with_name("domain")
                    .short('d')
                    .long("domain")
                    .value_name("Domain Name")
                    .help("Specify the domain name to be modified")
                    .default_value("false")
                    .takes_value(true)))
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("start") {
        if let Some(matches) = matches.subcommand_matches("cloudflare") {
            let cloudflare = Cloudflare::new(matches);

            loop {
                match get_v4_addr() {
                    Ok(ip) => {
                        let record = Record::A(ip);
                        cloudflare.update(record);
                    }
                    Err(e) => {
                        info!("Error: {}", e);
                    }
                }

                thread::sleep(time::Duration::from_secs(60));
            }
        }
    }
}

fn get_v4_addr() -> Result<Ipv4Addr, reqwest::Error> {
    let client = reqwest::blocking::Client::builder().local_address(IpAddr::from_str("0.0.0.0").unwrap()).build().unwrap();

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
