extern crate clap;

use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("Dynamic Dns Keeper")
        .version("0.0.1")
        .author("Tom Wu <luvnana618@gmail.com>")
        .about("DDNS tool that supports multiple sources")
        .subcommand(SubCommand::with_name("Cloudflare")
            .about("https://www.cloudflare.com/"))
        .get_matches();
}
