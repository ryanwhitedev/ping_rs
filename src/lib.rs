use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::statistics::Statistics;

pub mod config;
pub mod icmp;
pub mod ip;
pub mod socket;
pub mod util;
pub mod statistics;

pub fn ping(config: Config) {
    util::register_signal_handlers();

    let dst_addr = match util::resolve_hostname(&config.destination) {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("Unable to resolve hostname: {}", &config.destination);
            std::process::exit(1);
        }
    };

    let pid = std::process::id() as u16;
    let mut seq = 1;
    let payload = "hello world!".as_bytes();

    let (tx, rx) = mpsc::channel();

    let mut stats = Statistics::new(&config.destination);

    println!(
        "PING {} ({}) {}({}) bytes of data.",
        &config.destination,
        dst_addr,
        payload.len() + icmp::ICMP_HDR_LEN,
        payload.len() + icmp::ICMP_HDR_LEN + ip::IPV4_HDR_LEN
    );

    thread::spawn(move || {
        loop {
            let request = icmp::Request::new(dst_addr, pid, seq, payload.to_vec(), config.timeout);

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
        if let Ok(response) = rx.try_recv() {
            stats.update(response);
        }

        // Handle SIGINT: print statistics and exit
        if unsafe { util::SIGNAL_CTRL_C } {
            stats.print();
            std::process::exit(0);
        }

        // Add small delay to prevent 100% CPU usage
        let delay = Duration::from_millis(50);
        thread::sleep(delay);
    }
}
