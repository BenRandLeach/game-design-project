use bevy::{prelude::*};

use crate::{GameState, PROGRESS_FRAME, PROGRESS_HEIGHT, PROGRESS_LENGTH};

#[derive(Component)]
struct StartScreenProgressFrame;

#[derive(Component)]
struct StartScreenProgress;

#[derive(Resource, Deref, DerefMut)]
pub struct StartScreenAssets(pub Vec<UntypedHandle>);


const MIN_LOAD_TIME: f32 = 5.;

pub struct StartScreenPlugin;
impl Plugin for StartScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StartScreenAssets(Vec::new()))
            .add_systems(OnEnter(GameState::StartScreen), setup_startscreen)
            .add_systems(Update, mouse_button_input.run_if(in_state(GameState::StartScreen)))
            .add_systems(
                OnExit(GameState::StartScreen),
                (
                    despawn_with::<StartScreenProgressFrame>,
                    despawn_with::<StartScreenProgress>,
                    free_startscreen_handles,
                ),
            );
    }
}

fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        next_state.set(GameState::Credits);
    }

}

fn setup_startscreen(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::BLACK, Vec2::ONE),
        Transform {
            scale: Vec3::new(
                PROGRESS_LENGTH + PROGRESS_FRAME,
                PROGRESS_HEIGHT + PROGRESS_FRAME,
                0.,
            ),
            ..default()
        },
        StartScreenProgressFrame,
    ));
}


fn free_startscreen_handles(mut startscreen_assets: ResMut<StartScreenAssets>) {
    startscreen_assets.clear();
}

pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}