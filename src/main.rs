extern crate etherparse;
extern crate tun_tap;

mod packet;
mod state;

use std::collections::HashMap;
use std::io::Result;

use tun_tap::{Iface, Mode};

use crate::packet::{Packet, Protocol};
use crate::state::{Connection, TcpState};

fn main() -> std::io::Result<()> {
    let mut connections: HashMap<Connection, TcpState> = Default::default();

    let mut nic = Iface::new("tun0", Mode::Tun).expect("Failed to create tunnel");
    let mut buf = [0u8; 1504];

    loop {
        let bytes_read = nic.recv(&mut buf[..])?;
        let packet = Packet::new(buf, bytes_read);

        match packet.protocol() {
            Protocol::Ipv4 => { handle_ip_packet(&mut nic, &mut connections, &packet).unwrap(); () },
            Protocol::Other => {
                println!("ignoring non-ipv4 packet");
                continue;
            }
        }
    }
    Result::Ok(())
}

fn handle_ip_packet(nic: &mut Iface, state: &mut HashMap<Connection, TcpState>, packet: &Packet) -> std::io::Result<usize> {
    let buf = packet.bytes.clone();
    let buffer_length = packet.length;

    match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..buffer_length]) {
        Ok(ip_headers) => {
            // magic number for the TCP protocol ID
            if ip_headers.protocol() != 0x06 {
                eprintln!("ignoring non-tcp packet");
                return Ok(0);
            }

            // the ip protocol and sub-protocol identifiers take up 4 bytes
            let start_of_tcp_headers = 4 + ip_headers.slice().len();

            match etherparse::TcpHeaderSlice::from_slice(&buf[start_of_tcp_headers..buffer_length]) {
                Ok(tcp_headers) => {

                    let start_of_tcp_packet = start_of_tcp_headers + tcp_headers.slice().len();
                    state.entry(Connection {
                            source: (ip_headers.source_addr(), tcp_headers.source_port()),
                            destination: (ip_headers.destination_addr(), tcp_headers.destination_port()),
                        })
                        .or_default()
                        .on_packet(
                            nic,
                            ip_headers,
                            tcp_headers,
                            &buf[start_of_tcp_packet..buffer_length],
                        )
                }
                Err(e) => {
                    eprintln!("Dodgy TCP header packet, {:?}", e);
                    Ok(0)
                }
            }
        }
        Err(e) => {
            eprintln!("Dodgy packet ignored, {:?}", e);
            Ok(0)
        }
    }
}
