use std::time::Instant;

use crate::icmp::Response;

pub struct Statistics<'a> {
    destination: &'a str,
    start: Instant,
    sent: u32,
    received: u32,
    errors: u32,
    dropped: u32,
    rtt: Vec<f32>,
}

impl<'a> Statistics<'a> {
    pub fn new(destination: &'a str) -> Statistics {
        Self {
            destination,
            start: Instant::now(), // Start timer
            sent: 0,
            received: 0,
            errors: 0,
            dropped: 0,
            rtt: Vec::new(),
        }
    }
    // Update statistics struct based on type of response received
    pub fn update(&mut self, response: Response) {
        self.sent += 1;
        match response {
            Response::EchoReply { rtt, .. } => {
                self.received += 1;
                self.rtt.push(rtt);
            },
            Response::HostUnreachable => {
                self.errors += 1;
            },
            Response::Dropped => {
                self.dropped += 1;
            },
        }
    }
    // Calculate and print statistics to stdout
    pub fn print(self) {
        println!("\n--- {} ping statistics ---", self.destination);

        print!("{} packets transmitted, {} received, ", self.sent, self.received);

        if self.errors > 0 {
            print!("+{} errors, ", self.errors);
        }

        let packet_loss = if self.sent > 0 {
            (self.sent - self.received) as f32 / self.sent as f32 * 100_f32
        } else {
            0_f32
        };

        // Total amount of time the program has been running
        let duration = self.start.elapsed().as_micros() as f32 / 1000f32;

        println!("{:.0}% packet loss, time {:.0} ms", packet_loss, duration);

        // Use round trip time data to determine min/avg/max/mdev
        if !self.rtt.is_empty() {
            let min = self.rtt.iter().fold(f32::MAX, |a, b| a.min(*b));
            let max = self.rtt.iter().fold(f32::MIN, |a, b| a.max(*b));
            let avg: f32 = self.rtt.iter().sum::<f32>() / self.rtt.len() as f32;

            // Calculate standard deviation
            let sum_deviations: f32 = self.rtt.iter().map(|f| (f - avg).powi(2)).sum();
            let sdev = (sum_deviations / self.rtt.len() as f32).sqrt();

            println!(
                "rtt min/avg/max/mdev = {:.3}/{:.3}/{:.3}/{:.3} ms",
                min, avg, max, sdev
            );
        }

        println!();
    }
}
