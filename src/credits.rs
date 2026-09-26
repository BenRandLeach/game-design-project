use bevy::prelude::*;

use crate::{loading::LoadingAssets, GameState};
const SLIDE_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

#[derive(Component)]
struct CreditSlide;

#[derive(Resource)]
struct SlideShow {
    timer: Timer,
    index: usize,
    slides: Vec<Entity>,
}

#[derive(Message, Default)]
pub struct Credits;

#[derive(Component)]
pub struct CreditsScreen;


pub struct CreditsPlugin;
impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_credits)
            .add_systems(OnEnter(GameState::Credits), setup_credits)
            .add_systems(Update, credits_mess_listener)
            .add_systems(Update, cycle_slides.run_if(in_state(GameState::Credits)))
            .add_message::<Credits>();
    }
}

fn load_credits(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {

}

fn setup_credits(mut commands: Commands, asset_server: Res<AssetServer>) {

    let slide_paths = [
        "DannyGallagherNameSlide.png",
        "image0.png",
        "Ishay_cred.png",
        "Gio_cred.png",
        "PollyNanevaCredits.png",
        "sam_cred.png",
    ];

    let slides: Vec<Entity> = slide_paths
        .iter()
        .enumerate()
        .map(|(i, path)| {
            commands
                .spawn((
                    Sprite {
                        image: asset_server.load(*path),
                        custom_size: Some(SLIDE_SIZE),
                        ..default()
                    },
                    Transform::from_xyz(0., 0., 0.),
                    CreditSlide,
                    if i == 0 {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                ))
                .id()
        })
        .collect();

    commands.insert_resource(SlideShow {
        timer: Timer::from_seconds(3.0, TimerMode::Repeating),
        index: 0,
        slides,
    });
}

fn credits_mess_listener(
    mut credits_mess: MessageReader<Credits>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !credits_mess.is_empty() {
        next_state.set(GameState::Credits);
        credits_mess.clear();
    }
}

fn cycle_slides(
    time: Res<Time>,
    mut show: ResMut<SlideShow>,
    mut visibility: Query<&mut Visibility, With<CreditSlide>>,
) {
    if show.timer.tick(time.delta()).just_finished() {
        let next = (show.index + 1) % show.slides.len();
        if let Ok(mut v) = visibility.get_mut(show.slides[show.index]) {
            *v = Visibility::Hidden;
        }
        if let Ok(mut v) = visibility.get_mut(show.slides[next]) {
            *v = Visibility::Visible;
        }
        show.index = next;
    }
}