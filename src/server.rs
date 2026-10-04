use std::{
    io::prelude::*,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    thread,
};

use bevy::prelude::*;

//static PLAYERCOUNT: AtomicI32 = AtomicI32::new(0);

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_server_socket);
        app.add_systems(Update, server_listener);
    }
}

#[derive(Resource)]
struct ServerSocket(UdpSocket);
fn setup_server_socket(mut commands: Commands) {
    let socket = UdpSocket::bind("127.0.0.1:7777").expect("couldn't bind");
    commands.insert_resource(ServerSocket(socket));
}

fn server_listener(socket: Res<ServerSocket>) {
    //let listener = UdpSocket::bind("127.0.0.1:34254");
    //let cli = Cli::parse();
    let message = "hii";
    let n = socket
        .0
        .send_to(message.as_bytes(), "127.0.0.1:7777")
        .expect("couldn't send");
    println!("sent {n} bytes");
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
