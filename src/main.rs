use std::net::Ipv4Addr;
use std::str::FromStr;
use std::time::Duration;

use ping::{
    ipv4::IP_HDR_LEN, 
    icmp::{ICMP_HDR_LEN, Request, Reply}
};

static mut SIGNAL_CTRL_C: bool = false;

fn register_signal_handlers() {
    unsafe { 
        libc::signal(libc::SIGINT, handle_sigint as usize);
    }
}

// Handles SIGINT (ie. ctrl-c) by setting a global flag to indicate this signal has been
// received
fn handle_sigint(_signal: i32) {
    // re-register signal handlers
    register_signal_handlers();
    // set global flag to indicate interrupt signal received
    unsafe { 
        SIGNAL_CTRL_C = true;
    }
}

fn statistics(dst_addr: Ipv4Addr) {
    println!("\n--- {} ping statistics ---", dst_addr);
}

fn main() {
    register_signal_handlers();

    let dst_addr = Ipv4Addr::from_str("8.8.8.8").unwrap();

    let pid = std::process::id() as u16;
    let mut seq = 1;
    let payload = "hello world!".as_bytes();

    println!(
        "PING {} ({}) {}({}) bytes of data.",
        dst_addr,
        dst_addr,
        payload.len() + ICMP_HDR_LEN,
        payload.len() + ICMP_HDR_LEN + IP_HDR_LEN
    );

    loop {
        let request = Request::new(dst_addr, pid, seq, payload.to_vec());

        let _reply = match request.send() {
            Ok(reply) => match reply {
                Reply::Echo { ttl, rtt, ref data } => {
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
                Reply::Dropped => {
                    println!("Packet Dropped");
                    reply
                },
                Reply::HostUnreachable => {
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

        // Handle SIGINT: print statistics and exit
        if unsafe { SIGNAL_CTRL_C } {
            statistics(dst_addr);
            std::process::exit(1);
        }

        seq += 1;

        // Sleep 1s before sending next packet
        let delay = Duration::from_secs(1);
        std::thread::sleep(delay);
    }
}
