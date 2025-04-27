use bevy::{color::palettes::css::*, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, prelude::*, quick::ResourceInspectorPlugin};

use bevy_tweening::{lens::*, *};

mod utils;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "UiPositionLens".to_string(),
                resolution: (1400., 600.).into(),
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            }),
            ..default()
        }),
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            ResourceInspectorPlugin::<Options>::new(),
            TweeningPlugin,
        ))
        .init_resource::<Options>()
        .register_type::<Options>()
        .add_systems(Update, utils::close_on_esc)
        .add_systems(Startup, setup)
        .add_systems(Update, update_animation_speed)
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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    let size = 25.;

    let screen_x = 1400.;
    let screen_y = 600.;
    let offset_x = (screen_x - 30. * size) / 31. + size;
    let mut x = 10.;

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
            UiPositionLens {
                start: UiRect {
                    left: Val::Px(x),
                    top: Val::Px(10.),
                    right: Val::Auto,
                    bottom: Val::Auto,
                },
                end: UiRect {
                    left: Val::Px(x),
                    top: Val::Px(screen_y - 10. - size),
                    right: Val::Auto,
                    bottom: Val::Auto,
                },
            },
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.spawn((
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                left: Val::Px(x),
                top: Val::Px(10.),
                right: Val::Auto,
                bottom: Val::Auto,
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(RED.into()),
            Animator::new(tween),
        ));

        x += offset_x;
    }
}

fn update_animation_speed(mut animators: Query<&mut Animator<Node>>, options: Res<Options>) {
    if !options.is_changed() {
        return;
    }

    for mut animator in animators.iter_mut() {
        animator.set_speed(options.speed);
    }
}
