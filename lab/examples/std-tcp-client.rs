use std::{
    io::{Read, Write},
    net::TcpStream,
};

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:3333")?;
    println!("connected to {:?}", stream.local_addr()?);
    stream.write_all(b"2025, rakudo star!")?;

    let mut buf = [0u8; 17];
    stream.read_exact(&mut buf)?;
    println!("data: {:?}", String::from_utf8_lossy(&buf));
    Ok(())
}
