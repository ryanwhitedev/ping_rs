use std::net::Ipv4Addr;

use crate::icmp::Response;

pub static mut SIGNAL_CTRL_C: bool = false;

pub fn register_signal_handlers() {
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

pub fn statistics(dst_addr: Ipv4Addr, period: f32, data: Vec<Response>) {
    println!("\n--- {} ping statistics ---", dst_addr);

    let sent = data.len();
    let received = data
        .iter()
        .filter(|&p| matches!(p, Response::EchoReply { .. }))
        .count();

    print!("{} packets transmitted, {} received, ", sent, received);

    let errors = data
        .iter()
        .filter(|&p| matches!(p, Response::HostUnreachable))
        .count();

    if errors > 0 {
        print!("+{} errors, ", errors);
    }

    let packet_loss = if sent > 0 {
        (sent - received) as f32 / sent as f32 * 100_f32
    } else {
        0_f32
    };

    println!("{:.0}% packet loss, time {:.0} ms", packet_loss, period);

    let rtt: Vec<f32> = data
        .iter()
        .filter_map(|p| match p {
            Response::EchoReply { rtt, .. } => Some(*rtt),
            _ => None,
        })
        .collect();

    // Print min/avg/max/mdev when we have results
    if !rtt.is_empty() {
        let min = rtt.iter().fold(f32::MAX, |a, b| a.min(*b));
        let max = rtt.iter().fold(f32::MIN, |a, b| a.max(*b));
        let avg: f32 = rtt.iter().sum::<f32>() / rtt.len() as f32;

        // Calculate standard deviation
        let sum_deviations: f32 = rtt.iter().map(|f| (f - avg).powi(2)).sum();
        let sdev = (sum_deviations / rtt.len() as f32).sqrt();

        println!(
            "rtt min/avg/max/mdev = {:.3}/{:.3}/{:.3}/{:.3} ms",
            min, avg, max, sdev
        );
    }

    println!("\n");
}

// One's complement checksum
pub fn ip_checksum(bytes: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    // Split bytes into 16 bit words and sum
    for chunk in bytes.chunks(2) {
        let word = if chunk.len() == 2 {
            u16::from_le_bytes(chunk.try_into().unwrap()) as u32
        } else {
            chunk[0] as u32
        };
        sum += word;
    }
    // Get carry bytes from sum
    let carry = (sum >> 16) as u16;
    // Add carry to result
    let result: u16 = (sum & 0xFFFF) as u16 + carry;
    // Take ones complement
    result ^ 0xFFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_calculates_the_correct_checksum() {
        let header = [
            0x45, 0x0, 0x0, 0x73, 0x0, 0x0, 0x40, 0x0, 0x40, 0x11, 0x0, 0x0, 0xc0, 0xa8, 0x0, 0x1,
            0xc0, 0xa8, 0x0, 0xc7,
        ];
        let checksum = ip_checksum(&header);
        let expected = u16::from_le_bytes([0xb8, 0x61]);
        assert_eq!(checksum, expected);
    }
}
