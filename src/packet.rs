pub struct Packet {
    pub bytes: [u8; 1504],
    pub length: usize,
}

impl Packet {
    pub fn new(bytes: [u8; 1504], length: usize) -> Self {
        return Self { bytes, length };
    }

    pub fn protocol(&self) -> Protocol {
        let proto_bytes = self.protocol_bytes();

        match proto_bytes {
            0x0800 => Protocol::Ipv4,
            _ => Protocol::Other,
        }
    }

    pub fn protocol_bytes(&self) -> u16 {
        u16::from_be_bytes([self.bytes[2], self.bytes[3]])
    }
}

#[derive(PartialEq)]
pub enum Protocol {
    Ipv4,
    Other,
}
