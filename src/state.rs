use std::net::Ipv4Addr;

use etherparse::{IpTrafficClass, Ipv4HeaderSlice, TcpHeaderSlice};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Connection {
    pub source: (Ipv4Addr, u16),
    pub destination: (Ipv4Addr, u16),
}

pub enum TcpState {
    Closed,
    Listen,
    SynReceived,
    Established,
}

impl Default for TcpState {
    fn default() -> Self {
        TcpState::Listen
    }
}

impl TcpState {
    pub fn on_packet(
        &mut self,
        nic: &mut tun_tap::Iface,
        ip_headers: Ipv4HeaderSlice,
        tcp_headers: TcpHeaderSlice,
        packet_body: & [u8],
    ) -> std::io::Result<usize> {

        return match *self {
            TcpState::Closed => Ok(0),
            TcpState::Listen => self.start_listening(nic, ip_headers, tcp_headers, &packet_body),
            TcpState::SynReceived => Result::Err(unimplemented!()),
            _ => Result::Err(unimplemented!()),
        };
    }

    fn start_listening(
        &mut self,
        nic: &mut tun_tap::Iface,
        ip_headers: Ipv4HeaderSlice,
        tcp_headers: TcpHeaderSlice,
        packet_body: &[u8],
    ) -> std::io::Result<usize> {
        let mut buf = [0u8; 1500];

        if !tcp_headers.syn() {
            // expected synchronise, didn't get one
            return Ok(0);
        }

        let mut syn_ack = etherparse::TcpHeader::new(
            tcp_headers.destination_port(),
            tcp_headers.source_port(),
            // work these out later
            0, 
            0
        );
        syn_ack.ack = true;
        syn_ack.syn = true;

        let ip_response = etherparse::Ipv4Header::new(
            syn_ack.header_len(),
            64,
            IpTrafficClass::Tcp,
            ip_headers.destination_addr().octets(),
            ip_headers.source_addr().octets(),
        );
        
        let unwritten = {
            let mut unwritten = &mut buf[..];
            ip_response.write(&mut unwritten).unwrap();
            syn_ack.write(&mut unwritten)?;
            unwritten.len()
        };

        nic.send(&packet_body[..unwritten])
    }
}
