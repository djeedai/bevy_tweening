use bevy::prelude::*;
use bevy_tweening::*;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "Sequence".to_string(),
            width: 600.,
            height: 600.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_startup_system(setup)
        .add_system(update_text)
        .add_system(update_anim)
        .run();

    Ok(())
}

#[derive(Component)]
struct IndexText;

#[derive(Component)]
struct ProgressText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let font = asset_server.load("fonts/FiraMono-Regular.ttf");
    let text_style = TextStyle {
        font,
        font_size: 50.0,
        color: Color::WHITE,
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
                        value: "index: ".to_owned(),
                        style: text_style.clone(),
                    },
                    TextSection {
                        value: "0".to_owned(),
                        style: text_style.clone(),
                    },
                ],
                alignment: text_alignment,
            },
            transform: Transform::from_translation(Vec3::new(0., 40., 0.)),
            ..Default::default()
        })
        .insert(IndexText);

    // Text with progress of the active tween in the sequence
    commands
        .spawn_bundle(Text2dBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "progress: ".to_owned(),
                        style: text_style.clone(),
                    },
                    TextSection {
                        value: "0%".to_owned(),
                        style: text_style.clone(),
                    },
                ],
                alignment: text_alignment,
            },
            transform: Transform::from_translation(Vec3::new(0., -40., 0.)),
            ..Default::default()
        })
        .insert(ProgressText);

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
    let tweens = dests
        .windows(2)
        .map(|pair| {
            Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::Once,
                Duration::from_secs(1),
                TransformPositionLens {
                    start: pair[0] - center,
                    end: pair[1] - center,
                },
            )
        })
        .collect();

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                custom_size: Some(Vec2::new(size, size)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Animator::new_seq(tweens).with_state(AnimatorState::Paused));
}

fn update_anim(time: Res<Time>, mut q: Query<&mut Animator<Transform>>) {
    if time.seconds_since_startup() >= 10. {
        q.single_mut().state = AnimatorState::Playing;
    }
}

fn update_text(
    // Note: need a QuerySet<> due to the "&mut Text" in both queries
    mut query_text: QuerySet<(
        QueryState<&mut Text, With<IndexText>>,
        QueryState<&mut Text, With<ProgressText>>,
    )>,
    query_anim: Query<&Animator<Transform>>,
) {
    let anim = query_anim.single();
    let seq = &anim.tracks()[0];
    let index = seq.index();
    let tween = seq.current();
    let progress = tween.progress();

    // Use scopes to force-drop the mutable context before opening the next one
    {
        let mut q0 = query_text.q0();
        let mut index_text = q0.single_mut();
        index_text.sections[1].value = format!("{:1}", index).to_string();
    }
    {
        let mut q1 = query_text.q1();
        let mut progress_text = q1.single_mut();
        progress_text.sections[1].value = format!("{:5.1}%", progress * 100.).to_string();
    }
}
