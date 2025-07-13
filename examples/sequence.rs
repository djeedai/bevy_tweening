use std::time::Duration;

use bevy::{color::palettes::css::*, ecs::component::Components, prelude::*};
use bevy_tweening::{lens::*, *};

mod utils;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sequence".to_string(),
                resolution: (600., 600.).into(),
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, utils::close_on_esc)
        .add_plugins(TweeningPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update_text)
        .run();
}

#[derive(Component)]
struct RedProgress;

#[derive(Component)]
struct BlueProgress;

#[derive(Component)]
struct RedSprite {
    /// ID of the tween making the red sprite move along a path around the
    /// screen.
    pub path_tween_id: TweenId,
    /// ID of the tween making the red sprite rotate on itself back and forth.
    #[allow(unused)]
    pub rotate_tween_id: TweenId,
}

#[derive(Component)]
struct BlueSprite {
    /// ID of the tween making the blue sprite move along a path, then rotate.
    pub move_and_rotate_tween_id: TweenId,
    /// ID of the tween making the blue sprite scale.
    #[allow(unused)]
    pub scale_tween_id: TweenId,
}

#[derive(Component)]
struct ProgressValue;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    components: &Components,
    mut animator: ResMut<TweenAnimator>,
) -> Result<()> {
    commands.spawn(Camera2d::default());

    let font = asset_server.load("fonts/FiraMono-Regular.ttf");
    let text_font = TextFont {
        font,
        font_size: 50.0,
        ..default()
    };

    let text_color_red = TextColor(RED.into());
    let text_color_blue = TextColor(AQUA.into());

    let justify = JustifyText::Center;

    // Text with the index of the active tween in the sequence
    commands
        .spawn((
            Text2d::default(),
            TextLayout::new_with_justify(justify),
            Transform::from_translation(Vec3::new(0., 40., 0.)),
            RedProgress,
        ))
        .with_children(|children| {
            children.spawn((
                TextSpan::new("progress: "),
                text_font.clone(),
                text_color_red.clone(),
            ));
            children.spawn((
                TextSpan::new("0%"),
                text_font.clone(),
                text_color_red.clone(),
                ProgressValue,
            ));
        });

    // Text with progress of the active tween in the sequence
    commands
        .spawn((
            Text2d::default(),
            TextLayout::new_with_justify(justify),
            Transform::from_translation(Vec3::new(0., -40., 0.)),
            BlueProgress,
        ))
        .with_children(|children| {
            children.spawn((
                TextSpan::new("progress: "),
                text_font.clone(),
                text_color_blue.clone(),
            ));
            children.spawn((
                TextSpan::new("0%"),
                text_font.clone(),
                text_color_blue.clone(),
                ProgressValue,
            ));
        });

    let size = 25.;

    let margin = 40.;
    let screen_x = 600.;
    let screen_y = 600.;
    let center = Vec3::new(screen_x / 2., screen_y / 2., 0.);

    // Run around the window from corner to corner
    let dests = &[
        Vec3::new(margin, margin, 0.),
        Vec3::new(screen_x - margin, margin, 0.),
        Vec3::new(screen_x - margin, screen_y - margin, 0.),
        Vec3::new(margin, screen_y - margin, 0.),
        Vec3::new(margin, margin, 0.),
    ];

    // Red sprite
    {
        let entity = commands
            .spawn(Sprite {
                color: RED.into(),
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            })
            .id();

        // Build a sequence from an iterator over a Tweenable
        let anim_move_along_path = Sequence::new(dests.windows(2).map(|pair| {
            Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(1),
                TransformPositionLens {
                    start: pair[0] - center,
                    end: pair[1] - center,
                },
            )
            // Get an event after each segment
            //.with_completed_event(index as u64),
        }));

        // Rotate over self, forever. This will continue even after the move along path
        // above finished.
        let anim_rotate_back_and_forth = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(250),
            TransformRotateZLens {
                start: 0.,
                end: 180_f32.to_radians(),
            },
        )
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        // Because we want to monitor the progress of the animations, we need to fetch
        // their TweenId. This requires inserting them manually in the TweenAnimator
        // resource, instead of using the extensions of EntityCommands.
        let path_tween_id = animator.add_component(components, entity, anim_move_along_path)?;
        let rotate_tween_id =
            animator.add_component(components, entity, anim_rotate_back_and_forth)?;
        commands.entity(entity).insert(RedSprite {
            path_tween_id,
            rotate_tween_id,
        });
    }

    // Blue sprite
    {
        let entity = commands
            .spawn(Sprite {
                color: AQUA.into(),
                custom_size: Some(Vec2::new(size * 3., size)),
                ..default()
            })
            .id();

        // First move from left to right, then rotate around self 180 degrees while
        // scaling size at the same time.

        // In previous versions of bevy_tweening, this could be accomplished with a
        // Tracks, which allowed to run in parallel animations of different duration.
        // That interface was confusing and had too many corner cases, so was removed.
        //
        // Instead, we have 2 solutions:
        // 1. Insert one sequence which moves then rotates, and another which waits
        //    (Delay tweenable) then starts to scale at the same time the first sequence
        //    starts to rotate. In most cases this is the simplest, but requires
        //    controlling the timings of the animations.
        // 2. Insert a single sequence which moves then {rotates+scales}, using a custom
        //    Lens which can apply both the rotation and scale with a single Tween. This
        //    guarantees perfect timing alignment, and doesn't require knowing the
        //    duration of the first (move) animation. A minor drawback is that we have
        //    to write a custom Lens.
        //
        // Here we show how option 1. is implemented, which is often the simplest.

        let move_duration = Duration::from_secs(1);
        let tween_move = Tween::new(
            EaseFunction::QuadraticInOut,
            move_duration,
            TransformPositionLens {
                start: Vec3::new(-200., 100., 0.),
                end: Vec3::new(200., 100., 0.),
            },
        ); //.with_completed_event(99); // Get an event once move completed

        let tween_delay = Delay::new(move_duration);

        let tween_rotate = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_z(180_f32.to_radians()),
            },
        );

        let tween_scale = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            TransformScaleLens {
                start: Vec3::ONE,
                end: Vec3::splat(2.0),
            },
        );

        // Build a sequence from an heterogeneous list of tweenables by casting them
        // manually to a BoxedTweenable. This is only to demonstrate how it's done; in
        // general prefer using then() as below.
        let seq1 = Sequence::new([Box::new(tween_move) as BoxedTweenable, tween_rotate.into()]);
        let seq2 = tween_delay.then(tween_scale);

        // Because we want to monitor the progress of the animations, we need to fetch
        // their TweenId. This requires inserting them manually in the TweenAnimator
        // resource, instead of using the extensions of EntityCommands.
        let move_and_rotate_tween_id = animator.add_component(components, entity, seq1)?;
        let scale_tween_id = animator.add_component(components, entity, seq2)?;
        commands.entity(entity).insert(BlueSprite {
            move_and_rotate_tween_id,
            scale_tween_id,
        });
    }

    Ok(())
}

fn update_text(
    animator: Res<TweenAnimator>,
    red_text_children: Single<&Children, With<RedProgress>>,
    blue_text_children: Single<&Children, With<BlueProgress>>,
    mut q_textspans: Query<&mut TextSpan, With<ProgressValue>>,
    q_anim_red: Query<&RedSprite>,
    q_anim_blue: Query<&BlueSprite>,
    mut q_event_completed: EventReader<TweenCompletedEvent>,
) {
    let anim_red = q_anim_red.single().unwrap();
    let progress_red = if let Some(anim) = animator.get(anim_red.path_tween_id) {
        anim.tweenable().cycle_fraction()
    } else {
        1.
    };

    let anim_blue = q_anim_blue.single().unwrap();
    let progress_blue = if let Some(anim) = animator.get(anim_blue.move_and_rotate_tween_id) {
        anim.tweenable().cycle_fraction()
    } else {
        1.
    };

    let mut red_text = q_textspans.get_mut(red_text_children[1]).unwrap();
    red_text.0 = format!("{:5.1}%", progress_red * 100.);

    let mut blue_text = q_textspans.get_mut(blue_text_children[1]).unwrap();
    blue_text.0 = format!("{:5.1}%", progress_blue * 100.);

    for ev in q_event_completed.read() {
        println!(
            "Event: TweenCompletedEvent tween_id={:?} target={:?}",
            ev.id, ev.target
        );
    }
}
