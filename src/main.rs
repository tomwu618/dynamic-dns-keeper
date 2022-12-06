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
mod worker;

use crate::menu::build_menu;
use std::env;
use std::sync::{Arc, RwLock};
use crate::worker::start_worker;

const DO_UPDATE_CMD_IP_LIST: Vec<String> = Vec::new();

fn main() {
    let menu = build_menu();

    let toml_config = config::read_config(menu);

    init(&toml_config);

    start_worker(&toml_config);

    loop {
        thread::sleep(Duration::from_secs(18446744073709551615));
    }
}
