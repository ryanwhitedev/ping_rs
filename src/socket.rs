use std::net::Ipv4Addr;

pub struct IcmpSocket {
    fd: i32,
}

impl IcmpSocket {
    pub fn new() -> Result<IcmpSocket, std::io::Error> {
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_RAW, libc::IPPROTO_ICMP) };
        if fd < 0 {
            // Read the value of `errno` for the target platform
            let error = std::io::Error::last_os_error();
            println!("os error: {}", error);
            Err(error)
        } else {
            Ok(IcmpSocket { fd })
        }
    }
    pub fn sendto(&self, pkt: &[u8], addr: Ipv4Addr) -> Result<usize, std::io::Error> {
        let addr = libc::sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: 0,
            sin_addr: libc::in_addr {
                s_addr: addr.into(),
            },
            sin_zero: [0; 8],
        };

        // Coerce `sockaddr_in` struct to `sockaddr`
        let sockaddr = unsafe { std::mem::transmute::<libc::sockaddr_in, libc::sockaddr>(addr) };
        let socklen = std::mem::size_of_val(&sockaddr) as libc::socklen_t;

        let bytes = unsafe {
            libc::sendto(
                self.fd,
                pkt.as_ptr() as *const libc::c_void,
                pkt.len(),
                0,
                &sockaddr,
                socklen,
            )
        };

        // Check for sending errors
        if bytes < 0 {
            let error = std::io::Error::last_os_error();
            println!("os error: {}", error);
            Err(error)
        } else {
            Ok(bytes as usize)
        }
    }
    pub fn recvfrom(&self, buffer: &mut [u8]) -> Result<usize, std::io::Error> {
        let mut addr = std::mem::MaybeUninit::<libc::sockaddr>::uninit();
        let mut len = std::mem::size_of_val(&addr) as libc::socklen_t;

        let recv_bytes = unsafe {
            libc::recvfrom(
                self.fd,
                buffer.as_ptr() as *mut libc::c_void,
                buffer.len(),
                0,
                addr.as_mut_ptr() as *mut libc::sockaddr,
                &mut len as *mut libc::socklen_t,
            )
        };

        // Check for recv errors
        if recv_bytes < 0 {
            let error = std::io::Error::last_os_error();
            println!("os error: {}", error);
            Err(error)
        } else {
            Ok(recv_bytes as usize)
        }
    }
}

