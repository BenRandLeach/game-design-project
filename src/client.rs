use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}};
use bevy::{color::palettes::css::DARK_BLUE, window::WindowEvent::PanGesture};
use bevy::prelude::*;
use crate::GameState;



pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), network_ui)
            .add_systems(Update, do_send_button.run_if(in_state(GameState::Game)))
            .add_systems(Update, client_listener.run_if(resource_exists::<ClientSocket>));
    }
}

#[derive(Resource)]
struct ClientSocket {
    socket: UdpSocket,
    serv_addy: SocketAddr,
}
#[derive(Component)]
struct statusText;

#[derive(Component)]
struct SendButton;

pub fn connect(commands: &mut Commands, text:&str)->String{
    let full = if text.contains(':'){
        text.to_string()

    }else{
        format!("{text}:7777")
    };
    let Ok(serv_addy) = full.parse::<SocketAddr>()
    else{
        return format!("'{text}' is not a valid IP");
    };
    let socket = match UdpSocket::bind("0.0.0.0:0"){
        Ok(s)=> s,
        Err(e)=> return format!("couldnt open socket: {e}"),
    };
    socket.set_nonblocking(true).ok();
    commands.insert_resource(ClientSocket{socket, serv_addy});
    format!("Ready to talk to {serv_addy}")

}
//draws the send button and ststus line 
fn network_ui(mut commands: Commands,client: Option<Res<ClientSocket>>){
    let status_mess = match &client{
        Some(c)=> format!("connected to {}", c.serv_addy),
        None => String::from("not connected"),
    };
    let parent = commands
    .spawn((
        Node{
            position_type: PositionType::Absolute,
            top: px(10),
            left: px(10),
            flex_direction: FlexDirection::Column,
            row_gap:px(8),
            ..default()
        },
        DespawnOnExit(GameState::Game),
    )).id();
    let send_button = commands
    .spawn((
        Button,
        SendButton,
        Node{
            width: px(150),
            height: px(40),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(DARK_GREY.into()),
    )).id();
    let send_label = commands.spawn(Text::new("Send")).id();
    let status = commands.spawn((Text::new(status_mess), statusText)).id();
    commands.entity(send_button).add_child(send_label);
    commands.entity(parent).add_children(&[send_button, status]);
}
 fn do_send_button(
    buttons: Query<&Interaction, (Changed<Interaction>, With<SendButton>)>,
    client: Option<Res<ClientSocket>>,
    mut status: Single<&mut Text, With<statusText>>,
 ){
    for interaction in &buttons{
        if *interaction != Interaction::Pressed{
            continue;
        }
        status.0 = match &client{
            Some(c)=> match c.socket.send_to(b"hii", c.serv_addy){
                Ok(n)=> format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None=> String::from("Not connected, enter IP on the start screen"),
        };
    }
 }


fn client_listener(mut client: ResMut<ClientSocket>, mut status: Query<&mut Text, With<statusText>>) {
    let mut buf = [0u8; 1024];
    loop {
        match client.socket.recv_from(&mut buf) {
        Ok((n, from)) => {
            let reply = String::from_utf8_lossy(&buf[..n]);
            println!("client got {n} bytes from {from}: {:?}", &buf[..n]);
            for mut t in &mut status{
                t.0 = format!("got a reply form {from}: {reply}");
            }
        }
        Err(e) if e.kind() == ErrorKind::WouldBlock => break,
        Err(e) =>{ println!("recv error: {e}"); break;}
    }
        
    }
 
}
