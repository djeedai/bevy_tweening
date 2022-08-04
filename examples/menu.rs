use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_tweening::{lens::*, *};
use std::time::Duration;

fn main() {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "Menu".to_string(),
            width: 800.,
            height: 400.,
            present_mode: bevy::window::PresentMode::Fifo, // vsync
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let font = asset_server.load("fonts/FiraMono-Regular.ttf");

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::all(Val::Px(0.)),
                margin: UiRect::all(Val::Px(16.)),
                padding: UiRect::all(Val::Px(16.)),
                flex_direction: FlexDirection::ColumnReverse,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: UiColor(Color::NONE),
            ..default()
        })
        .insert(Name::new("menu"))
        .with_children(|container| {
            let mut start_time_ms = 0;
            for text in &["Continue", "New Game", "Settings", "Quit"] {
                let delay = Delay::new(Duration::from_millis(start_time_ms));
                start_time_ms += 500;
                let tween_scale = Tween::new(
                    EaseFunction::BounceOut,
                    TweeningType::Once,
                    Duration::from_secs(2),
                    TransformScaleLens {
                        start: Vec3::splat(0.01),
                        end: Vec3::ONE,
                    },
                );
                let seq = delay.then(tween_scale);
                container
                    .spawn_bundle(NodeBundle {
                        node: Node {
                            size: Vec2::new(300., 80.),
                        },
                        style: Style {
                            min_size: Size::new(Val::Px(300.), Val::Px(80.)),
                            margin: UiRect::all(Val::Px(8.)),
                            padding: UiRect::all(Val::Px(8.)),
                            align_content: AlignContent::Center,
                            align_items: AlignItems::Center,
                            align_self: AlignSelf::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        color: UiColor(Color::rgb_u8(162, 226, 95)),
                        transform: Transform::from_scale(Vec3::splat(0.01)),
                        ..default()
                    })
                    .insert(Name::new(format!("button:{}", text)))
                    .insert(Animator::new(seq))
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle {
                            text: Text::from_section(
                                text.to_string(),
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 48.0,
                                    color: Color::rgb_u8(83, 163, 130),
                                },
                            )
                            .with_alignment(TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            }),
                            ..default()
                        });
                    });
            }
        });
}
