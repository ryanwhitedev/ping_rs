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
