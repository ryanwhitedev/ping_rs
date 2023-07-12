// One's complement checksum
pub fn ip_checksum(bytes: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    // Split bytes into 16 bit words and sum
    for chunk in bytes.chunks(2) {
        let word = if chunk.len() == 2 {
            u16::from_be_bytes(chunk.try_into().unwrap()) as u32
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
        let expected = u16::from_be_bytes([0xb8, 0x61]);
        assert_eq!(checksum, expected);
    }
}

