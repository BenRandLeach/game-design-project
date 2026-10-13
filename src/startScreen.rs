use bevy::{prelude::*};

use crate::{GameState, PROGRESS_FRAME, PROGRESS_HEIGHT, PROGRESS_LENGTH};

#[derive(Component)]
struct StartScreenProgressFrame;

#[derive(Component)]
struct StartScreenProgress;

#[derive(Component)]
struct StartScreen;

#[derive(Resource, Deref, DerefMut)]
pub struct StartScreenAssets(pub Vec<UntypedHandle>);

use bevy::color::palettes::css::DARK_BLUE;

use bevy::color::palettes::tailwind::SLATE_300;

use bevy::input_focus::AutoFocus;

use bevy::input_focus::{
    tab_navigation::{TabGroup, TabIndex, TabNavigationPlugin},
    InputFocus,
};

use bevy::text::{EditableText, TextCursorStyle}; 


#[derive(Component)]
struct TextOutput;


const MIN_LOAD_TIME: f32 = 5.;

pub struct StartScreenPlugin;
impl Plugin for StartScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StartScreenAssets(Vec::new()))

            .add_plugins(TabNavigationPlugin)
            .add_systems(OnEnter(GameState::StartScreen), setup_startscreen)
            .add_systems(OnEnter(GameState::StartScreen),spawn_IP_Text)
            .add_systems(Update, text_submission.run_if(in_state(GameState::StartScreen)))
            .add_systems(Update, mouse_button_input.run_if(in_state(GameState::StartScreen)))
            .add_systems(
                OnExit(GameState::StartScreen),
                (
                    despawn_with::<StartScreen>,
                    despawn_with::<StartScreenProgressFrame>,
                    despawn_with::<StartScreenProgress>,
                    free_startscreen_handles,
                ),
            );
    }
}

fn spawn_IP_Text(mut commands: Commands, asset_server: Res<AssetServer>) {

    let root = commands
        .spawn((Node {
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            padding: px(20).all(),
            row_gap: px(16),
            margin: auto().all(),
            ..default()
        },
        StartScreen,
    ))
        .id();

    let text_instructions = commands
        .spawn((
            Text::new("Tab to select the box\nType the host's IP and press Enter to connect\nThe host doesn't need to type anything"),
            TextFont {
                
                font_size: FontSize::Px(25.0),
                ..default()
            },
            BorderColor::from(Color::from(SLATE_300)),
            BackgroundColor(DARK_BLUE.into()),
        ))
        .id();

    let text_input_left = build_input_text(&mut commands, true, 24.0);
   // let text_input_right = build_input_text(&mut commands, false, 24.0);

    let input_container = commands
        .spawn((
            Node {
                column_gap: px(16),
                ..default()
            },
            TabGroup::new(0),
        ))
        .id();

    // Set up a text output to see the result of our text input
    let text_output = commands
        .spawn((
            Node {
                width: px(400),
                border: px(2).all(),
                padding: px(8).all(),
                ..Default::default()
            },
            BorderColor::from(Color::from(SLATE_300)),
            BackgroundColor(DARK_BLUE.into()),
            Text::new(""),
            TextOutput,
            TextLayout {
                linebreak: LineBreak::WordOrCharacter,
                ..default()
            },
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
        ))
        .id();

    commands
        .entity(input_container)
        .add_children(&[text_input_left]);

    commands
        .entity(root)
        .add_children(&[text_instructions, input_container, text_output]);
}

fn build_input_text(commands: &mut Commands, is_left: bool, font_size: f32) -> Entity {
    commands
        .spawn((
            Node {
                border: px(2).all(),
                ..Default::default()
            },
            BorderColor::from(Color::from(SLATE_300)),
            Name::new(if is_left { "Connecting_To: " } else { "Server IP: " }),
            EditableText {
                visible_width: Some(10.),
                allow_newlines: false,
                ..Default::default()
            },
            TextLayout::no_wrap(),
            TextFont {
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextCursorStyle::default(),
            TabIndex(if is_left { 0 } else { 1 }),
            BackgroundColor(DARK_BLUE.into()),
        ))
        .id()
}




fn mouse_button_input(
    buttons: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        next_state.set(GameState::Game);
    }

}

fn setup_startscreen(mut commands: Commands,asset_server: Res<AssetServer>,) {
    commands.spawn((
        Sprite 
                        {image: asset_server.load("Tower_Balance_Start.png"), ..default()},
                        StartScreen,
        Transform::from_xyz(0., 0., 0.),

    ));
}

fn text_submission(
    mut commands : Commands,
    input_focus: Res<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_input: Query<(&mut EditableText, &Name, &TabIndex)>,
    mut text_output: Single<&mut Text, With<TextOutput>>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter)
        && let Some(focused_entity) = input_focus.get()
        && let Ok((mut text_input, name, tab)) = text_input.get_mut(focused_entity)
    {
        let typed = text_input.value().to_string();
        if tab.0 == 0{
            text_output.0 = crate::client::connect(&mut commands, typed.trim());
        }
        else{
            text_output.0 = format!("{:}: {:}", name, typed.trim());
        }
        text_input.clear();
    }
}

fn free_startscreen_handles(mut startscreen_assets: ResMut<StartScreenAssets>) {
    startscreen_assets.clear();
}

pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}