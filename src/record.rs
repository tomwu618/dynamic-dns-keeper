use std::net::{Ipv4Addr, Ipv6Addr};

pub enum Record {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
}
