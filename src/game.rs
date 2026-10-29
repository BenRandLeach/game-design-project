use bevy::prelude::*;
/* Artificial lag imports
use std::thread;
use std::time::Duration;
*/
const SLIDE_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

use crate::server::ClientInput;
use crate::server::stop_input;
use crate::client::ServerInput;
use crate::client::stop_server_input;
use crate::{GameState, PROGRESS_FRAME, PROGRESS_HEIGHT, PROGRESS_LENGTH, WIN_H};
use crate::physics::Velocity;
use crate::side::LocalSide;

const GROUND_Y: f32 = -WIN_H / 2.0;
const TOWER_BASE_Y: f32 = (-WIN_H / 2.0) + 20.0;

const BLOCK_WIDTH: f32 = 16.0;
const BLOCK_HEIGHT: f32 = 48.0;

#[derive(Component)]
pub struct Block {
    pub mass: f32,
    pub angular_velocity: f32,
    pub inertia: f32,
    pub fcoeff: f32,
}

#[derive(Component)]
pub struct Slingshot;

#[derive(Component)]
pub struct BeingPlaced;

#[derive(Component)]
pub struct JustDropped;

#[derive(Component)]
pub struct NewEntity;

#[derive(Resource, Default)]
pub struct CurrentlyActive(pub Option<Entity>);

#[derive(Component)]
pub struct Ammo;

#[derive(Message, Default)]
pub struct Game;

#[derive(Component)]
pub struct GameScreen;

#[derive(Resource)]
pub struct GameScreenImage(Handle<Image>);

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_game)
            .add_systems(OnEnter(GameState::Game), setup_game)
            .add_systems(Update, game_mess_listener)
            .add_systems(Update, swap_to_credits.run_if(in_state(GameState::Game)))
            .add_systems(Update, spawn_block.run_if(in_state(GameState::Game)))
            .add_systems(Update, spawn_slingshot.run_if(in_state(GameState::Game)))
            .add_systems(Update, launch_cannonball.run_if(in_state(GameState::Game)))
            //.add_systems(Update, move_blocks.run_if(in_state(GameState::Game)))
            //.add_systems(Update, simulate_lag.run_if(in_state(GameState::Game)))
            .insert_resource(CurrentlyActive::default())
            .add_message::<Game>();
    }
}
/* For testing 
fn simulate_lag() {
    thread::sleep(Duration::from_millis(200));
}
*/

fn game_mess_listener(
    mut game_mess: MessageReader<Game>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !game_mess.is_empty() {
        next_state.set(GameState::Game);
        game_mess.clear();
    }
}

fn load_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    //mut loading_assets: ResMut<LoadingAssets>,
) {
    //let game_texture_handle = asset_server.load("Tower_Balance_Background.png");

    //loading_assets.0.push(game_texture_handle.clone().untyped());
    //commands.insert_resource(GameScreenImage(game_texture_handle));
}

fn move_blocks(
    input: Res<ButtonInput<KeyCode>>,
    mut active: Query<(&mut Transform, &mut Velocity), With<NewEntity>>,
){
    let mut vel = Vec3::ZERO;
    if input.pressed(KeyCode::KeyA) {
        vel.x -= 1.0;
    }

    if input.pressed(KeyCode::KeyD) {
        vel.x += 1.;
    }

    if let Ok((mut transform, mut velocity)) = active.single_mut() {
        transform.translation += vel;
        velocity.x = vel.x;
    }
    //for mut block_transform in &mut blocks { 
    //    block_transform.translation += vel; }
    }


fn spawn_block(mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>, 
    mut currently_active: ResMut<CurrentlyActive>, 
    being_placed: Query<Entity, With<BeingPlaced>>,
    local_side: Res<LocalSide>,
    client_data: ResMut<ClientInput>, 
    server_data:ResMut<ServerInput>,
) {
    if being_placed.iter().count() > 0 {
        return;
    }
    if server_data.input==b'B' || client_data.input==b'B'{
        stop_input(client_data);
        stop_server_input(server_data);
    if let Some(active_entity) = currently_active.0 {
            commands.entity(active_entity).remove::<NewEntity>();
        }
    let mass = 1.0;
    let inertia = mass * ((BLOCK_WIDTH * BLOCK_WIDTH) + (BLOCK_HEIGHT * BLOCK_HEIGHT)) / 12.0;


    let entity = commands.spawn((
        Sprite {
            image: asset_server.load("WoodBlock.png"),
            ..default()
        },
        Block {
            mass,
            angular_velocity: 0.0,
            inertia,
            fcoeff: 0.25,
        },
        // NewEntity,
        BeingPlaced,
        Velocity(Vec2::new(0.,0.)),
        local_side.0,
        Transform::from_xyz(local_side.base_center(), 275., 1.),
)).id();
    currently_active.0 = Some(entity);
    }
}

fn spawn_slingshot(
    mut commands: Commands,keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>, 
    mut currently_active: ResMut<CurrentlyActive>,
    client_data: ResMut<ClientInput>,
    server_data:ResMut<ServerInput>,
    local_side: Res<LocalSide>,
) {
    if server_data.input==b'N' || client_data.input==b'N'{
        let slingshot = commands.spawn((Sprite {image: asset_server.load("Cannon2.png"), ..default()}, 
            Slingshot,
            Block {
                mass: 1.0,
                angular_velocity: 0.0,
                inertia: 0.0,
                fcoeff: 0.75,
            },
            JustDropped,
            Velocity::default(), 
            local_side.0,
            Transform::from_xyz(local_side.base_center(), 275., 2.),)).id();
            stop_input(client_data);
            stop_server_input(server_data);
    }
}

fn launch_cannonball(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    slingshots: Query<&Transform, With<Slingshot>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    client_data: ResMut<ClientInput>,
    server_data:ResMut<ServerInput>,
    local_side: Res<LocalSide>,
) {

        if server_data.input==b'M'|| client_data.input==b'M'{
           let launch_vel = Vec2::new(800.0, 500.0);

           for transform in &slingshots {
                let position = transform.translation;
                commands.spawn((
                Sprite {
                    image: asset_server.load("cannonball2.png"), 
                    custom_size: Some(Vec2::new(250.0, 150.0)),
                    ..default()
                }, 
                Ammo,
                Block {
                mass: 1.0,
                angular_velocity: 0.0,
                inertia: 0.0,
                fcoeff: 0.75,
                },
                Velocity(launch_vel),
                Transform::from_translation(position), 
            ));
           }
        if server_data.input==b'M'|| client_data.input==b'M' {
        stop_input(client_data);
        stop_server_input(server_data);
        }
    }
}

fn swap_to_credits(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        // also need to clear all blocks here!
        next_state.set(GameState::Credits);
    }

}

fn setup_game(
    mut commands: Commands,
    
    asset_server: Res<AssetServer>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    commands.spawn((
        Sprite {
                        image: asset_server.load("Tower_outline.png"),
                        color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                        custom_size: Some(SLIDE_SIZE),
                        ..default()
                    },
        Transform::from_xyz(0., 0., 0.1),
        
    ));
    commands.spawn((
        Sprite {
                        image: asset_server.load("Tower_Balance_Background.png"),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.9),
                        custom_size: Some(SLIDE_SIZE),
                        ..default()
                    },
        Transform::from_xyz(0., 0., 0.),
        
    ));
    camera.translation.x = 0.;
}
