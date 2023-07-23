use std::net::Ipv4Addr;
use std::str::FromStr;

use ping::ping;

fn main() {
    let dst_addr = Ipv4Addr::from_str("127.0.0.1").unwrap();
    ping(dst_addr);
}
