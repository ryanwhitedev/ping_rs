# Ping

This is a basic IPv4-only implementation of the ping utility written in Rust. It was created as a learning project to understand low-level networking concepts, such as socket programming and ICMP, and systems programming concepts, including syscalls and interrupt handling.

This project leverages the `libc` library, primarily for it's convienience over creating a foreign-function interface for raw sockets. `libc` is used to send ICMP packets over raw sockets to communicate with a specified host. Packet statistics are tracked and a report is generated when the program is terminated.

## Prerequisites
- Rust 1.56.0 or greater

## Installation
1. Clone the repository
2. Navigate to the project directory
3. Run `make`
Note: using raw sockets requires superuser privileges or the `cap_net_raw` capability.
```
git clone <repo>
cd ping_rs
make
```

## Usage
```
ping_rs --help
```

