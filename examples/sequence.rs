use bevy::{color::palettes::css::*, prelude::*};
use bevy_tweening::{lens::*, *};
use std::time::Duration;

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
struct RedSprite;

#[derive(Component)]
struct BlueSprite;

#[derive(Component)]
struct ProgressValue;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());

    let font = asset_server.load("fonts/FiraMono-Regular.ttf");
    let text_font = TextFont {
        font,
        font_size: 50.0,
        ..default()
    };

    let text_color_red = TextColor(RED.into());
    let text_color_blue = TextColor(BLUE.into());

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
    // Build a sequence from an iterator over a Tweenable (here, a
    // Tracks<Transform>)
    let seq = Sequence::new(dests.windows(2).enumerate().map(|(index, pair)| {
        Tracks::new([
            Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(250),
                TransformRotateZLens {
                    start: 0.,
                    end: 180_f32.to_radians(),
                },
            )
            .with_repeat_count(RepeatCount::Finite(4))
            .with_repeat_strategy(RepeatStrategy::MirroredRepeat),
            Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(1),
                TransformPositionLens {
                    start: pair[0] - center,
                    end: pair[1] - center,
                },
            )
            // Get an event after each segment
            .with_completed_event(index as u64),
        ])
    }));

    commands.spawn((
        Sprite {
            color: RED.into(),
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
        RedSprite,
        Animator::new(seq),
    ));

    // First move from left to right, then rotate around self 180 degrees while
    // scaling size at the same time.
    let tween_move = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_secs(1),
        TransformPositionLens {
            start: Vec3::new(-200., 100., 0.),
            end: Vec3::new(200., 100., 0.),
        },
    )
    .with_completed_event(99); // Get an event once move completed
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
    // Build parallel tracks executing two tweens at the same time: rotate and
    // scale.
    let tracks = Tracks::new([tween_rotate, tween_scale]);
    // Build a sequence from an heterogeneous list of tweenables by casting them
    // manually to a BoxedTweenable: first move, then { rotate + scale }.
    let seq2 = Sequence::new([Box::new(tween_move) as BoxedTweenable<_>, tracks.into()]);

    commands.spawn((
        Sprite {
            color: BLUE.into(),
            custom_size: Some(Vec2::new(size * 3., size)),
            ..default()
        },
        BlueSprite,
        Animator::new(seq2),
    ));
}

fn update_text(
    red_text_children: Single<&Children, With<RedProgress>>,
    blue_text_children: Single<&Children, With<BlueProgress>>,
    mut text_spans: Query<&mut TextSpan, With<ProgressValue>>,
    anim_red: Single<&Animator<Transform>, With<RedSprite>>,
    anim_blue: Single<&Animator<Transform>, With<BlueSprite>>,
    mut query_event: EventReader<TweenCompleted>,
) {
    let progress_red = anim_red.tweenable().progress();

    let progress_blue = anim_blue.tweenable().progress();

    let mut red_text = text_spans.get_mut(red_text_children[1]).unwrap();
    red_text.0 = format!("{:5.1}%", progress_red * 100.);

    let mut blue_text = text_spans.get_mut(blue_text_children[1]).unwrap();
    blue_text.0 = format!("{:5.1}%", progress_blue * 100.);

    for ev in query_event.read() {
        println!(
            "Event: TweenCompleted entity={:?} user_data={}",
            ev.entity, ev.user_data
        );
    }
}
