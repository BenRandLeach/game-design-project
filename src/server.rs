use std::{io::ErrorKind, net::UdpSocket};
use bevy::prelude::*;

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_server_socket)
       // app.add_systems(Update, server_listener);
            .add_systems(Update, server_listener.run_if(resource_exists::<ServerSocket>));
    }
}

#[derive(Resource)]
struct ServerSocket(UdpSocket);

fn setup_server_socket(mut commands: Commands) {
    match UdpSocket::bind("0.0.0.0:7777") {
        Ok(socket)=>{
            socket.set_nonblocking(true).expect("couldnt set non blocking");
            println!("i am the host!");
            commands.insert_resource(ServerSocket(socket));
        }
        Err(e) => {
            println!("not the host, the port is taken: {e}");
        }
    }
    //let socket = UdpSocket::bind("0.0.0.0:7777").expect("couldn't bind");
    //socket.set_nonblocking(true).expect("couldn't set nonblocking");
    //commands.insert_resource(ServerSocket(socket));
}

fn server_listener(server: Res<ServerSocket>) {
    let mut buf = [0u8; 1024];
    loop{
    match server.0.recv_from(&mut buf) {
        Ok((n, from)) => {
            println!("server got {n} bytes from {from}: {:?}", &buf[..n]);
            // echo back to whoever sent it
            if let Err(e) = server.0.send_to(&buf[..n], from){
                println!("server couldnt reply: {e}");
            }
        }
        Err(e) if e.kind() == ErrorKind::WouldBlock => break, //empty
        Err(e) =>{ 
            println!("recv error: {e}");
            break;
    }
}
}
}
