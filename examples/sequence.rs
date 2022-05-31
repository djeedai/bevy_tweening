use bevy::prelude::*;
use bevy_tweening::{lens::*, *};
use std::time::Duration;

fn main() {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "Sequence".to_string(),
            width: 600.,
            height: 600.,
            present_mode: bevy::window::PresentMode::Fifo, // vsync
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup)
        .add_system(update_text)
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let font = asset_server.load("fonts/FiraMono-Regular.ttf");
    let text_style_red = TextStyle {
        font: font.clone(),
        font_size: 50.0,
        color: Color::RED,
    };
    let text_style_blue = TextStyle {
        font,
        font_size: 50.0,
        color: Color::BLUE,
    };

    let text_alignment = TextAlignment {
        vertical: VerticalAlign::Center,
        horizontal: HorizontalAlign::Center,
    };

    // Text with the index of the active tween in the sequence
    commands
        .spawn_bundle(Text2dBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "progress: ".to_owned(),
                        style: text_style_red.clone(),
                    },
                    TextSection {
                        value: "0%".to_owned(),
                        style: text_style_red,
                    },
                ],
                alignment: text_alignment,
            },
            transform: Transform::from_translation(Vec3::new(0., 40., 0.)),
            ..default()
        })
        .insert(RedProgress);

    // Text with progress of the active tween in the sequence
    commands
        .spawn_bundle(Text2dBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "progress: ".to_owned(),
                        style: text_style_blue.clone(),
                    },
                    TextSection {
                        value: "0%".to_owned(),
                        style: text_style_blue,
                    },
                ],
                alignment: text_alignment,
            },
            transform: Transform::from_translation(Vec3::new(0., -40., 0.)),
            ..default()
        })
        .insert(BlueProgress);

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
    // Build a sequence from an iterator over a Tweenable (here, a Tween<Transform>)
    let seq = Sequence::new(dests.windows(2).enumerate().map(|(index, pair)| {
        Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::Once,
            Duration::from_secs(1),
            TransformPositionLens {
                start: pair[0] - center,
                end: pair[1] - center,
            },
        )
        .with_completed_event(true, index as u64) // Get an event after each segment
    }));

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            ..default()
        })
        .insert(RedSprite)
        .insert(Animator::new(seq));

    // First move from left to right, then rotate around self 180 degrees while scaling
    // size at the same time.
    let tween_move = Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::Once,
        Duration::from_secs(1),
        TransformPositionLens {
            start: Vec3::new(-200., 100., 0.),
            end: Vec3::new(200., 100., 0.),
        },
    )
    .with_completed_event(true, 99); // Get an event once move completed
    let tween_rotate = Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::Once,
        Duration::from_secs(1),
        TransformRotationLens {
            start: Quat::IDENTITY,
            end: Quat::from_rotation_z(180_f32.to_radians()),
        },
    );
    let tween_scale = Tween::new(
        EaseFunction::QuadraticInOut,
        TweeningType::Once,
        Duration::from_secs(1),
        TransformScaleLens {
            start: Vec3::ONE,
            end: Vec3::splat(2.0),
        },
    );
    // Build parallel tracks executing two tweens at the same time : rotate and scale.
    let tracks = Tracks::new([tween_rotate, tween_scale]);
    // Build a sequence from an heterogeneous list of tweenables by casting them manually
    // to a boxed Tweenable<Transform> : first move, then { rotate + scale }.
    let seq2 = Sequence::new([Box::new(tween_move) as BoxedTweenable<_>, tracks.into()]);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(size * 3., size)),
                ..default()
            },
            ..Default::default()
        })
        .insert(BlueSprite)
        .insert(Animator::new(seq2));
}

fn update_text(
    mut query_text_red: Query<&mut Text, (With<RedProgress>, Without<BlueProgress>)>,
    mut query_text_blue: Query<&mut Text, (With<BlueProgress>, Without<RedProgress>)>,
    query_anim_red: Query<&Animator<Transform>, With<RedSprite>>,
    query_anim_blue: Query<&Animator<Transform>, With<BlueSprite>>,
    mut query_event: EventReader<TweenCompleted>,
) {
    let anim_red = query_anim_red.single();
    let tween_red = anim_red.tweenable().unwrap();
    let progress_red = tween_red.progress();

    let anim_blue = query_anim_blue.single();
    let tween_blue = anim_blue.tweenable().unwrap();
    let progress_blue = tween_blue.progress();

    let mut red_text = query_text_red.single_mut();
    red_text.sections[1].value = format!("{:5.1}%", progress_red * 100.);

    let mut blue_text = query_text_blue.single_mut();
    blue_text.sections[1].value = format!("{:5.1}%", progress_blue * 100.);

    for ev in query_event.iter() {
        println!(
            "Event: TweenCompleted entity={:?} user_data={}",
            ev.entity, ev.user_data
        );
    }
}
