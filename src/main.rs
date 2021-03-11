extern crate tun_tap;

use std::io;
use tun_tap::{Iface, Mode};

fn main() -> io::Result<()> {
    let nic = Iface::new("tun0", Mode::Tun).expect("Failed to create tunnel");
    let mut buf = [0u8; 1504];
    let bytes_read = nic.recv(&mut buf[..])?;

    eprintln!("read {} bytes: {:x?}", bytes_read, &buf[..bytes_read]);

    Ok(())
}
