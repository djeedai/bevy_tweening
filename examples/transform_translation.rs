use bevy::prelude::*;
use bevy_inspector_egui::{prelude::*, quick::ResourceInspectorPlugin};

use bevy_tweening::{lens::*, *};

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "TransformPositionLens".to_string(),
                resolution: (1400., 600.).into(),
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Options>()
        .add_systems(Update, bevy::window::close_on_esc)
        .add_plugins(TweeningPlugin)
        .add_plugins(ResourceInspectorPlugin::<Options>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_animation_speed)
        .run();
}

#[derive(Copy, Clone, PartialEq, Resource, Reflect, InspectorOptions)]
struct Options {
    #[inspector(min = 0.01, max = 100.)]
    speed: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self { speed: 1. }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let size = 25.;

    let spacing = 1.5;
    let screen_x = 570.;
    let screen_y = 150.;
    let mut x = -screen_x;

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
            std::time::Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::new(x, screen_y, 0.),
                end: Vec3::new(x, -screen_y, 0.),
            },
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(size, size)),
                    ..default()
                },
                ..default()
            },
            Animator::new(tween),
        ));

        x += size * spacing;
    }
}

fn update_animation_speed(options: Res<Options>, mut animators: Query<&mut Animator<Transform>>) {
    if !options.is_changed() {
        return;
    }

    for mut animator in animators.iter_mut() {
        animator.set_speed(options.speed);
    }
}
