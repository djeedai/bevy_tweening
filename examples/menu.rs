use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_tweening::{lens::*, *};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "Menu".to_string(),
            width: 800.,
            height: 400.,
            present_mode: bevy::window::PresentMode::Fifo, // vsync
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TweeningPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .run();

    Ok(())
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());

    let font = asset_server.load("fonts/FiraMono-Regular.ttf");

    let container = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect::all(Val::Px(0.)),
                margin: Rect::all(Val::Px(16.)),
                padding: Rect::all(Val::Px(16.)),
                flex_direction: FlexDirection::ColumnReverse,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..Default::default()
        })
        .insert(Name::new("menu"))
        .id();

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
        commands
            .spawn_bundle(NodeBundle {
                node: Node {
                    size: Vec2::new(300., 80.),
                },
                style: Style {
                    min_size: Size::new(Val::Px(300.), Val::Px(80.)),
                    margin: Rect::all(Val::Px(8.)),
                    padding: Rect::all(Val::Px(8.)),
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                color: UiColor(Color::rgb_u8(162, 226, 95)),
                transform: Transform::from_scale(Vec3::splat(0.01)),
                ..Default::default()
            })
            .insert(Name::new(format!("button:{}", text)))
            .insert(Parent(container))
            .insert(Animator::new(seq))
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::with_section(
                        text.to_string(),
                        TextStyle {
                            font: font.clone(),
                            font_size: 48.0,
                            color: Color::rgb_u8(83, 163, 130),
                        },
                        TextAlignment {
                            vertical: VerticalAlign::Center,
                            horizontal: HorizontalAlign::Center,
                        },
                    ),
                    ..Default::default()
                });
            });
    }
}
