use bevy::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "SpriteColorLens".to_string(),
            width: 1200.,
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

    let size = 80.;

    let spacing = 1.25;
    let screen_x = 450.;
    let screen_y = 120.;
    let mut x = -screen_x;
    let mut y = screen_y;

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
                transform: Transform::from_translation(Vec3::new(x, y, 0.)),
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(size, size)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(bevy_tweening::Animator::new(
                *ease_function,
                bevy_tweening::TweeningType::PingPong,
                std::time::Duration::from_secs(1),
                bevy_tweening::SpriteColorLens {
                    start: Color::RED,
                    end: Color::BLUE,
                },
            ));
        y -= size * spacing;
        if y < -screen_y {
            x += size * spacing;
            y = screen_y;
        }
    }
}
