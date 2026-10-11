use bevy::prelude::*;

const SLIDE_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

use crate::{GameState, PROGRESS_FRAME, PROGRESS_HEIGHT, PROGRESS_LENGTH, WIN_H, WIN_W};
use crate::physics::Velocity;
use crate::physics::resolve_block_collision;
use crate::physics::bound_placement;

const GROUND_Y: f32 = -WIN_H / 2.0;
const LEFT: f32 = -WIN_W / 2.0;
const RIGHT: f32 = WIN_W / 2.0;
const TOWER_BASE_Y: f32 = (-WIN_H / 2.0) + 20.0;
const EDGE_TO_TOWER: f32 = 48.0; // 48px between base and border, 8px is half the width of a block
const BASE_WIDTH: f32 = 256.0;

#[derive(Component)]
pub struct Block;

#[derive(Component)]
pub struct BeingPlaced;

#[derive(Component)]
pub struct JustDropped;

#[derive(Component)]
pub struct NewEntity;

#[derive(Resource, Default)]
pub struct CurrentlyActive(pub Option<Entity>);

#[derive(Component)]
struct Ammo;

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
            .insert_resource(CurrentlyActive::default())
            .add_systems(Update, resolve_block_collision.run_if(in_state(GameState::Game)))
            .add_systems(Update, bound_placement.run_if(in_state(GameState::Game)))
            .add_message::<Game>();
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
    being_placed: Query<Entity, With<BeingPlaced>>,) {
    if being_placed.iter().count() > 0 {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::KeyB) {
    if let Some(active_entity) = currently_active.0 {
            commands.entity(active_entity).remove::<NewEntity>();
        }
    let entity = commands.spawn((
        Sprite {
                        image: asset_server.load("WoodBlock.png"),
                        ..default()
                    },
                    Block,
                    JustDropped,
                    BeingPlaced,
                    Velocity(Vec2::new(2.,0.)),
        Transform::from_xyz(LEFT + EDGE_TO_TOWER + BASE_WIDTH / 2.0, 275., 1.),
)).id();
    currently_active.0 = Some(entity);
    }
}

fn spawn_slingshot(mut commands: Commands,keyboard_input: Res<ButtonInput<KeyCode>>,asset_server: Res<AssetServer>, mut currently_active: ResMut<CurrentlyActive>,) {
    if keyboard_input.just_pressed(KeyCode::KeyN) {
        let slingshot = commands.spawn((Sprite {image: asset_server.load("slingshot.png"), ..default()}, 
                                Block,Velocity::default(), 
                                NewEntity,
                                Transform::from_xyz(0., 0., 2.),)).id();
                                currently_active.0 = Some(slingshot);
        commands.spawn((Sprite {image: asset_server.load("cannonball.png"), ..default()}, 
                                Ammo,Velocity::default(), 
                                NewEntity,
                                Transform::from_xyz(0., 0., 1.),));
    }
}

fn launch_cannonball(mut cannonballs: Query<&mut Velocity, With<Ammo>>, keyboard_input: Res<ButtonInput<KeyCode>>,) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        let mut accel = Vec2::ZERO;

        if keyboard_input.just_pressed(KeyCode::KeyM) {
           let launch_vel = Vec2::new(300.0, 1000.0);

           for mut velocity in &mut cannonballs {
                velocity.0 = launch_vel;
           }
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
                        custom_size: Some(SLIDE_SIZE),
                        ..default()
                    },
        Transform::from_xyz(0., 0., 0.),
        
    ));

    camera.translation.x = 0.;
}
