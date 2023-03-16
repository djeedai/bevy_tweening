use bevy::prelude::*;
use bevy_tweening::{lens::*, *};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CustomRelativeLens".to_string(),
                resolution: (1200., 600.).into(),
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            }),
            ..default()
        }))
        .add_system(bevy::window::close_on_esc)
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup)
        .run();

    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let size = 25.;
    let screen_y = 150.;

    let tween = Tween::new(
        EaseFunction::QuadraticInOut,
        std::time::Duration::from_millis(500),
        TransformRelativePositionLens {
            end: Vec3::new(100., -screen_y, 0.),
            ..Default::default()
        },
    );

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(size, size)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Animator::new(tween));
}

#[derive(Default)]
pub struct TransformRelativePositionLens {
    start: Vec3,
    pub end: Vec3,
}

impl Lens<Transform> for TransformRelativePositionLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.start + (self.end - self.start) * ratio;
        target.translation = value;
    }

    fn update_on_tween_start(&mut self, target: &Transform) {
        self.start = target.translation;
    }
}
