use std::time::Duration;

use bevy::{color::palettes::css::*, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, prelude::*, quick::ResourceInspectorPlugin};
use bevy_tweening::{lens::*, *};

mod utils;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "ColorMaterialColorLens".to_string(),
                    resolution: bevy::window::WindowResolution::new(1200, 600),
                    present_mode: bevy::window::PresentMode::Fifo, // vsync
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin::default(),
            ResourceInspectorPlugin::<Options>::new(),
        ))
        .init_resource::<Options>()
        .register_type::<Options>()
        .add_systems(Update, utils::close_on_esc)
        .add_plugins(TweeningPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_animation_speed)
        .run();
}

#[derive(Resource, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
struct Options {
    #[inspector(min = 0., max = 100.)]
    speed: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self { speed: 1. }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> Result<(), BevyError> {
    commands.spawn(Camera2d::default());

    let size = 80.;

    let spacing = 1.25;
    let screen_x = 450.;
    let screen_y = 120.;
    let mut x = -screen_x;
    let mut y = screen_y;

    let quad_mesh = meshes.add(Rectangle::new(1., 1.));

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
        // Create a unique material per entity, so that it can be animated
        // without affecting the other entities. Note that we could share
        // that material among multiple entities, and animating the material
        // asset would change the color of all entities using that material.
        let unique_material = materials.add(Color::BLACK);

        let tween = Tween::new(
            *ease_function,
            Duration::from_secs(1),
            ColorMaterialColorLens {
                start: RED.into(),
                end: BLUE.into(),
            },
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        commands.spawn((
            Mesh2d(quad_mesh.clone()),
            MeshMaterial2d(unique_material.clone()),
            Transform::from_translation(Vec3::new(x, y, 0.)).with_scale(Vec3::splat(size)),
            TweenAnim::new(tween),
            AnimTarget::asset(&unique_material),
        ));

        y -= size * spacing;
        if y < -screen_y {
            x += size * spacing;
            y = screen_y;
        }
    }

    Ok(())
}

fn update_animation_speed(options: Res<Options>, mut q_anims: Query<&mut TweenAnim>) {
    if !options.is_changed() {
        return;
    }

    for mut anim in &mut q_anims {
        anim.speed = options.speed;
    }
}
