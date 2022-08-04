use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_tweening::{lens::*, *};
use std::time::Duration;

fn main() {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "ColorMaterialColorLens".to_string(),
            width: 1200.,
            height: 600.,
            present_mode: bevy::window::PresentMode::Fifo, // vsync
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    let size = 80.;

    let spacing = 1.25;
    let screen_x = 450.;
    let screen_y = 120.;
    let mut x = -screen_x;
    let mut y = screen_y;

    let quad_mesh: Mesh2dHandle = meshes.add(Mesh::from(shape::Quad::default())).into();

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
        let unique_material = materials.add(Color::BLACK.into());

        let tween = Tween::new(
            *ease_function,
            TweeningType::PingPong,
            Duration::from_secs(1),
            ColorMaterialColorLens {
                start: Color::RED,
                end: Color::BLUE,
            },
        );

        commands
            .spawn_bundle(MaterialMesh2dBundle {
                mesh: quad_mesh.clone(),
                transform: Transform::from_translation(Vec3::new(x, y, 0.))
                    .with_scale(Vec3::splat(size)),
                material: unique_material.clone(),
                ..default()
            })
            .insert(AssetAnimator::new(unique_material.clone(), tween));
        y -= size * spacing;
        if y < -screen_y {
            x += size * spacing;
            y = screen_y;
        }
    }
}
