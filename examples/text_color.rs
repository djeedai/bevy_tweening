use bevy::prelude::*;
use bevy_tweening::{lens::*, *};

const WIDTH: f32 = 1200.;
const HEIGHT: f32 = 600.;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "TextColorLens".to_string(),
                width: WIDTH,
                height: HEIGHT,
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            },
            ..default()
        }))
        .add_system(bevy::window::close_on_esc)
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

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
                start: Color::RED,
                end: Color::BLUE,
                section: 0,
            },
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.spawn((
            TextBundle {
                style: Style {
                    size: Size::new(Val::Px(size_x), Val::Px(size_y)),
                    position: UiRect {
                        left: Val::Px(x),
                        top: Val::Px(y),
                        right: Val::Auto,
                        bottom: Val::Auto,
                    },
                    position_type: PositionType::Absolute,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                text: Text::from_section(
                    *ease_name,
                    TextStyle {
                        font: font.clone(),
                        font_size: 24.0,
                        color: Color::WHITE,
                    },
                ),
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
