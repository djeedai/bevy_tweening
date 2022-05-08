use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

use bevy_tweening::{lens::*, *};

fn main() {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "UiPositionLens".to_string(),
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
    commands.spawn_bundle(UiCameraBundle::default());

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
            TweeningType::PingPong,
            std::time::Duration::from_secs(1),
            UiPositionLens {
                start: Rect {
                    left: Val::Px(x),
                    top: Val::Px(10.),
                    right: Val::Auto,
                    bottom: Val::Auto,
                },
                end: Rect {
                    left: Val::Px(x),
                    top: Val::Px(screen_y - 10. - size),
                    right: Val::Auto,
                    bottom: Val::Auto,
                },
            },
        );

        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(size), Val::Px(size)),
                    position: Rect {
                        left: Val::Px(x),
                        top: Val::Px(10.),
                        right: Val::Auto,
                        bottom: Val::Auto,
                    },
                    position_type: PositionType::Absolute,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                color: UiColor(Color::RED),
                ..Default::default()
            })
            .insert(Animator::new(tween));

        x += offset_x;
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

fn update_animation_speed(options: Res<Options>, mut animators: Query<&mut Animator<Style>>) {
    if !options.is_changed() {
        return;
    }

    for mut animator in animators.iter_mut() {
        animator.set_speed(options.speed);
    }
}
