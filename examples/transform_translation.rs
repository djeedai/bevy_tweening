use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use bevy_tweening::{lens::*, *};

fn main() {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "TransformPositionLens".to_string(),
            width: 1400.,
            height: 600.,
            present_mode: bevy::window::PresentMode::Fifo, // vsync
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(EguiPlugin)
        .add_startup_system(setup)
        .init_resource::<Options>()
        .add_system(options_panel)
        .add_system(update_animation_speed)
        .run();
}

#[derive(Copy, Clone, PartialEq)]
struct Options {
    open: bool,
    speed: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            open: true,
            speed: 1.,
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

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
            TweeningType::PingPong,
            std::time::Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::new(x, screen_y, 0.),
                end: Vec3::new(x, -screen_y, 0.),
            },
        );

        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(size, size)),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Animator::new(tween));

        x += size * spacing;
    }
}

fn options_panel(mut egui_context: ResMut<EguiContext>, mut options: ResMut<Options>) {
    let mut local_options = options.clone();
    egui::Window::new("Options")
        .open(&mut local_options.open)
        .show(egui_context.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Speed modifier");
                ui.add(
                    egui::DragValue::new(&mut local_options.speed)
                        .speed(0.01)
                        .clamp_range(0.01..=100.),
                );
            });
        });

    if local_options != *options {
        *options = local_options;
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
