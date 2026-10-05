<<<<<<< HEAD
use std::{
    io::prelude::*,
    net::{IpAddr, Ipv4Addr, SocketAddr,  UdpSocket},
    thread,
};
use bevy::{prelude::*};
use rand::{prelude::*};

=======
use std::{io::ErrorKind, net::UdpSocket};
use bevy::prelude::*;
>>>>>>> 1bf391c8fe9aa3f838cea30f9d0953b356ecb8d6

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
<<<<<<< HEAD
        app.add_systems(Update, client_listener);
           
    }
}

fn client_listener()-> Result<()>
{

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 34254);
    println!("Sending UDP datagram to {}:{}", "127.0.0.1", "34254");
    //let listener = UdpSocket::bind("127.0.0.1:34254");
    //let cli = Cli::parse();
    //let socket = UdpSocket::bind("127.0.0.1:34254")?;
    //info!("Listening @{}", socket.local_addr()?);
    Ok(())
}


/*
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
}*/

/*fn main() -> Result<()> {
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
}*/
=======
        app.add_systems(Startup, setup_client_socket)
            .add_systems(Update, client_listener);
    }
}

#[derive(Resource)]
struct ClientSocket {
    socket: UdpSocket,
    sent_request: bool,
}

fn setup_client_socket(mut commands: Commands) {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind client socket");
    socket.set_nonblocking(true).expect("couldn't set nonblocking");
    commands.insert_resource(ClientSocket { socket, sent_request: false });
}

fn client_listener(mut client: ResMut<ClientSocket>) {
    if !client.sent_request {
        let n = client.socket
            .send_to(b"hii", "127.0.0.1:7777")
            .expect("couldn't send to server");
        println!("sent {n} bytes to server");
        client.sent_request = true;
    }

    let mut buf = [0u8; 1024];
    match client.socket.recv_from(&mut buf) {
        Ok((n, from)) => {
            println!("got {n} bytes from {from}: {:?}", &buf[..n]);
        }
        Err(e) if e.kind() == ErrorKind::WouldBlock => {}
        Err(e) => panic!("recv error: {e}"),
    }
}
>>>>>>> 1bf391c8fe9aa3f838cea30f9d0953b356ecb8d6
