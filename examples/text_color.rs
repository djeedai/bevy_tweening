use bevy::{color::palettes::css::*, prelude::*};
use bevy_tweening::{lens::*, *};

mod utils;

const WIDTH: f32 = 1200.;
const HEIGHT: f32 = 600.;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TextColorLens".to_string(),
                resolution: (WIDTH, HEIGHT).into(),
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, utils::close_on_esc)
        .add_plugins(TweeningPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());

    let font = asset_server.load("fonts/FiraMono-Regular.ttf");

    let size_x = 140.;
    let size_y = 40.;

    let delta_x = WIDTH / 5.;
    let delta_y = HEIGHT / 6.;

    let mut x = 20.;
    let mut y = 20.;
    let mut iy = 0;

    for (ease_function, ease_name) in &[
        (EaseFunction::QuadraticIn, "QuadraticIn"),
        (EaseFunction::QuadraticOut, "QuadraticOut"),
        (EaseFunction::QuadraticInOut, "QuadraticInOut"),
        (EaseFunction::CubicIn, "CubicIn"),
        (EaseFunction::CubicOut, "CubicOut"),
        (EaseFunction::CubicInOut, "CubicInOut"),
        (EaseFunction::QuarticIn, "QuarticIn"),
        (EaseFunction::QuarticOut, "QuarticOut"),
        (EaseFunction::QuarticInOut, "QuarticInOut"),
        (EaseFunction::QuinticIn, "QuinticIn"),
        (EaseFunction::QuinticOut, "QuinticOut"),
        (EaseFunction::QuinticInOut, "QuinticInOut"),
        (EaseFunction::SineIn, "SineIn"),
        (EaseFunction::SineOut, "SineOut"),
        (EaseFunction::SineInOut, "SineInOut"),
        (EaseFunction::CircularIn, "CircularIn"),
        (EaseFunction::CircularOut, "CircularOut"),
        (EaseFunction::CircularInOut, "CircularInOut"),
        (EaseFunction::ExponentialIn, "ExponentialIn"),
        (EaseFunction::ExponentialOut, "ExponentialOut"),
        (EaseFunction::ExponentialInOut, "ExponentialInOut"),
        (EaseFunction::ElasticIn, "ElasticIn"),
        (EaseFunction::ElasticOut, "ElasticOut"),
        (EaseFunction::ElasticInOut, "ElasticInOut"),
        (EaseFunction::BackIn, "BackIn"),
        (EaseFunction::BackOut, "BackOut"),
        (EaseFunction::BackInOut, "BackInOut"),
        (EaseFunction::BounceIn, "BounceIn"),
        (EaseFunction::BounceOut, "BounceOut"),
        (EaseFunction::BounceInOut, "BounceInOut"),
    ] {
        let tween = Tween::new(
            *ease_function,
            std::time::Duration::from_secs(1),
            TextColorLens {
                start: RED.into(),
                end: BLUE.into(),
            },
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.spawn((
            Text::new(*ease_name),
            TextFont {
                font: font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                width: Val::Px(size_x),
                height: Val::Px(size_y),
                left: Val::Px(x),
                top: Val::Px(y),
                right: Val::Auto,
                bottom: Val::Auto,
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Animator::new(tween),
        ));

        y += delta_y;
        iy += 1;
        if iy >= 6 {
            x += delta_x;
            y = 20.;
            iy = 0;
        }
    }
}
