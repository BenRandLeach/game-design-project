<<<<<<< HEAD
use std::{
    io::prelude::*,
    net::{IpAddr, Ipv4Addr, SocketAddr,  UdpSocket},
    thread,
};


use bevy::{prelude::*};
use rand::{prelude::*};


//static PLAYERCOUNT: AtomicI32 = AtomicI32::new(0);

=======
use std::{io::ErrorKind, net::UdpSocket};
use bevy::prelude::*;
>>>>>>> 1bf391c8fe9aa3f838cea30f9d0953b356ecb8d6

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
<<<<<<< HEAD
        app.add_systems(Update, server_listener);
           
    }
}

fn server_listener()-> Result<()>
{
    //let listener = UdpSocket::bind("127.0.0.1:34254");
    //let cli = Cli::parse();
    let socket = UdpSocket::bind("127.0.0.1:34254")?;
    info!("Listening @{}", socket.local_addr()?);
    Ok(())
}


/*fn handle_connection(mut stream: UdpStream) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    info!("Accepted connection from {}", peer_addr);


    let mut buf = [0 as u8; 1024];
    let bytes_read = stream.read(&mut buf)?;
    if bytes_read == 0 {
        info!("({}) Read 0 bytes, connection closed", peer_addr);
        return Ok(());
    } else {
        info!("({}) Read {} bytes.", peer_addr, bytes_read);
    }


    let recvd: Message = bincode::deserialize(&buf)?;
    info!("({}) Got: {:?}", peer_addr, recvd);


    let to_send = recvd.increment();
    info!("({}) Sending: {:?}", peer_addr, to_send);


    let to_send_bytes = bincode::serialize(&to_send)?;
    stream.write_all(&to_send_bytes)?;
    stream.flush()?;
    info!("({}) {} bytes sent!", peer_addr, to_send_bytes.len());


    Ok(())
}*/

=======
        app.add_systems(Startup, setup_server_socket);
        app.add_systems(Update, server_listener);
    }
}

#[derive(Resource)]
struct ServerSocket(UdpSocket);

fn setup_server_socket(mut commands: Commands) {
    let socket = UdpSocket::bind("0.0.0.0:7777").expect("couldn't bind");
    socket.set_nonblocking(true).expect("couldn't set nonblocking");
    commands.insert_resource(ServerSocket(socket));
}

fn server_listener(server: Res<ServerSocket>) {
    let mut buf = [0u8; 1024];
    match server.0.recv_from(&mut buf) {
        Ok((n, from)) => {
            println!("server got {n} bytes from {from}: {:?}", &buf[..n]);
            // echo back to whoever sent it
            server.0.send_to(&buf[..n], from).expect("couldn't reply");
        }
        Err(e) if e.kind() == ErrorKind::WouldBlock => {}
        Err(e) => panic!("recv error: {e}"),
    }
}
>>>>>>> 1bf391c8fe9aa3f838cea30f9d0953b356ecb8d6
