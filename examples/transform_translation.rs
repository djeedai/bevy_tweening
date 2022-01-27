use bevy::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "TransformPositionLens".to_string(),
            width: 1400.,
            height: 600.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_tweening::TweeningPlugin)
        .add_startup_system(setup)
        .run();

    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let size = 25.;

    let spacing = 1.5;
    let screen_x = 570.;
    let screen_y = 150.;
    let mut x = -screen_x;

    for ease_function in &[
        bevy_tweening::EaseFunction::QuadraticIn,
        bevy_tweening::EaseFunction::QuadraticOut,
        bevy_tweening::EaseFunction::QuadraticInOut,
        bevy_tweening::EaseFunction::CubicIn,
        bevy_tweening::EaseFunction::CubicOut,
        bevy_tweening::EaseFunction::CubicInOut,
        bevy_tweening::EaseFunction::QuarticIn,
        bevy_tweening::EaseFunction::QuarticOut,
        bevy_tweening::EaseFunction::QuarticInOut,
        bevy_tweening::EaseFunction::QuinticIn,
        bevy_tweening::EaseFunction::QuinticOut,
        bevy_tweening::EaseFunction::QuinticInOut,
        bevy_tweening::EaseFunction::SineIn,
        bevy_tweening::EaseFunction::SineOut,
        bevy_tweening::EaseFunction::SineInOut,
        bevy_tweening::EaseFunction::CircularIn,
        bevy_tweening::EaseFunction::CircularOut,
        bevy_tweening::EaseFunction::CircularInOut,
        bevy_tweening::EaseFunction::ExponentialIn,
        bevy_tweening::EaseFunction::ExponentialOut,
        bevy_tweening::EaseFunction::ExponentialInOut,
        bevy_tweening::EaseFunction::ElasticIn,
        bevy_tweening::EaseFunction::ElasticOut,
        bevy_tweening::EaseFunction::ElasticInOut,
        bevy_tweening::EaseFunction::BackIn,
        bevy_tweening::EaseFunction::BackOut,
        bevy_tweening::EaseFunction::BackInOut,
        bevy_tweening::EaseFunction::BounceIn,
        bevy_tweening::EaseFunction::BounceOut,
        bevy_tweening::EaseFunction::BounceInOut,
    ] {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(size, size)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(bevy_tweening::Animator::new(
                *ease_function,
                bevy_tweening::TweeningType::PingPong {
                    duration: std::time::Duration::from_secs(1),
                    pause: Some(std::time::Duration::from_millis(500)),
                },
                bevy_tweening::TransformPositionLens {
                    start: Vec3::new(x, screen_y, 0.),
                    end: Vec3::new(x, -screen_y, 0.),
                },
            ));
        x += size * spacing;
    }
}
