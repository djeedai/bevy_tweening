use bevy::{color::palettes::css::*, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, prelude::*, quick::ResourceInspectorPlugin};

use bevy_tweening::{lens::*, *};

mod utils;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "TransformPositionLens".to_string(),
                    resolution: (1400., 600.).into(),
                    present_mode: bevy::window::PresentMode::Fifo, // vsync
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            //DefaultInspectorConfigPlugin,
            ResourceInspectorPlugin::<Options>::new(),
            TweeningPlugin,
        ))
        .init_resource::<Options>()
        .register_type::<Options>()
        .add_systems(Update, utils::close_on_esc)
        .add_systems(Startup, setup)
        .add_systems(Update, update_animation_speed)
        .add_systems(
            Update,
            (
                component_animator_system::<Transform, TransformTranslation>,
                component_animator_system::<Transform, TransformScale>,
                component_animator_system::<Transform, TransformRotation>,
            ),
        )
        .run();
}

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct Options {
    #[inspector(min = 0.01, max = 100.)]
    speed: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self { speed: 1. }
    }
}

struct TransformTranslation;
struct TransformScale;
struct TransformRotation;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    let size = 25.;
    let screen_y = 150.;

    let translation_tween = Tween::new(
        EaseFunction::QuadraticInOut,
        std::time::Duration::from_secs(1),
        TransformPositionLens {
            start: Vec3::new(0., screen_y, 0.),
            end: Vec3::new(0., -screen_y, 0.),
        },
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);
    let scale_tween = Tween::new(
        EaseFunction::SineInOut,
        std::time::Duration::from_secs_f32(0.5),
        TransformScaleLens {
            start: Vec3::ONE,
            end: Vec2::splat(1.5).extend(1.),
        },
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);
    let rotation_tween = Tween::new(
        EaseFunction::QuarticInOut,
        std::time::Duration::from_secs_f32(0.75),
        TransformRotationLens {
            start: Quat::IDENTITY,
            end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 2.),
        },
    )
    .with_repeat_count(RepeatCount::Infinite)
    .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

    commands.spawn((
        Sprite {
            color: RED.into(),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Animator::new(translation_tween).with_marker::<TransformTranslation>(),
        Animator::new(scale_tween).with_marker::<TransformScale>(),
        Animator::new(rotation_tween).with_marker::<TransformRotation>(),
    ));
}

fn update_animation_speed(options: Res<Options>, mut animators: Query<&mut Animator<Transform>>) {
    if !options.is_changed() {
        return;
    }

    for mut animator in animators.iter_mut() {
        animator.set_speed(options.speed);
    }
}
