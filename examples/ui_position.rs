use bevy::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::default()
        .insert_resource(WindowDescriptor {
            title: "UiPositionLens".to_string(),
            width: 1400.,
            height: 600.,
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_tweening::TweeningPlugin)
        .add_startup_system(setup)
        .run();

    Ok(())
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());

    let size = 25.;

    let screen_x = 1400.;
    let screen_y = 600.;
    let offset_x = (screen_x - 30. * size) / 31. + size;
    let mut x = 10.;

    for ease_function in &[
        bevy_tweening::EaseFunction::QuadraticIn,
        bevy_tweening::EaseFunction::QuadraticOut,
        bevy_tweening::EaseFunction::QuadraticInOut,
        bevy_tweening::EaseFunction::CubicIn,
        bevy_tweening::EaseFunction::CubicOut,
        bevy_tweening::EaseFunction::CubicInOut,
        bevy_tweening::EaseFunction::QuarticIn,
        bevy_tweening::EaseFunction::QuarticOut,
        bevy_tweening::EaseFunction::QuarticInOut,
        bevy_tweening::EaseFunction::QuinticIn,
        bevy_tweening::EaseFunction::QuinticOut,
        bevy_tweening::EaseFunction::QuinticInOut,
        bevy_tweening::EaseFunction::SineIn,
        bevy_tweening::EaseFunction::SineOut,
        bevy_tweening::EaseFunction::SineInOut,
        bevy_tweening::EaseFunction::CircularIn,
        bevy_tweening::EaseFunction::CircularOut,
        bevy_tweening::EaseFunction::CircularInOut,
        bevy_tweening::EaseFunction::ExponentialIn,
        bevy_tweening::EaseFunction::ExponentialOut,
        bevy_tweening::EaseFunction::ExponentialInOut,
        bevy_tweening::EaseFunction::ElasticIn,
        bevy_tweening::EaseFunction::ElasticOut,
        bevy_tweening::EaseFunction::ElasticInOut,
        bevy_tweening::EaseFunction::BackIn,
        bevy_tweening::EaseFunction::BackOut,
        bevy_tweening::EaseFunction::BackInOut,
        bevy_tweening::EaseFunction::BounceIn,
        bevy_tweening::EaseFunction::BounceOut,
        bevy_tweening::EaseFunction::BounceInOut,
    ] {
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Px(size), Val::Px(size)),
                    position: Rect {
                        left: Val::Px(x),
                        top: Val::Px(10.),
                        right: Val::Auto,
                        bottom: Val::Auto,
                    },
                    position_type: PositionType::Absolute,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                color: UiColor(Color::RED),
                ..Default::default()
            })
            .insert(bevy_tweening::Animator::new(
                *ease_function,
                bevy_tweening::TweeningType::PingPong {
                    duration: std::time::Duration::from_secs(1),
                    pause: Some(std::time::Duration::from_millis(500)),
                },
                bevy_tweening::UiPositionLens {
                    start: Rect {
                        left: Val::Px(x),
                        top: Val::Px(10.),
                        right: Val::Auto,
                        bottom: Val::Auto,
                    },
                    end: Rect {
                        left: Val::Px(x),
                        top: Val::Px(screen_y - 10. - size),
                        right: Val::Auto,
                        bottom: Val::Auto,
                    },
                },
            ));
        x += offset_x;
    }
}
