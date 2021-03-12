extern crate tun_tap;
extern crate etherparse;

use std::io;
use tun_tap::{Iface, Mode};


struct Packet {
    bytes: [u8; 1504],
}

impl Packet {
    fn new(bytes: [u8; 1504]) -> Self {
        return Self {
            bytes,
        }
    }

    fn flags(&self) -> u16 {
        return u16::from_be_bytes([self.bytes[0], self.bytes[1]]);
    }

    fn protocol(&self) -> Protocol {
        let proto_bytes = self.protocol_bytes();

        match proto_bytes {
            0x0800 => Protocol::Ipv4,
            _ => Protocol::Other
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
        let bytes_read = nic.recv(&mut buf[..])?;
        let packet = Packet::new(buf);

        match packet.protocol() {
            Protocol::Ipv4 => {
                 match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..bytes_read]) {
                    Ok(p) => {
                        eprintln!(
                            "read {} bytes: (flags: {:x}, proto: {:x}) {:x?}",
                            bytes_read - 4,
                            packet.flags(),
                            packet.protocol_bytes(),
                            p
                        );
                    }
                    Err(e) =>  {
                        eprintln!("Dodgy packet ignored, {:?}", e);
                    }
                }
            }
            Protocol::Other => {
                println!("ignoring non-ipv4 packet");
                continue;
            }
        }
    }
    Result::Ok(())
}
