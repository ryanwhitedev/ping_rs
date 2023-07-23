use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::thread;

use crate::icmp::{Request, Response};

pub mod icmp;
pub mod ip;
pub mod socket;
pub mod util;

pub fn ping(dst_addr: Ipv4Addr) {
    util::register_signal_handlers();

    let pid = std::process::id() as u16;
    let mut seq = 1;
    let payload = "hello world!".as_bytes();

    let (tx, rx) = mpsc::channel();
    let mut results = Vec::new();

    // Start timer
    let now = Instant::now();

    println!(
        "PING {} ({}) {}({}) bytes of data.",
        dst_addr,
        dst_addr,
        payload.len() + icmp::ICMP_HDR_LEN,
        payload.len() + icmp::ICMP_HDR_LEN + ip::IPV4_HDR_LEN
    );

    thread::spawn(move || {
        loop {
            let request = Request::new(dst_addr, pid, seq, payload.to_vec());

            let reply = match request.send() {
                Ok(reply) => match reply {
                    Response::EchoReply { ttl, rtt, ref data } => {
                        println!(
                            "{} bytes from {}: icmp_seq={} ttl={} time={:.2} ms",
                            data.len(),
                            dst_addr,
                            seq,
                            ttl,
                            rtt,
                        );
                        reply
                    },
                    Response::Dropped => {
                        println!("Packet Dropped");
                        reply
                    },
                    Response::HostUnreachable => {
                        println!(
                            "From {} ({}) icmp_seq={} Destination Host Unreachable",
                            dst_addr,
                            dst_addr,
                            seq
                        );
                        reply
                    },
                    _ => reply,
                },
                Err(error) => {
                    println!("{}", error);
                    std::process::exit(1);
                }
            };

            tx.send(reply).expect("failed sending to channel");

            seq += 1;

            // Sleep 1s before sending next packet
            let delay = Duration::from_secs(1);
            thread::sleep(delay);
        }
    });

    loop {
        // Check if we have received any messages from the thread
        if let Ok(received) = rx.try_recv() {
            results.push(received);
        }

        // Handle SIGINT: print statistics and exit
        if unsafe { util::SIGNAL_CTRL_C } {
            let duration = now.elapsed().as_micros() as f32 / 1000f32;
            util::statistics(dst_addr, duration, results);
            std::process::exit(1);
        }

        // Add small delay to prevent 100% CPU usage
        let delay = Duration::from_millis(50);
        thread::sleep(delay);
    }
}
