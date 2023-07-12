use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::{Duration, Instant};

use ping::socket::IcmpSocket;
use ping::{icmp, ipv4};

const IP_HDR_LEN: usize = 20;

fn main() {
    let dst_addr = Ipv4Addr::from_str("8.8.8.8").unwrap();

    let mut request = icmp::Request::new(49995, 1, vec![9, 0, 2, 1, 0]);
    let request = request.pack();

    println!(
        "PING {} ({}) {}({}) bytes of data.",
        dst_addr,
        dst_addr,
        request.len(),
        request.len() + IP_HDR_LEN
    );

    loop {
        let sock = IcmpSocket::new().expect("failed creating icmp socket");

        let now = Instant::now();

        let _sent_bytes = sock.sendto(&request, dst_addr).unwrap();

        let mut buf: [u8; 128] = [0; 128];
        let recv_bytes = sock.recvfrom(&mut buf).unwrap();

        // round trip time in ms
        let rtt = now.elapsed().as_micros() as f32 / 1000f32;

        if let Ok(ip_hdr) = ipv4::HdrIpv4::try_from(&buf[0..IP_HDR_LEN]) {
            match icmp::Reply::try_from(&buf[IP_HDR_LEN..recv_bytes as usize]) {
                Ok(reply) => println!(
                    "{} bytes from {}: icmp_seq={} ttl={} time={:.2} ms",
                    recv_bytes - IP_HDR_LEN,
                    ip_hdr.src_addr,
                    reply.seq,
                    ip_hdr.ttl,
                    rtt
                ),
                Err(_) => println!("something went wrong"),
            }
        }

        // Sleep 1s before sending next packet
        let delay = Duration::from_secs(1);
        std::thread::sleep(delay);
    }
}
