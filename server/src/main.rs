use anyhow::Result;
use clap::Parser;
use log::{error, info};
use shared::Message;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    addr: IpAddr,

    #[arg(short, long, default_value_t = 8080, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let socket = UdpSocket::bind(SocketAddr::new(cli.addr, cli.port))?;

    info!("Listening @{}", socket.local_addr()?);

    loop {
        let mut buf = [0_u8; 1024];
        let (bytes_read, peer_addr) = socket.recv_from(&mut buf)?;
        if bytes_read == 0 {
            info!("({}) Received 0 bytes, ignoring.", peer_addr);
            continue;
        }

        if let Err(e) = handle_datagram(&socket, &buf[..bytes_read], peer_addr) {
            error!("Error handling datagram from {}: {:?}", peer_addr, e);
        }
    }
}

fn handle_datagram(socket: &UdpSocket, buf: &[u8], peer_addr: SocketAddr) -> Result<()> {
    info!("Accepted datagram from {}", peer_addr);
    info!("({}) Read {} bytes.", peer_addr, buf.len());

    let recvd: Message = bincode::deserialize(buf)?;
    info!("({}) Got: {:?}", peer_addr, recvd);

    let to_send = recvd.increment();
    info!("({}) Sending: {:?}", peer_addr, to_send);

    let to_send_bytes = bincode::serialize(&to_send)?;
    socket.send_to(&to_send_bytes, peer_addr)?;
    info!("({}) {} bytes sent!", peer_addr, to_send_bytes.len());

    Ok(())
}
