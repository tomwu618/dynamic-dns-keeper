use std::borrow::Borrow;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use crate::config::Config;
use crate::{cmd, function};
use std::env;

pub(crate) fn start_worker(toml_config: &Config) {
    toml_config.record.iter().for_each(|r| {
        let record = r.clone();

        thread::spawn(move || {
            let mut updated_ip_list: Vec<&str> = Vec::new();

            loop {
                let fun = function::create(&record);
                let ip_addr_result = cmd::run(record.get("ip_address_from_cmd").unwrap().as_str().unwrap());

                if !ip_addr_result.is_err() {
                    let ip_addr = ip_addr_result.unwrap();
                    let ip = IpAddr::from_str(&*ip_addr);
                    if !ip.is_err() {
                        let updated = fun.update(ip.unwrap());
                        if updated {
                            println!("updated");
                        }

                        let updated_ip_list = env::var("ddk_update_ip_list").unwrap_or("".to_string());
                        if !updated_ip_list.contains(&ip_addr) {
                            let on_update_cmd = record.get("ip_address_on_update_cmd");

                            if on_update_cmd.is_some() {
                                let cmd_str = on_update_cmd.unwrap().as_str().unwrap().replace("${IP_ADDRESS}", &*ip_addr);
                                cmd::run_array(cmd_str.to_string().as_str());
                            }

                            env::set_var("ddk_update_ip_list", format!("{};{}", updated_ip_list, ip_addr));
                        }
                    }
                }
                thread::sleep(Duration::from_secs(60));
            }
        });
    });
}
