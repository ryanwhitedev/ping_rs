use std::net::Ipv4Addr;
use std::time::Instant;

use crate::{ipv4, socket};

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
        Self { dst_addr, pid, seq, payload }
    }
    pub fn pack(&self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.push(ECHO_REQUEST);
        packet.push(ECHO_CODE);
        packet.extend(0u16.to_be_bytes());
        packet.extend(self.pid.to_be_bytes());
        packet.extend(self.seq.to_be_bytes());
        packet.extend(self.payload.clone());

        // Calc checksum from packet with zeroed checksum
        let checksum = crate::util::ip_checksum(&packet);

        // Replace zeroed checksum with actual checksum
        let checksum_bytes = checksum.to_be_bytes();
        packet[2] = checksum_bytes[0];
        packet[3] = checksum_bytes[1];

        packet
    }
    pub fn send(self) -> Result<Reply, String> {
        let request = self.pack();

        let now = Instant::now();

        let socket = socket::SocketIcmp::new(DEFAULT_TIMEOUT).expect("failed creating icmp socket");
        let _sent_bytes = socket.sendto(&request, self.dst_addr).unwrap();

        // Receive buffer
        let mut buf: [u8; 128] = [0; 128];
        let recv_bytes = socket.recvfrom(&mut buf).unwrap();

        // Round trip time in ms
        let rtt_ms = now.elapsed().as_micros() as f32 / 1000f32;

        let ip_hdr = match ipv4::HdrIpv4::try_from(&buf[0..ipv4::IP_HDR_LEN]) {
            Ok(hdr) => hdr,
            Err(_) => return Err("failed to parse IP header".into()),
        };

        match Reply::try_from(&buf[ipv4::IP_HDR_LEN..recv_bytes as usize]) {
            Ok(reply) => {
                println!(
                    "{} bytes from {}: icmp_seq={} ttl={} time={:.2} ms",
                    recv_bytes - ipv4::IP_HDR_LEN as isize,
                    ip_hdr.src_addr,
                    reply.seq,
                    ip_hdr.ttl,
                    rtt_ms
                );
                Ok(reply)
            },
            Err(_) => Err("failed to parse icmp reply".into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Reply {
    pub r#type: u8,
    pub code: u8,
    pub checksum: u16,
    pub pid: u16,
    pub seq: u16,
    pub payload: Vec<u8>,
}

impl TryFrom<&[u8]> for Reply {
    type Error = std::array::TryFromSliceError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let checksum = u16::from_be_bytes(bytes[2..4].try_into()?);
        let pid = u16::from_be_bytes(bytes[4..6].try_into()?);
        let seq = u16::from_be_bytes(bytes[6..8].try_into()?);
        Ok(Reply {
            r#type: bytes[0],
            code: bytes[1],
            checksum,
            pid,
            seq,
            payload: bytes[8..].to_vec(),
        })
    }
}
