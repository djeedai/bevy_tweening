use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_tweening::{lens::*, *};
use std::time::Duration;

const NORMAL_COLOR: Color = Color::rgba(162. / 255., 226. / 255., 95. / 255., 1.);
const HOVER_COLOR: Color = Color::AZURE;
const CLICK_COLOR: Color = Color::ALICE_BLUE;
const TEXT_COLOR: Color = Color::rgba(83. / 255., 163. / 255., 130. / 255., 1.);
const INIT_TRANSITION_DONE: u64 = 1;

/// The menu in this example has two set of animations:
/// one for appearance, one for interaction. Interaction animations
/// are only enabled after appearance animations finished.
///
/// The logic is handled as:
/// 1. Appearance animations send a `TweenComplete` event with
/// `INIT_TRANSITION_DONE` 2. The `enable_interaction_after_initial_animation`
/// system adds a label component `InitTransitionDone` to any button component
/// which completed its appearance animation, to mark it as active.
/// 3. The `interaction` system only queries buttons with a `InitTransitionDone`
/// marker.
fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Menu".to_string(),
                width: 800.,
                height: 400.,
                present_mode: bevy::window::PresentMode::Fifo, // vsync
                ..default()
            },
            ..default()
        }))
        .add_system(bevy::window::close_on_esc)
        .add_system(interaction)
        .add_system(enable_interaction_after_initial_animation)
        .add_plugin(TweeningPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let font = asset_server.load("fonts/FiraMono-Regular.ttf");

    commands
        .spawn((
            NodeBundle {
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
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            Name::new("menu"),
        ))
        .with_children(|container| {
            let mut start_time_ms = 0;
            for (text, label) in [
                ("Continue", ButtonLabel::Continue),
                ("New Game", ButtonLabel::NewGame),
                ("Settings", ButtonLabel::Settings),
                ("Quit", ButtonLabel::Quit),
            ] {
                let tween_scale = Tween::new(
                    EaseFunction::BounceOut,
                    Duration::from_secs(2),
                    TransformScaleLens {
                        start: Vec3::splat(0.01),
                        end: Vec3::ONE,
                    },
                )
                .with_completed_event(INIT_TRANSITION_DONE);

                let animator = if start_time_ms > 0 {
                    let delay = Delay::new(Duration::from_millis(start_time_ms));
                    Animator::new(delay.then(tween_scale))
                } else {
                    Animator::new(tween_scale)
                };

                start_time_ms += 500;
                container
                    .spawn((
                        ButtonBundle {
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
                            background_color: BackgroundColor(NORMAL_COLOR),
                            transform: Transform::from_scale(Vec3::splat(0.01)),
                            ..default()
                        },
                        Name::new(format!("button:{}", text)),
                        animator,
                        label,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                text.to_string(),
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 48.0,
                                    color: TEXT_COLOR,
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

fn enable_interaction_after_initial_animation(
    mut commands: Commands,
    mut reader: EventReader<TweenCompleted>,
) {
    for event in reader.iter() {
        if event.user_data == INIT_TRANSITION_DONE {
            commands.entity(event.entity).insert(InitTransitionDone);
        }
    }
}

#[derive(Component)]
struct InitTransitionDone;

#[derive(Component, Clone, Copy)]
enum ButtonLabel {
    Continue,
    NewGame,
    Settings,
    Quit,
}

fn interaction(
    mut interaction_query: Query<
        (
            &mut Animator<Transform>,
            &Transform,
            &Interaction,
            &mut BackgroundColor,
            &ButtonLabel,
        ),
        (Changed<Interaction>, With<InitTransitionDone>),
    >,
) {
    for (mut animator, transform, interaction, mut color, button_label) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = CLICK_COLOR.into();

                match button_label {
                    ButtonLabel::Continue => {
                        println!("Continue clicked");
                    }
                    ButtonLabel::NewGame => {
                        println!("NewGame clicked");
                    }
                    ButtonLabel::Settings => {
                        println!("Settings clicked");
                    }
                    ButtonLabel::Quit => {
                        println!("Quit clicked");
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVER_COLOR.into();
                animator.set_tweenable(Tween::new(
                    EaseFunction::QuadraticIn,
                    Duration::from_millis(200),
                    TransformScaleLens {
                        start: Vec3::ONE,
                        end: Vec3::splat(1.1),
                    },
                ));
            }
            Interaction::None => {
                *color = NORMAL_COLOR.into();
                let start_scale = transform.scale;

                animator.set_tweenable(Tween::new(
                    EaseFunction::QuadraticIn,
                    Duration::from_millis(200),
                    TransformScaleLens {
                        start: start_scale,
                        end: Vec3::ONE,
                    },
                ));
            }
        }
    }
}
