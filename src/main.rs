use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;

use ping::socket::IcmpSocket;

const PING: &[u8] = &[8, 0, 41, 178, 195, 75, 0, 1, 9, 0, 2, 1, 0];
const IP_HDR_LEN: usize = 20;

fn main() {
    let ipv4 = Ipv4Addr::from_str("127.0.0.1").unwrap();

    loop {
        let sock = IcmpSocket::new().expect("failed creating icmp socket");
        let _sent_bytes = sock.sendto(PING, ipv4).unwrap();

        let mut buf: [u8; 128] = [0; 128];
        let recv_bytes = sock.recvfrom(&mut buf).unwrap();

        let reply = &buf[IP_HDR_LEN..recv_bytes as usize];
        dbg!(reply);

        // Sleep 1s before sending next packet
        let delay = Duration::from_secs(1);
        std::thread::sleep(delay);
    }
}
