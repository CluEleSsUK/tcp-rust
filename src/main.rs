extern crate etherparse;
extern crate tun_tap;

mod packet;
mod state;

use std::collections::HashMap;
use std::io;

use tun_tap::{Iface, Mode};

use crate::packet::{Packet, Protocol};
use crate::state::{Connection, TcpState};

fn main() -> io::Result<()> {
    let mut connections: HashMap<Connection, TcpState> = Default::default();

    let nic = Iface::new("tun0", Mode::Tun).expect("Failed to create tunnel");
    let mut buf = [0u8; 1504];

    loop {
        nic.recv(&mut buf[..])?;
        let packet = Packet::new(buf);

        match packet.protocol() {
            Protocol::Ipv4 => handle_ip_packet(&mut connections, packet),
            Protocol::Other => {
                println!("ignoring non-ipv4 packet");
                continue;
            }
        }
    }
    Result::Ok(())
}

fn handle_ip_packet(state: &mut HashMap<Connection, TcpState>, packet: Packet) {
    let buf = packet.bytes;
    let buffer_length = packet.bytes.len();

    match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..buffer_length]) {
        Ok(ip_headers) => {
            // if not TCP
            if ip_headers.protocol() != 0x06 {
                eprintln!("ignoring non-tcp packet");
                return;
            }

            let start_of_tcp_headers = 4 + ip_headers.slice().len();
            match etherparse::TcpHeaderSlice::from_slice(&buf[start_of_tcp_headers..]) {
                Ok(tcp_headers) => {
                    let start_of_tcp_packet = start_of_tcp_headers + tcp_headers.slice().len();
                    state
                        .entry(Connection {
                            source: (ip_headers.source_addr(), tcp_headers.source_port()),
                            destination: (
                                ip_headers.destination_addr(),
                                tcp_headers.destination_port(),
                            ),
                        })
                        .or_default()
                        .on_packet(ip_headers, tcp_headers, &buf[start_of_tcp_packet..])
                }
                Err(e) => {
                    eprintln!("Dodgy TCP header packet, {:?}", e)
                }
            }
        }
        Err(e) => eprintln!("Dodgy packet ignored, {:?}", e),
    }
}
