use bevy::{asset::Asset, ecs::component::Component, prelude::*};

use crate::{Animator, AnimatorState, AssetAnimator, TweenCompleted};

/// Plugin to add systems related to tweening of common components and assets.
///
/// This plugin adds systems for a predefined set of components and assets, to allow their
/// respective animators to be updated each frame:
/// - [`Transform`]
/// - [`Text`]
/// - [`Style`]
/// - [`Sprite`]
/// - [`ColorMaterial`]
///
/// This ensures that all predefined lenses work as intended, as well as any custom lens
/// animating the same component or asset type.
///
/// For other components and assets, including custom ones, the relevant system needs to be
/// added manually by the application:
/// - For components, add [`component_animator_system::<T>`] where `T: Component`
/// - For assets, add [`asset_animator_system::<T>`] where `T: Asset`
///
/// This plugin is entirely optional. If you want more control, you can instead add manually
/// the relevant systems for the exact set of components and assets actually animated.
///
/// [`Transform`]: https://docs.rs/bevy/0.7.0/bevy/transform/components/struct.Transform.html
/// [`Text`]: https://docs.rs/bevy/0.7.0/bevy/text/struct.Text.html
/// [`Style`]: https://docs.rs/bevy/0.7.0/bevy/ui/struct.Style.html
/// [`Sprite`]: https://docs.rs/bevy/0.7.0/bevy/sprite/struct.Sprite.html
/// [`ColorMaterial`]: https://docs.rs/bevy/0.7.0/bevy/sprite/struct.ColorMaterial.html
#[derive(Debug, Clone, Copy)]
pub struct TweeningPlugin;

impl Plugin for TweeningPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TweenCompleted>().add_system(
            component_animator_system::<Transform>.label(AnimationSystem::AnimationUpdate),
        );

        #[cfg(feature = "bevy_ui")]
        app.add_system(component_animator_system::<Text>.label(AnimationSystem::AnimationUpdate))
            .add_system(component_animator_system::<Style>.label(AnimationSystem::AnimationUpdate));

        #[cfg(feature = "bevy_sprite")]
        app.add_system(component_animator_system::<Sprite>.label(AnimationSystem::AnimationUpdate))
            .add_system(
                asset_animator_system::<ColorMaterial>.label(AnimationSystem::AnimationUpdate),
            );
    }
}

/// Label enum for the systems relating to animations
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemLabel)]
pub enum AnimationSystem {
    /// Ticks animations
    AnimationUpdate,
}

/// Animator system for components.
///
/// This system extracts all components of type `T` with an `Animator<T>` attached to the same entity,
/// and tick the animator to animate the component.
pub fn component_animator_system<T: Component>(
    time: Res<Time>,
    mut query: Query<(Entity, &mut T, &mut Animator<T>)>,
    mut event_writer: EventWriter<TweenCompleted>,
) {
    for (entity, ref mut target, ref mut animator) in query.iter_mut() {
        if animator.state != AnimatorState::Paused {
            animator.tick(time.delta(), target, entity, &mut event_writer);
        }
    }
}

/// Animator system for assets.
///
/// This system ticks all `AssetAnimator<T>` components to animate their associated asset.
pub fn asset_animator_system<T: Asset>(
    time: Res<Time>,
    mut assets: ResMut<Assets<T>>,
    mut query: Query<(Entity, &mut AssetAnimator<T>)>,
    mut event_writer: EventWriter<TweenCompleted>,
) {
    for (entity, ref mut animator) in query.iter_mut() {
        if animator.state != AnimatorState::Paused {
            if let Some(target) = assets.get_mut(animator.handle()) {
                animator.tick(time.delta(), target, entity, &mut event_writer);
            }
        }
    }
}
