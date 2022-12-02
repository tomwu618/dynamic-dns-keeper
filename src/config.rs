use std::env;
use std::env::VarError;
use std::fs::File;
use std::io::Read;
use clap::ArgMatches;

use toml::value::Array;

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Config {
    pub global: Global,
    pub record: Array,
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Global {
    pub api_version: String,
    pub post_up_wait: u64,
    pub post_up_cmd: String,
}

pub(crate) fn read_config(menu: ArgMatches) -> Config {
    let mut config_file_path = String::new();

    match env::var("DDK_CONFIG") {
        Ok(val) => {
            config_file_path = val;
        }
        Err(VarError::NotPresent) => {
            config_file_path = menu.value_of("config").unwrap().to_string();
        }
        _ => {}
    }

    println!("configFilePath: {}", config_file_path);

    let mut config_file = match File::open(config_file_path) {
        Ok(f) => f,
        Err(e) => panic!("{}", e)
    };

    let mut config_str = String::new();
    match config_file.read_to_string(&mut config_str) {
        Ok(s) => s,
        Err(e) => panic!("Error Reading file: {}", e)
    };

    let config: Config = toml::from_str(&config_str).unwrap();
    config
}
