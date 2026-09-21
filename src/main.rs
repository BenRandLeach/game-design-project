use bevy::{prelude::*, window::PresentMode};

#[derive(Component)]
struct CreditSlide;

#[derive(Resource)]
struct SlideShow {
    timer: Timer,
    index: usize,
    slides: Vec<Entity>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Credits".into(),
                resolution: (1280, 720).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, cycle_slides)
        .run();
}

// The source images are all different resolutions and aspect ratios, so give
// every slide the same custom size to keep the rotation visually consistent.
const SLIDE_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

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
