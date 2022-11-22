use clap::{App, Arg, ArgMatches};

pub(crate) fn build_menu() -> ArgMatches {
    return App::new("Dynamic Dns Keeper")
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
}
