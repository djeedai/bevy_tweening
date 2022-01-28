use bevy::prelude::*;
use bevy_tweening::*;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "Sequence".to_string(),
            width: 600.,
            height: 600.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup)
        .run();

    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let size = 25.;

    let margin = 40.;
    let screen_x = 600.;
    let screen_y = 600.;
    let center = Vec3::new(screen_x / 2., screen_y / 2., 0.);

    let dests = &[
        Vec3::new(margin, margin, 0.),
        Vec3::new(screen_x - margin, margin, 0.),
        Vec3::new(screen_x - margin, screen_y - margin, 0.),
        Vec3::new(margin, screen_y - margin, 0.),
        Vec3::new(margin, margin, 0.),
    ];
    let tweens = dests
        .windows(2)
        .map(|pair| {
            Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once {
                    duration: Duration::from_secs(1),
                },
                TransformPositionLens {
                    start: pair[0] - center,
                    end: pair[1] - center,
                },
            )
        })
        .collect();

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(size, size)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Animator::new_seq(tweens));
}
