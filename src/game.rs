use bevy::prelude::*;
/* Artificial lag imports
use std::thread;
use std::time::Duration;
*/
use crate::server::IsHost;
use crate::side::Side;
use crate::{GameState, WIN_H};
const SLIDE_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

use crate::client::Client1Money;
use crate::client::Client2Money;
use crate::client::ServerInput;
use crate::client::stop_server_input;
use crate::physics::AngularVelocity;
use crate::physics::Velocity;
use crate::server::ClientInput;
use crate::server::stop_input;
use crate::side::LocalSide;
use crate::server::{Player1Money, Player2Money};

const STARTING_MONEY: u64 = 100;
const BLOCK_COST: u64 = 1;
const GROUND_Y: f32 = -WIN_H / 2.0;
const TOWER_BASE_Y: f32 = (-WIN_H / 2.0) + 20.0;
// H: 24, W: 96 for new block
// H: 48, W: 16 for OG block
const BLOCK_WIDTH: f32 = 48.0;
const BLOCK_HEIGHT: f32 = 16.0;

const LAUNCH_SPEED: Vec2 = Vec2::new(800.0, 500.0);
const MUZZLE_OFFSET: f32 = 36.0; // Past the slingshot's hitbox, or the shot collides with its own slingshot

#[derive(Component)]
pub struct StaticBody;

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

#[derive(Resource, Default)]
pub struct NextBlockId(pub u32);

#[derive(Component, Clone, Copy)]
pub struct BlockId(pub u32);

#[derive(Component)]
struct MoneyUi;

#[derive(Component)]
struct MoneyText(u8);

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_game)
            .add_systems(OnEnter(GameState::Game), setup_game)
            .add_systems(OnEnter(GameState::Game), (reset_money, spawn_money_ui))
            .add_systems(Update, game_mess_listener)
            .add_systems(Update, swap_to_credits.run_if(in_state(GameState::Game)))
            .add_systems(Update, update_money_ui.run_if(in_state(GameState::Game)))
            .add_systems(
                Update,
                (spawn_block, spawn_slingshot, launch_cannonball)
                    .run_if(in_state(GameState::Game))
                    .run_if(resource_exists::<IsHost>),
            )
            .add_systems(OnExit(GameState::Game), crate::startscreen::despawn_with::<MoneyUi>)
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
fn spawn_money_ui(mut commands: Commands, asset_server: Res<AssetServer>){
    let coin = asset_server.load("Coin.png");
    for (player, left_side)in[(1u8, true), (2u8, false)]{
        commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(50),
                left: if left_side{px(16)} else{auto()},
                right: if left_side{auto()} else{px(16)},
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
            MoneyUi,
            ))
            .with_children(|parent|{
                parent.spawn((
                    ImageNode::new(coin.clone()),
                    Node{
                        width:px(32),
                        height: px(32),
                        ..default()
                    },
                ));
                parent.spawn((
                    Text::new(STARTING_MONEY.to_string()),
                    TextFont{
                        font_size: FontSize::Px(28.0),
                        ..default()
                    },
                    MoneyText(player),
                ));
            });
    }
}
fn update_money_ui(p1: Res<Client1Money>, p2: Res<Client2Money>, mut texts: Query<(&mut Text, &MoneyText)>,){

    for(mut text, who) in &mut texts{
        let amount = if who.0 == 1{p1.money} else{ p2.money};
        text.0= amount.to_string();
    }
}
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
) {
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

fn spawn_block(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut currently_active: ResMut<CurrentlyActive>,
    being_placed: Query<Entity, With<BeingPlaced>>,
    local_side: Res<LocalSide>,
    client_data: ResMut<ClientInput>,
    server_data: ResMut<ServerInput>,
    mut next_id: ResMut<NextBlockId>,
    mut p1_money: ResMut<Player1Money>,
    mut p2_money: ResMut<Player2Money>,
) {


    if being_placed.iter().count() > 0 {
        return;
    }
    if server_data.input != b'B' && client_data.input != b'B' {
        return;
    }
        let player = client_data.player;
        stop_input(client_data);
        stop_server_input(server_data);

        let wallet = match player {
            1 => &mut p1_money.money,
            2=> &mut p2_money.money,
            _ => return,
            
        };
        if *wallet < BLOCK_COST{
            return;
        }
        *wallet -= BLOCK_COST;
        if let Some(active_entity) = currently_active.0 {
            commands.entity(active_entity).remove::<NewEntity>();
        }
        let id = next_id.0;
        next_id.0 += 1;
        let mass = 1.0;
        let inertia = mass * ((BLOCK_WIDTH * BLOCK_WIDTH) + (BLOCK_HEIGHT * BLOCK_HEIGHT)) / 12.0;

        let entity = commands
            .spawn((
                Sprite {
                    image: asset_server.load("WoodBlock.png"),
                    //custom_size: Some(Vec2::new(BLOCK_WIDTH, BLOCK_HEIGHT)),
                    ..default()
                },
                Block {
                    mass,
                    angular_velocity: 0.0,
                    inertia,
                    fcoeff: 0.25,
                },
                // NewEntity,
                BlockId(id),
                BeingPlaced,
                Velocity(Vec2::new(0., 0.)),
                AngularVelocity(0.),
                local_side.0,
                Transform::from_xyz(local_side.base_center(), 275., 1.),
            ))
            .id();
        currently_active.0 = Some(entity);
    }


fn spawn_slingshot(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut currently_active: ResMut<CurrentlyActive>,
    client_data: ResMut<ClientInput>,
    server_data: ResMut<ServerInput>,
    local_side: Res<LocalSide>,
) {
    if server_data.input == b'N' || client_data.input == b'N' {
        let slingshot = commands
            .spawn((
                Sprite {
                    image: asset_server.load("MPCannon.png"),
                    ..default()
                },
                Slingshot,
                Block {
                    mass: 1.0,
                    angular_velocity: 0.0,
                    inertia: 0.0,
                    fcoeff: 0.75,
                },
                JustDropped,
                Velocity::default(),
                AngularVelocity(0.),
                local_side.0,
                Transform::from_xyz(local_side.base_center(), 275., 2.),
            ))
            .id();
        stop_input(client_data);
        stop_server_input(server_data);
    }
}

fn launch_cannonball(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    slingshots: Query<(&Transform, &Side), With<Slingshot>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    client_data: ResMut<ClientInput>,
    server_data: ResMut<ServerInput>,
    local_side: Res<LocalSide>,
) {
    if server_data.input == b'M' || client_data.input == b'M' {
        for (transform, side) in &slingshots {
            let direction = if *side == Side::Left { 1.0 } else { -1.0 };
            let launch_vel = Vec2::new(LAUNCH_SPEED.x * direction, LAUNCH_SPEED.y);
            let position = transform.translation + Vec3::X * MUZZLE_OFFSET * direction;
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
                AngularVelocity(0.),
                Transform::from_translation(position),
            ));
        }
        stop_input(client_data);
        stop_server_input(server_data);
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
fn reset_money(mut p1: ResMut<Player1Money>, mut p2: ResMut<Player2Money>){
    p1.money = STARTING_MONEY;
    p2.money = STARTING_MONEY;
}

fn setup_game(
    mut commands: Commands,

    asset_server: Res<AssetServer>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    /*commands.spawn((
        Sprite {
            image: asset_server.load("Tower_outline.png"),
            color: Color::srgba(1.0, 1.0, 1.0, 1.0),
            custom_size: Some(SLIDE_SIZE),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.1),
    ));*/
    
    commands.spawn((
        Sprite {
            image: asset_server.load("MPBR.png"),
            color: Color::srgba(1.0, 1.0, 1.0, 0.9),
            custom_size: Some(SLIDE_SIZE),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.),
    ));
    let base_mass = 1.0;
    let base_inertia = base_mass * ((256.0 * 256.0) + (20.0 * 20.0)) / 12.0;

    for x in [-464.0, 464.0] {
        commands.spawn((
            Block {
                mass: base_mass,
                angular_velocity: 0.0,
                inertia: base_inertia,
                fcoeff: 0.25,
            },
            Velocity(Vec2::ZERO),
            AngularVelocity(0.0),
            StaticBody,
            Transform::from_xyz(x, TOWER_BASE_Y - 10.0, 0.0),
        ));
    }
    camera.translation.x = 0.;
}
