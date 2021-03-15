use std::net::Ipv4Addr;

use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Connection {
    pub source: (Ipv4Addr, u16),
    pub destination: (Ipv4Addr, u16),
}

pub struct TcpState {}

impl Default for TcpState {
    fn default() -> Self {
        TcpState {}
    }
}

impl TcpState {
    pub fn on_packet(
        &mut self,
        ip_headers: Ipv4HeaderSlice,
        tcp_headers: TcpHeaderSlice,
        _packet_body: &[u8],
    ) {
        eprintln!(
            "{:?} - {:?} {}byte(s) of TCP to port {}",
            ip_headers.source_addr(),
            ip_headers.destination_addr(),
            tcp_headers.slice().len(),
            tcp_headers.destination_port()
        );

    }
}
