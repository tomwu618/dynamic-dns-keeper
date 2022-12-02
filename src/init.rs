use std::thread;
use std::time::Duration;
use crate::cmd;
use crate::config::Config;

pub fn init(toml_config: &Config) {
    println!("waiting for network...");
    thread::sleep(Duration::from_secs(toml_config.global.post_up_wait));
    println!("start");
    cmd::run_array(toml_config.global.post_up_cmd.as_str());
}
