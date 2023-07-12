const ECHO_REQUEST: u8 = 8;
const ECHO_CODE: u8 = 0;

#[derive(Debug)]
pub struct Request {
    pid: u16,
    seq: u16,
    payload: Vec<u8>,
}

impl Request {
    pub fn new(pid: u16, seq: u16, payload: Vec<u8>) -> Self {
        Self { pid, seq, payload }
    }
    pub fn pack(&mut self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.push(ECHO_REQUEST);
        packet.push(ECHO_CODE);
        packet.extend(0u16.to_be_bytes());
        packet.extend(self.pid.to_be_bytes());
        packet.extend(self.seq.to_be_bytes());
        packet.append(&mut self.payload.clone());

        // Calc checksum using packet with zeroed checksum
        let checksum = crate::util::ip_checksum(&packet);

        // Replace zeroed checksum with actual checksum
        let checksum_bytes = checksum.to_be_bytes();
        packet[2] = checksum_bytes[0];
        packet[3] = checksum_bytes[1];

        packet
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
