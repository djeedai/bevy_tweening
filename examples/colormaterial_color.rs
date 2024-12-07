use bevy::{color::palettes::css::*, prelude::*};
use bevy_tweening::{lens::*, *};
use std::time::Duration;

mod utils;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ColorMaterialColorLens".to_string(),
                resolution: (1200., 600.).into(),
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, utils::close_on_esc)
        .add_plugins(TweeningPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
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
            MeshMaterial2d(unique_material),
            Transform::from_translation(Vec3::new(x, y, 0.)).with_scale(Vec3::splat(size)),
            AssetAnimator::new(tween),
        ));
        y -= size * spacing;
        if y < -screen_y {
            x += size * spacing;
            y = screen_y;
        }
    }
}
