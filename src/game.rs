use bevy::prelude::*;

use crate::{loading::LoadingAssets, GameState};
const SLIDE_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

#[derive(Component)]
struct CreditSlide;


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
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let game_texture_handle = asset_server.load("Tower_Balance_Background.png");

    loading_assets.0.push(game_texture_handle.clone().untyped());
    commands.insert_resource(GameScreenImage(game_texture_handle));
}

fn setup_game(
    mut commands: Commands,
    gamescreen_image: Res<GameScreenImage>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    commands.spawn((
        Sprite::from_image(gamescreen_image.0.clone()),
        Transform::from_xyz(0., 0., 0.),
        GameScreen,
    ));

    camera.translation.x = 0.;
}
