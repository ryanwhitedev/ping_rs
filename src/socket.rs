use std::fmt;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub enum SocketError {
    TimedOut,
    IoError(std::io::Error),
    PollError(i32),
}

impl fmt::Display for SocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TimedOut => f.write_str("socket timed out"),
            Self::IoError(err) => write!(f, "IO error: {}", err),
            Self::PollError(errno) => write!(f, "Poll error: {}", errno),
        }
    }
}

impl std::error::Error for SocketError {}

pub struct SocketIcmp {
    fd: i32,
    timeout: i32,
}

impl SocketIcmp {
    pub fn new(timeout: i32) -> Result<SocketIcmp, SocketError> {
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_RAW, libc::IPPROTO_ICMP) };
        if fd < 0 {
            // Read the value of `errno` for the target platform
            let error = std::io::Error::last_os_error();
            Err(SocketError::IoError(error))
        } else {
            Ok(SocketIcmp { fd, timeout })
        }
    }
    pub fn sendto(&self, pkt: &[u8], dst_addr: Ipv4Addr) -> Result<isize, SocketError> {
        let addr = libc::sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: 0,
            sin_addr: libc::in_addr {
                s_addr: u32::from_be(dst_addr.into()),
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
            Err(SocketError::IoError(error))
        } else {
            Ok(bytes)
        }
    }
    pub fn recvfrom(&self, buffer: &mut [u8]) -> Result<isize, SocketError> {
        let mut addr = std::mem::MaybeUninit::<libc::sockaddr>::uninit();
        let mut len = std::mem::size_of_val(&addr) as libc::socklen_t;

        let mut pollfd = libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN,
            revents: 0,
        };

        unsafe {
            // Use `poll` to set a timeout on the socket fd
            let events = libc::poll(&mut pollfd as *mut libc::pollfd, 1, self.timeout);
            // `poll` timed out
            if events == 0 {
                return Err(SocketError::TimedOut);
            }

            // fd is ready to read
            if pollfd.revents & libc::POLLIN != 0 {
                let recv_bytes = libc::recvfrom(
                    self.fd,
                    buffer.as_ptr() as *mut libc::c_void,
                    buffer.len(),
                    0,
                    addr.as_mut_ptr() as *mut libc::sockaddr,
                    &mut len as *mut libc::socklen_t,
                );

                // Check for recv errors
                if recv_bytes < 0 {
                    let error = std::io::Error::last_os_error();
                    Err(SocketError::IoError(error))
                } else {
                    Ok(recv_bytes)
                }
            } else {
                // unexpected event
                Err(SocketError::PollError(pollfd.revents.into()))
            }
        }
    }
}
