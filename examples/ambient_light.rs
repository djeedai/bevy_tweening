//! Example demonstrating resource animation and various transform shortcuts.
//!
//! The example animates the `AmbientLight` resource of Bevy's PBR renderer.
//! This is mostly for example purpose; you probably want to animate some other
//! (custom) resource.

use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::{color::palettes::css::*, core_pipeline::bloom::Bloom, prelude::*};
use bevy_tweening::{lens::*, *};

mod utils;

// Define our own `Lens` to animate the `AmbientLight` resource.
struct AmbientLightBrightnessLens {
    pub start: f32,
    pub end: f32,
}

// Implement the `Lens` trait.
impl Lens<AmbientLight> for AmbientLightBrightnessLens {
    fn lerp(&mut self, mut target: Mut<AmbientLight>, ratio: f32) {
        target.brightness = self.start.lerp(self.end, ratio);
    }
}

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AmbientLightLens".to_string(),
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
) -> Result<(), BevyError> {
    // Some fancy 3D camera with HDR and bloom, to emphasize the change of ambient
    // brightness.
    commands.spawn((
        Camera {
            hdr: true,
            clear_color: Color::BLACK.into(),
            ..default()
        },
        Bloom {
            intensity: 0.2,
            ..default()
        },
        Camera3d::default(),
        Transform::from_xyz(0., -7., 2.).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    // Set some default ambient color, and zero out brightness for now
    ambient_light.color = Color::linear_rgb(1., 1., 1.);
    ambient_light.brightness = 0.0;

    // Some sample "ground" circle, not animated.
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: LIGHT_GREEN.into(),
            ..default()
        })),
        Transform::default(),
    ));

    // Animate the ambient light's brightness between fairly extreme values, for
    // example purpose only (please don't do that).
    let tween = Tween::new(
        EaseFunction::CubicIn,
        Duration::from_secs(2),
        AmbientLightBrightnessLens {
            start: 0.0,   // pitch black
            end: 100000., // ahhhh, my eyes!
        },
    )
    .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);
    commands.spawn_empty().tween_resource::<AmbientLight>(tween);

    // Spawn some animated character-like capsule...
    commands
        .spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.5, 1.))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: PURPLE.into(),
                ..default()
            })),
            Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2))
                .with_translation(Vec3::new(-1., 0., 1.)),
        ))
        // ...moving left and right (the start position is the Transform::translation)...
        .move_to(
            Vec3::new(1., 0., 1.),
            Duration::from_secs(1),
            EaseFunction::CircularInOut,
        )
        .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat)
        // This demonstrates that we can continue using the regular EntityCommands to insert more
        // components on the current entity for example.
        .insert(Name::new("haha"))
        // However, after doing so reborrow() is required because insert() returns &mut
        // EntityCommands, but the animation extensions need it by value.
        .reborrow()
        // ...slightly scaling up and back to normal size (the start scale is the Transform::scale).
        .scale_to(
            Vec3::splat(1.1),
            Duration::from_secs(1),
            EaseFunction::BounceInOut,
        )
        .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);

    Ok(())
}
