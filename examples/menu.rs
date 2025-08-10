use std::time::Duration;

use bevy::{color::palettes::css::*, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_tweening::{lens::*, *};

mod utils;

const NORMAL_COLOR: Color = Color::srgba(162. / 255., 226. / 255., 95. / 255., 1.);
const HOVER_COLOR: Color = Color::Srgba(AZURE);
const CLICK_COLOR: Color = Color::Srgba(ALICE_BLUE);
const TEXT_COLOR: Color = Color::srgba(83. / 255., 163. / 255., 130. / 255., 1.);

#[derive(Component)]
struct InitialAnimMarker;

/// The menu in this example has two set of animations: one for appearance, one
/// for interaction. Interaction animations are only enabled after appearance
/// animations finished.
///
/// The logic is handled as:
/// 1. Appearance animations send an `AnimCompletedEvent`
/// 2. The `enable_interaction_after_initial_animation()` system adds a
///    `HoverAnim` component to any button component which completed its
///    appearance animation, to mark it as active. This component also contains
///    the entity of the current hover animation being played, if any.
/// 3. The `interaction()` system only queries buttons with a `HoverAnim`
///    component, and override the tweenable animation based on the hover state.
///
/// For simplicity step 2. is handled via an observer. Note that the observer is
/// on the Entity which owns the TweenAnim, and not on the one owning the
/// animated component.
fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Menu".to_string(),
                    resolution: (800., 400.).into(),
                    present_mode: bevy::window::PresentMode::Fifo, // vsync
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            WorldInspectorPlugin::new(),
            TweeningPlugin,
        ))
        .add_systems(Update, utils::close_on_esc)
        .add_systems(Update, interaction)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d::default());

    let font = asset_server.load("fonts/FiraMono-Regular.ttf");

    // The menu "container" node, parent of all menu buttons
    commands
        .spawn((
            Name::new("menu"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.),
                right: Val::Px(0.),
                top: Val::Px(0.),
                bottom: Val::Px(0.),
                margin: UiRect::all(Val::Px(16.)),
                padding: UiRect::all(Val::Px(16.)),
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|container| {
            // The individual menu buttons
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
                .with_cycle_completed_event(true);

                let target = container
                    .spawn((
                        Name::new(format!("button:{}", text)),
                        Button,
                        Node {
                            min_width: Val::Px(300.),
                            min_height: Val::Px(80.),
                            margin: UiRect::all(Val::Px(8.)),
                            padding: UiRect::all(Val::Px(8.)),
                            align_content: AlignContent::Center,
                            align_items: AlignItems::Center,
                            align_self: AlignSelf::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(NORMAL_COLOR),
                        Transform::from_scale(Vec3::splat(0.01)),
                        label,
                        children![(
                            Text::new(text.to_string()),
                            TextFont {
                                font: font.clone(),
                                font_size: 48.0,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                            TextLayout::new_with_justify(JustifyText::Center),
                        )],
                    ))
                    .id();

                let tweenable = if start_time_ms > 0 {
                    let delay = Delay::new(Duration::from_millis(start_time_ms));
                    delay.then(tween_scale).into_boxed()
                } else {
                    tween_scale.into_boxed()
                };
                container
                    .spawn((
                        InitialAnimMarker,
                        TweenAnim::new(tweenable),
                        AnimTarget::component::<Transform>(target),
                    ))
                    .observe(enable_interaction_after_initial_animation);

                start_time_ms += 500;
            }
        });
}

fn enable_interaction_after_initial_animation(
    trigger: Trigger<AnimCompletedEvent>,
    mut commands: Commands,
    q_names: Query<&Name>,
) {
    if let AnimTargetKind::Component {
        entity: target_entity,
    } = &trigger.target
    {
        // Resolve the Entity to a friendly name through the Name component. This is
        // optional, just to make the message nicer.
        let name = q_names
            .get(*target_entity)
            .ok()
            .map(Into::into)
            .unwrap_or(format!("{:?}", target_entity));

        println!("Button on entity {name} completed initial animation, activating...",);

        // Spawn an Entity to hold the animation itself. We add the AnimTarget, which
        // doesn't change, but not yet any TweenAnim since we have no animation to play.
        let anim_entity = commands
            .spawn(AnimTarget::component::<Transform>(*target_entity))
            .id();

        // Add the HoverAnim component which also acts as a marker
        commands
            .entity(*target_entity)
            .insert(HoverAnim(anim_entity));
    }
}

#[derive(Component)]
struct HoverAnim(pub Entity);

#[derive(Component, Clone, Copy)]
enum ButtonLabel {
    Continue,
    NewGame,
    Settings,
    Quit,
}

fn interaction(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Transform,
            &Interaction,
            &mut BackgroundColor,
            &ButtonLabel,
            &HoverAnim,
        ),
        Changed<Interaction>,
    >,
) {
    for (transform, interaction, mut color, button_label, hover_anim) in &mut interaction_query {
        let anim_entity = hover_anim.0;

        match *interaction {
            Interaction::Pressed => {
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
                let tween = Tween::new(
                    EaseFunction::QuadraticIn,
                    Duration::from_millis(200),
                    TransformScaleLens {
                        start: transform.scale,
                        end: Vec3::splat(1.1),
                    },
                );

                // Set the animation by overwriting the TweenAnim component. This way we don't
                // need to check if the previous animation was finished or not (and therefore if
                // the TweenAnim component was deleted or not).
                commands.entity(anim_entity).insert(TweenAnim::new(tween));
            }
            Interaction::None => {
                *color = NORMAL_COLOR.into();
                let tween = Tween::new(
                    EaseFunction::QuadraticIn,
                    Duration::from_millis(200),
                    TransformScaleLens {
                        start: transform.scale,
                        end: Vec3::ONE,
                    },
                );

                // Set the animation by overwriting the TweenAnim component. This way we don't
                // need to check if the previous animation was finished or not (and therefore if
                // the TweenAnim component was deleted or not).
                commands.entity(anim_entity).insert(TweenAnim::new(tween));
            }
        }
    }
}
