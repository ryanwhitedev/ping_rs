use std::fmt;
use std::net::Ipv4Addr;
use std::time::Instant;

use crate::ip;
use crate::socket::{SocketError, SocketIcmp};

pub const ICMP_HDR_LEN: usize = 8;

pub const MAX_RECV_RETRIES: u8 = 8;

const ECHO_REQUEST: u8 = 8;
const ECHO_REQUEST_CODE: u8 = 0;
const ECHO_REPLY: u8 = 0;
const ECHO_REPLY_CODE: u8 = 0;
const DEST_UNREACHABLE: u8 = 3;
const HOST_UNREACHABLE_CODE: u8 = 1;

#[derive(Debug)]
pub struct Request {
    dst_addr: Ipv4Addr,
    pid: u16,
    seq: u16,
    payload: Vec<u8>,
    timeout: i32,
}

impl Request {
    pub fn new(dst_addr: Ipv4Addr, pid: u16, seq: u16, payload: Vec<u8>, timeout: i32) -> Self {
        Self {
            dst_addr,
            pid,
            seq,
            payload,
            timeout,
        }
    }
    pub fn pack(&self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.push(ECHO_REQUEST);
        packet.push(ECHO_REQUEST_CODE);
        packet.extend(0u16.to_be_bytes());
        packet.extend(self.pid.to_be_bytes());
        packet.extend(self.seq.to_be_bytes());
        packet.extend(self.payload.clone());

        // Calc checksum from packet with zeroed checksum
        let checksum = crate::util::ip_checksum(&packet);

        // Replace zeroed checksum with actual checksum
        let checksum_bytes = checksum.to_le_bytes();
        packet[2] = checksum_bytes[0];
        packet[3] = checksum_bytes[1];

        packet
    }
    pub fn send(self) -> Result<Response, ResponseError> {
        let request = self.pack();

        let now = Instant::now();

        let socket = SocketIcmp::new(self.timeout).expect("failed creating icmp socket");
        let _sent_bytes = socket.sendto(&request, self.dst_addr).unwrap();

        for _ in 0..MAX_RECV_RETRIES {
            let mut buf: [u8; 128] = [0; 128];
            let recv_bytes = match socket.recvfrom(&mut buf) {
                Ok(bytes) => bytes,
                Err(SocketError::TimedOut) => return Ok(Response::Dropped),
                Err(e) => return Err(ResponseError::Error(format!("recv error: {}", e))),
            };

            // Round trip time in ms
            let rtt_ms = now.elapsed().as_micros() as f32 / 1000f32;

            let ip_hdr = match ip::HdrIpv4::try_from(&buf[0..ip::IPV4_HDR_LEN]) {
                Ok(hdr) => hdr,
                Err(_) => return Err(ResponseError::Error("failed to parse IP header".into())),
            };

            match Response::parse(
                &buf[ip::IPV4_HDR_LEN..recv_bytes as usize],
                self.pid,
                self.seq,
                ip_hdr.src_addr,
                ip_hdr.ttl,
                rtt_ms,
            ) {
                Ok(resp) => match resp {
                    Response::EchoReply {
                        addr,
                        seq,
                        ttl,
                        rtt,
                        ref data,
                    } => {
                        println!(
                            "{} bytes from {}: icmp_seq={} ttl={} time={:.2} ms",
                            data.len(),
                            addr,
                            seq,
                            ttl,
                            rtt,
                        );
                        return Ok(resp);
                    }
                    Response::Dropped => {
                        println!("Packet Dropped");
                        return Ok(resp);
                    }
                    Response::HostUnreachable => {
                        println!(
                            "From {} ({}) icmp_seq={} Destination Host Unreachable",
                            self.dst_addr, self.dst_addr, self.seq
                        );
                        return Ok(resp);
                    }
                },
                Err(error) => match error {
                    ResponseError::UnexpectedPacket => {
                        // Go to start of loop and check if next packet matches
                        continue;
                    }
                    ResponseError::Error(err) => {
                        println!("Response error: {}", err);
                        std::process::exit(1);
                    }
                },
            }
        }
        Err(ResponseError::Error("exceeded max recv retries".into()))
    }
}

#[derive(Debug)]
pub enum Response {
    EchoReply {
        addr: Ipv4Addr,
        seq: u16,
        ttl: u8,
        rtt: f32,
        data: Vec<u8>,
    },
    HostUnreachable,
    Dropped,
}

impl Response {
    pub fn parse(
        bytes: &[u8],
        req_pid: u16,
        req_seq: u16,
        addr: Ipv4Addr,
        ttl: u8,
        rtt: f32,
    ) -> Result<Response, ResponseError> {
        let icmp_type = bytes[0];
        let icmp_code = bytes[1];
        let pid = u16::from_be_bytes(bytes[4..6].try_into().unwrap());
        let seq = u16::from_be_bytes(bytes[6..8].try_into().unwrap());
        let data = bytes.to_vec();

        match (icmp_type, icmp_code) {
            (ECHO_REPLY, ECHO_REPLY_CODE) if req_pid == pid && req_seq == seq => {
                Ok(Response::EchoReply {
                    addr,
                    seq,
                    ttl,
                    rtt,
                    data,
                })
            }
            (DEST_UNREACHABLE, HOST_UNREACHABLE_CODE) => Ok(Response::HostUnreachable),
            (_, _) => Err(ResponseError::UnexpectedPacket),
        }
    }
}

#[derive(Debug)]
pub enum ResponseError {
    UnexpectedPacket,
    Error(String),
}

impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedPacket => f.write_str("Received unexpected packet"),
            Self::Error(err) => write!(f, "Response error: {}", err),
        }
    }
}

impl std::error::Error for ResponseError {}
