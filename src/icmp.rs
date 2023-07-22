use std::net::Ipv4Addr;
use std::time::Instant;

use crate::ipv4;
use crate::socket::{SocketError, SocketIcmp};

pub const ICMP_HDR_LEN: usize = 20;
const DEFAULT_TIMEOUT: i32 = 4000; // ms

const ECHO_REQUEST: u8 = 8;
const ECHO_CODE: u8 = 0;

#[derive(Debug)]
pub struct Request {
    dst_addr: Ipv4Addr,
    pid: u16,
    seq: u16,
    payload: Vec<u8>,
}

impl Request {
    pub fn new(dst_addr: Ipv4Addr, pid: u16, seq: u16, payload: Vec<u8>) -> Self {
        Self {
            dst_addr,
            pid,
            seq,
            payload,
        }
    }
    pub fn pack(&self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.push(ECHO_REQUEST);
        packet.push(ECHO_CODE);
        packet.extend(0u16.to_le_bytes());
        packet.extend(self.pid.to_le_bytes());
        packet.extend(self.seq.to_le_bytes());
        packet.extend(self.payload.clone());

        // Calc checksum from packet with zeroed checksum
        let checksum = crate::util::ip_checksum(&packet);

        // Replace zeroed checksum with actual checksum
        let checksum_bytes = checksum.to_le_bytes();
        packet[2] = checksum_bytes[0];
        packet[3] = checksum_bytes[1];

        packet
    }
    pub fn send(self) -> Result<Reply, String> {
        let request = self.pack();

        let now = Instant::now();

        let socket = SocketIcmp::new(DEFAULT_TIMEOUT).expect("failed creating icmp socket");
        let _sent_bytes = socket.sendto(&request, self.dst_addr).unwrap();

        // Receive buffer
        let mut buf: [u8; 128] = [0; 128];
        let recv_bytes = match socket.recvfrom(&mut buf) {
            Ok(bytes) => bytes,
            Err(SocketError::TimedOut) => return Ok(Reply::Dropped),
            Err(e) => return Err(format!("recv error: {}", e)),
        };

        // Round trip time in ms
        let rtt_ms = now.elapsed().as_micros() as f32 / 1000f32;
        dbg!(rtt_ms);

        let ip_hdr = match ipv4::HdrIpv4::try_from(&buf[0..ipv4::IP_HDR_LEN]) {
            Ok(hdr) => hdr,
            Err(_) => return Err("failed to parse IP header".into()),
        };

        Reply::parse(
            &buf[ipv4::IP_HDR_LEN..recv_bytes as usize],
            ip_hdr.ttl,
            rtt_ms,
        )
        .map_err(|e| e.to_string())
    }
}

#[derive(Debug)]
pub enum Reply {
    Echo {
        ttl: u8,
        rtt: f32,
        data: Vec<u8>,
    },
    Dropped,
    HostUnreachable,
    Unknown,
}

impl Reply {
    pub fn parse(bytes: &[u8], ttl: u8, rtt: f32) -> Result<Reply, Box<dyn std::error::Error>> {
        let data = bytes.to_vec();
        dbg!(bytes);
        match (bytes[0], bytes[1]) {
            (0, 0) => Ok(Reply::Echo {
                ttl,
                rtt,
                data,
            }),
            (3, 1) => Ok(Reply::HostUnreachable),
            (_, _) => Ok(Reply::Unknown),
        }
    }
}
