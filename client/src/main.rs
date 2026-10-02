use anyhow::Result;
use clap::Parser;
use shared::Message;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    addr: IpAddr,

    #[arg(short, long, default_value_t = 8080, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    #[arg(short, long, default_value_t = 5)]
    int_to_send: i32,

    #[arg(short, long, default_value_t = String::from("foo"))]
    string_to_send: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let server_addr = SocketAddr::new(cli.addr, cli.port);

    println!("Sending UDP datagram to {}:{}", cli.addr, cli.port);

    let msg = Message::new(cli.int_to_send, cli.string_to_send, true);
    let msg_bytes = bincode::serialize(&msg)?;

    socket.send_to(&msg_bytes, server_addr)?;

    println!("Sent {} bytes to server!", msg_bytes.len());

    println!("Getting server response...");
    let mut buf = [0_u8; 1024];
    let (bytes_read, _) = socket.recv_from(&mut buf)?;
    if bytes_read == 0 {
        println!("No UDP response received.");
        return Ok(());
    } else {
        println!("Read {} bytes.", bytes_read);
    }

    let recvd: Message = bincode::deserialize(&buf[..bytes_read])?;
    println!("Got: {:?}", recvd);

    Ok(())
}
