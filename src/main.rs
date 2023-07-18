use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;

use ping::{icmp, ipv4};

fn main() {
    let dst_addr = Ipv4Addr::from_str("8.8.8.8").unwrap();

    let pid = std::process::id() as u16;
    let mut seq = 1;
    let payload = "hello world!".as_bytes();

    println!(
        "PING {} ({}) {}({}) bytes of data.",
        dst_addr,
        dst_addr,
        payload.len() + icmp::ICMP_HDR_LEN,
        payload.len() + icmp::ICMP_HDR_LEN + ipv4::IP_HDR_LEN
    );

    loop {
        let request = icmp::Request::new(dst_addr, pid, seq, payload.to_vec());

        let reply = match request.send() {
            Ok(reply) => reply,
            Err(error) => {
                println!("{}", error);
                std::process::exit(1);
            }
        };

        seq = reply.seq + 1;

        // Sleep 1s before sending next packet
        let delay = Duration::from_secs(1);
        std::thread::sleep(delay);
    }
}
