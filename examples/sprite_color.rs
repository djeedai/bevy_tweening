use bevy::prelude::*;
use bevy_tweening::{lens::*, *};

fn main() {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "SpriteColorLens".to_string(),
            width: 1200.,
            height: 600.,
            present_mode: bevy::window::PresentMode::Fifo, // vsync
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());

    let size = 80.;

    let spacing = 1.25;
    let screen_x = 450.;
    let screen_y = 120.;
    let mut x = -screen_x;
    let mut y = screen_y;

    for ease_function in &[
        EaseFunction::QuadraticIn,
        EaseFunction::QuadraticOut,
        EaseFunction::QuadraticInOut,
        EaseFunction::CubicIn,
        EaseFunction::CubicOut,
        EaseFunction::CubicInOut,
        EaseFunction::QuarticIn,
        EaseFunction::QuarticOut,
        EaseFunction::QuarticInOut,
        EaseFunction::QuinticIn,
        EaseFunction::QuinticOut,
        EaseFunction::QuinticInOut,
        EaseFunction::SineIn,
        EaseFunction::SineOut,
        EaseFunction::SineInOut,
        EaseFunction::CircularIn,
        EaseFunction::CircularOut,
        EaseFunction::CircularInOut,
        EaseFunction::ExponentialIn,
        EaseFunction::ExponentialOut,
        EaseFunction::ExponentialInOut,
        EaseFunction::ElasticIn,
        EaseFunction::ElasticOut,
        EaseFunction::ElasticInOut,
        EaseFunction::BackIn,
        EaseFunction::BackOut,
        EaseFunction::BackInOut,
        EaseFunction::BounceIn,
        EaseFunction::BounceOut,
        EaseFunction::BounceInOut,
    ] {
        let tween = Tween::new(
            *ease_function,
            TweeningType::PingPong,
            std::time::Duration::from_secs(1),
            SpriteColorLens {
                start: Color::RED,
                end: Color::BLUE,
            },
        );

        commands
            .spawn_bundle(SpriteBundle {
                transform: Transform::from_translation(Vec3::new(x, y, 0.)),
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(size, size)),
                    ..default()
                },
                ..default()
            })
            .insert(Animator::new(tween));

        y -= size * spacing;
        if y < -screen_y {
            x += size * spacing;
            y = screen_y;
        }
    }
}
