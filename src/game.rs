use bevy::prelude::*;

const SLIDE_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

use crate::{GameState, PROGRESS_FRAME, PROGRESS_HEIGHT, PROGRESS_LENGTH};



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

fn spawn_block(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyB) {
    commands.spawn((
        Sprite {
                        image: asset_server.load("WoodBlock.png"),
                        ..default()
                    },
        Transform::from_xyz(0., 0., 1.),
        
    ));
}

}

fn swap_to_credits(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
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
                        image: asset_server.load("Tower_Balance_Background.png"),
                        custom_size: Some(SLIDE_SIZE),
                        ..default()
                    },
        Transform::from_xyz(0., 0., 0.),
        
    ));

    camera.translation.x = 0.;
}
