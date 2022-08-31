use std::collections::HashMap;
use clap::{App, Arg, ArgMatches, SubCommand};
use super::Function;

struct Context {
    email: String,
    key: String,
    zones: String,
    type_: String,
    name: String,
    ttl: String,
    proxied: String,
}

pub struct Cloudflare {
    context: Context,
}

impl Function for Cloudflare {
    fn buildCliCommand<'help>() -> App<'help> {
        clap::SubCommand::with_name("cloudflare")
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
            .arg(Arg::with_name("content")
                .short('c')
                .long("content")
                .value_name("CONTENT")
                .help("DNS record content")
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
                .takes_value(true))
    }

    fn select(_: ArgMatches) -> &Function {
        let context = Context {
            email: matches.value_of("email").unwrap(),
            key: matches.value_of("key").unwrap(),
            zones: matches.value_of("zones").unwrap(),
            type_: matches.value_of("type").unwrap(),
            name: matches.value_of("name").unwrap(),
            ttl: matches.value_of("ttl").unwrap(),
            proxied: matches.value_of("proxied").unwrap(),
        };

        return &Cloudflare { context };
    }


    fn query() -> String {
        todo!()
    }

    fn update(ip: String) {
        todo!()
    }
}
