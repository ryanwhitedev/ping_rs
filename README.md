# Ping

This is a basic IPv4-only implementation of the ping utility written in Rust. It was created as a learning project to understand low-level networking concepts, such as socket programming and ICMP, and systems programming concepts, including syscalls and interrupt handling.

This project leverages the `libc` library, primarily for it's convienience over creating a foreign-function interface for raw sockets. `libc` is used to send ICMP packets over raw sockets to communicate with a specified host. Packet statistics are tracked and a report is generated when the program is terminated.

## Prerequisites
- Rust 1.56.0 or greater

## Usage
1. Clone the repository:
```
git clone <repo>
```

2. Navigate to the project directory:
```
cd ping_rs
```

3. Run `cargo build`:
```
cargo build
```

4. Run the ping utility:
```
sudo ./target/debug/ping <hostname or IP address>
```
Note: using raw sockets requires superuser privileges or the `cap_net_raw` capability.

