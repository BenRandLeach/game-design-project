use bevy::{prelude::*, window::PresentMode};

#[derive(Component)]
struct PopupTimer{
    timer: Timer,
    layer: f32,
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Credits".into(),
                resolution: (640, 480).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, show_popup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn(Sprite::from_image(asset_server.load("img/DannyGallagherNameSlide.png")));
    commands.spawn((
        Sprite{
          image: asset_server.load("img/image0.png"),
        custom_size: Some(Vec2::new(1280.0, 720.0)),
    ..default()
    },
        Transform{
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer{
            timer: Timer::from_seconds(3.0, TimerMode::Once),
            layer:1.0,

        },
     
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("img/Ishay_cred.png")),
        Transform{
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer{
            timer: Timer::from_seconds(6.0, TimerMode::Once),
            layer:2.0,

        },
     
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("img/Gio_cred.png")),
        Transform{
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer{
            timer: Timer::from_seconds(9.0, TimerMode::Once),
            layer:3.0,

        },
     
    ));
     commands.spawn((
        Sprite::from_image(asset_server.load("img/PollyNanevaCredits.png")),
        Transform{
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer{
            timer: Timer::from_seconds(12.0, TimerMode::Once),
            layer:4.0,

        },
     
    ));
     commands.spawn((
        Sprite::from_image(asset_server.load("img/sam_cred.png")),
        Transform{
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer{
            timer: Timer::from_seconds(15.0, TimerMode::Once),
            layer:5.0,

        },
     
    ));
   
}

fn show_popup(time: Res<Time>, mut popup: Query<(&mut PopupTimer, &mut Transform)>) {
    for (mut popup, mut transform) in popup.iter_mut() {
        popup.timer.tick(time.delta());
        if popup.timer.just_finished() {
            transform.translation.z = popup.layer;
        }
    }
}