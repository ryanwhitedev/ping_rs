use std::net::Ipv4Addr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub mod icmp;
pub mod ip;
pub mod socket;
pub mod statistics;
pub mod util;

use statistics::Statistics;

pub fn ping(dst_addr: Ipv4Addr) {
    util::register_signal_handlers();

    let pid = std::process::id() as u16;
    let mut seq = 1;
    let payload = "hello world!".as_bytes();

    let (tx, rx) = mpsc::channel();

    let mut stats = Statistics::new(dst_addr);

    println!(
        "PING {} ({}) {}({}) bytes of data.",
        dst_addr,
        dst_addr,
        payload.len() + icmp::ICMP_HDR_LEN,
        payload.len() + icmp::ICMP_HDR_LEN + ip::IPV4_HDR_LEN
    );

    thread::spawn(move || {
        loop {
            let request = icmp::Request::new(dst_addr, pid, seq, payload.to_vec());

            let reply = match request.send() {
                Ok(reply) => reply,
                Err(error) => {
                    println!("Response error: {}", error);
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
        if let Ok(resp) = rx.try_recv() {
            stats.update(resp);
        }

        // Handle SIGINT: print statistics and exit
        if unsafe { util::SIGNAL_CTRL_C } {
            stats.print();
            std::process::exit(1);
        }

        // Add small delay to prevent 100% CPU usage
        let delay = Duration::from_millis(50);
        thread::sleep(delay);
    }
}
