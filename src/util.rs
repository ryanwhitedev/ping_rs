use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};

pub static mut SIGNAL_CTRL_C: bool = false;

// Resolves hostname, or converts the string representation of an ipv4 address, into the IPv4 address type.
pub fn resolve_hostname(destination: &str) -> Result<Ipv4Addr, String> {
    let destination = format!("{}:0", destination);
    let socket_addrs: Vec<_> = destination
        .to_socket_addrs()
        .expect("unable to resolve hostname")
        .filter(|addr| addr.is_ipv4())
        .collect();

    if let Some(addr) = socket_addrs.get(0) {
        match addr.ip() {
            IpAddr::V4(ipv4) => Ok(Ipv4Addr::from(ipv4.octets())),
            IpAddr::V6(_) => Err("IPv6 is not implemented".into()),
        }
    } else {
        Err("unable to resolve hostname".into())
    }
}

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
