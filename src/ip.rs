use std::net::Ipv4Addr;

pub const IPV4_HDR_LEN: usize = 20;

#[derive(Debug)]
pub struct HdrIpv4 {
    pub vers_ihl: u8,
    pub dcsp_ecn: u8,
    pub len: u16,
    pub id: u16,
    pub flags_off: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub src_addr: Ipv4Addr,
    pub dst_addr: Ipv4Addr,
}

impl TryFrom<&[u8]> for HdrIpv4 {
    type Error = std::array::TryFromSliceError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let vers_ihl = bytes[0];
        let dcsp_ecn = bytes[1];
        let len = u16::from_be_bytes(bytes[2..4].try_into()?);
        let id = u16::from_be_bytes(bytes[4..6].try_into()?);
        let flags_off = u16::from_be_bytes(bytes[6..8].try_into()?);
        let ttl = bytes[8];
        let protocol = bytes[9];
        let checksum = u16::from_be_bytes(bytes[10..12].try_into()?);

        // src_addr is backwards... dst_addr is correct...
        let src_addr = Ipv4Addr::from(<&[u8] as TryInto<[u8; 4]>>::try_into(&bytes[12..16])?);
        let dst_addr = Ipv4Addr::from(<&[u8] as TryInto<[u8; 4]>>::try_into(&bytes[16..20])?);

        Ok(Self {
            vers_ihl,
            dcsp_ecn,
            len,
            id,
            flags_off,
            ttl,
            protocol,
            checksum,
            src_addr,
            dst_addr,
        })
    }
}
