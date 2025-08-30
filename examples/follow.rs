//! Example demonstrating resource animation and various transform shortcuts.
//!
//! The example animates the `AmbientLight` resource of Bevy's PBR renderer.
//! This is mostly for example purpose; you probably want to animate some other
//! (custom) resource.

use std::time::Duration;

use bevy::{color::palettes::css::*, prelude::*};
use bevy_tweening::{lens::TransformPositionLens, *};

mod utils;

#[derive(Component)]
struct Follower;

#[derive(Debug, Default, Clone, Copy)]
enum AnimOption {
    OverwriteComponent,
    #[default]
    CallSetTweenable,
}

// Simple way to toggle between the two options. Don't do that in real code.
impl std::ops::Not for AnimOption {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::OverwriteComponent => Self::CallSetTweenable,
            Self::CallSetTweenable => Self::OverwriteComponent,
        }
    }
}

#[derive(Default, Component)]
struct Anim {
    pub option: AnimOption,
}

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Follow".to_string(),
                resolution: (1200., 600.).into(),
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, utils::close_on_esc)
        .add_plugins(TweeningPlugin)
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, change_option)
        .add_systems(Update, follow)
        .run();
}

fn make_tween(start: Vec3, end: Vec3) -> Tween {
    let lens = TransformPositionLens { start, end };
    Tween::new(
        // Start fast to catch-up with target, slow down when closer
        EaseFunction::QuadraticOut,
        // Short duration for the appareance to remain snappy
        Duration::from_millis(200),
        lens,
    )
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d::default());

    // Spawn the follower entity
    let entity = commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::new(10., 10.))),
            MeshMaterial2d(materials.add(ColorMaterial {
                color: WHITE.into(),
                ..default()
            })),
            Follower,
        ))
        .id();

    // Spawn the TweenAnim animating the follower. We want to keep overwriting it as
    // the cursor move, so we need to remember its Entity (we use a marker component
    // here).
    commands.spawn((
        // Our marker
        Anim::default(),
        // The animation itself
        TweenAnim::new(make_tween(Vec3::ZERO, Vec3::ZERO))
            // For performance reason, we keep this animation around and overwrite it. Otherwise it
            // will keep being removed and re-inserted.
            .with_destroy_on_completed(false),
        // The target of the animation, here a component on the given entity
        AnimTarget::component::<Transform>(entity),
    ));
}

fn change_option(keyboard: Res<ButtonInput<KeyCode>>, mut q_anim: Single<&mut Anim>) {
    if keyboard.just_pressed(KeyCode::Space) {
        q_anim.option = !q_anim.option;
        println!("Anim option : {:?}", q_anim.option);
    }
}

fn follow(
    mut commands: Commands,
    mut ev: EventReader<CursorMoved>,
    q_follower: Single<&Transform, With<Follower>>,
    q_camera: Single<(&GlobalTransform, &Camera)>,
    mut q_anim: Single<(Entity, &Anim, &mut TweenAnim)>,
) {
    let Some(ev) = ev.read().last() else {
        return;
    };
    let (camera_transform, camera) = *q_camera;

    let target_transform = *q_follower;
    if let Ok(pos) = camera.viewport_to_world_2d(camera_transform, ev.position) {
        if pos != target_transform.translation.truncate() {
            // Note: Do NOT use move_to(), because it spawns a new separate TweenAnim and
            // Entity each time, and we will end up with multiple of them animating the same
            // target but aiming at different end positions, which will produce some visual
            // jittering. We want to overwrite the same TweenAnim again and again with a new
            // target position.

            match q_anim.1.option {
                // Option 1. Use the fact spawning a component overwrites a previous instance.
                AnimOption::OverwriteComponent => {
                    commands.entity(q_anim.0).insert(
                        TweenAnim::new(make_tween(target_transform.translation, pos.extend(0.)))
                            // We again keep that component around even when the animation
                            // completed, both for performance and because we use a Single<> query
                            // which otherwise would fail once the animation completed and the
                            // TweenAnim was auto-destroyed.
                            .with_destroy_on_completed(false),
                    );
                }
                // Option 2. Keep the same component, simply use set_tweenable() to
                // update the Tween with a new one targetting the new cursor position.
                AnimOption::CallSetTweenable => {
                    q_anim
                        .2
                        .set_tweenable(make_tween(target_transform.translation, pos.extend(0.)))
                        .unwrap();
                }
            }
        }
    }
}
