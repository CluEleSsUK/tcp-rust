extern crate etherparse;
extern crate tun_tap;

use std::io;
use tun_tap::{Iface, Mode};

struct Packet {
    bytes: [u8; 1504],
}

impl Packet {
    fn new(bytes: [u8; 1504]) -> Self {
        return Self { bytes };
    }

    fn protocol(&self) -> Protocol {
        let proto_bytes = self.protocol_bytes();

        match proto_bytes {
            0x0800 => Protocol::Ipv4,
            _ => Protocol::Other,
        }
    }

    fn protocol_bytes(&self) -> u16 {
        u16::from_be_bytes([self.bytes[2], self.bytes[3]])
    }
}

#[derive(PartialEq)]
enum Protocol {
    Ipv4,
    Other,
}

fn main() -> io::Result<()> {
    let nic = Iface::new("tun0", Mode::Tun).expect("Failed to create tunnel");
    let mut buf = [0u8; 1504];

    loop {
        nic.recv(&mut buf[..])?;
        let packet = Packet::new(buf);

        match packet.protocol() {
            Protocol::Ipv4 => handle_ip_packet(packet),
            Protocol::Other => {
                println!("ignoring non-ipv4 packet");
                continue;
            }
        }
    }
    Result::Ok(())
}

fn handle_ip_packet(packet: Packet) {
    let buf = packet.bytes;
    let bytes_read = packet.bytes.len();

    match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..bytes_read]) {
        Ok(p) => {
            let src = p.source_addr();
            let dest = p.destination_addr();
            let proto = p.protocol();

            // if not TCP
            if proto != 0x06 {
                eprintln!("ignoring non-tcp packet");
                return;
            }

            let start_of_tcp_headers = 4 + p.slice().len();
            match etherparse::TcpHeaderSlice::from_slice(&buf[start_of_tcp_headers..]) {
                Ok(tcp_headers) => {
                    eprintln!("{} - {} {}byte(s) of TCP to port {}", src, dest, tcp_headers.slice().len(), tcp_headers.destination_port());
                }
                Err(e) => eprintln!("Dodgy TCP header packet, {:?}", e),
            }
        }
        Err(e) => {
            eprintln!("Dodgy packet ignored, {:?}", e);
        }
    }
}
