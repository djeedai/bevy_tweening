#[cfg(feature = "bevy_asset")]
use bevy::asset::Asset;
use bevy::{ecs::component::Component, prelude::*};

#[cfg(feature = "bevy_asset")]
use crate::{tweenable::AssetTarget, AssetAnimator};
use crate::{tweenable::ComponentTarget, Animator, AnimatorState, TweenCompleted};

/// Plugin to add systems related to tweening of common components and assets.
///
/// This plugin adds systems for a predefined set of components and assets, to
/// allow their respective animators to be updated each frame:
/// - [`Transform`]
/// - [`Text`]
/// - [`Style`]
/// - [`Sprite`]
/// - [`ColorMaterial`]
///
/// This ensures that all predefined lenses work as intended, as well as any
/// custom lens animating the same component or asset type.
///
/// For other components and assets, including custom ones, the relevant system
/// needs to be added manually by the application:
/// - For components, add [`component_animator_system::<T>`] where `T:
///   Component`
/// - For assets, add [`asset_animator_system::<T>`] where `T: Asset`
///
/// This plugin is entirely optional. If you want more control, you can instead
/// add manually the relevant systems for the exact set of components and assets
/// actually animated.
///
/// [`Transform`]: https://docs.rs/bevy/0.8.0/bevy/transform/components/struct.Transform.html
/// [`Text`]: https://docs.rs/bevy/0.8.0/bevy/text/struct.Text.html
/// [`Style`]: https://docs.rs/bevy/0.8.0/bevy/ui/struct.Style.html
/// [`Sprite`]: https://docs.rs/bevy/0.8.0/bevy/sprite/struct.Sprite.html
/// [`ColorMaterial`]: https://docs.rs/bevy/0.8.0/bevy/sprite/struct.ColorMaterial.html
#[derive(Debug, Clone, Copy)]
pub struct TweeningPlugin;

impl Plugin for TweeningPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TweenCompleted>().add_system(
            component_animator_system::<Transform>.label(AnimationSystem::AnimationUpdate),
        );

        #[cfg(feature = "bevy_ui")]
        app.add_system(component_animator_system::<Style>.label(AnimationSystem::AnimationUpdate));

        #[cfg(feature = "bevy_sprite")]
        app.add_system(component_animator_system::<Sprite>.label(AnimationSystem::AnimationUpdate));

        #[cfg(all(feature = "bevy_sprite", feature = "bevy_asset"))]
        app.add_system(
            asset_animator_system::<ColorMaterial>.label(AnimationSystem::AnimationUpdate),
        );

        #[cfg(feature = "bevy_text")]
        app.add_system(component_animator_system::<Text>.label(AnimationSystem::AnimationUpdate));
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
/// This system extracts all components of type `T` with an `Animator<T>`
/// attached to the same entity, and tick the animator to animate the component.
pub fn component_animator_system<T: Component>(
    time: Res<Time>,
    mut query: Query<(Entity, &mut T, &mut Animator<T>)>,
    events: ResMut<Events<TweenCompleted>>,
) {
    let mut events: Mut<Events<TweenCompleted>> = events.into();
    for (entity, target, mut animator) in query.iter_mut() {
        if animator.state != AnimatorState::Paused {
            let speed = animator.speed();
            let mut target = ComponentTarget::new(target);
            animator.tweenable_mut().tick(
                time.delta().mul_f32(speed),
                &mut target,
                entity,
                &mut events,
            );
        }
    }
}

/// Animator system for assets.
///
/// This system ticks all `AssetAnimator<T>` components to animate their
/// associated asset.
///
/// This requires the `bevy_asset` feature (enabled by default).
#[cfg(feature = "bevy_asset")]
pub fn asset_animator_system<T: Asset>(
    time: Res<Time>,
    assets: ResMut<Assets<T>>,
    mut query: Query<(Entity, &mut AssetAnimator<T>)>,
    events: ResMut<Events<TweenCompleted>>,
) {
    let mut events: Mut<Events<TweenCompleted>> = events.into();
    let mut target = AssetTarget::new(assets);
    for (entity, mut animator) in query.iter_mut() {
        if animator.state != AnimatorState::Paused {
            target.handle = animator.handle().clone();
            if !target.is_valid() {
                continue;
            }
            let speed = animator.speed();
            animator.tweenable_mut().tick(
                time.delta().mul_f32(speed),
                &mut target,
                entity,
                &mut events,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::{Events, IntoSystem, System, Transform, World};

    use crate::{lens::TransformPositionLens, *};

    /// A simple isolated test environment with a [`World`] and a single
    /// [`Entity`] in it.
    struct TestEnv {
        world: World,
        entity: Entity,
    }

    impl TestEnv {
        /// Create a new test environment containing a single entity with a
        /// [`Transform`], and add the given animator on that same entity.
        pub fn new<T: Component>(animator: T) -> Self {
            let mut world = World::new();
            world.init_resource::<Events<TweenCompleted>>();

            let mut time = Time::default();
            time.update();
            world.insert_resource(time);

            let entity = world
                .spawn()
                .insert(Transform::default())
                .insert(animator)
                .id();

            Self { world, entity }
        }

        /// Get the test world.
        pub fn world_mut(&mut self) -> &mut World {
            &mut self.world
        }

        /// Tick the test environment, updating the simulation time and ticking
        /// the given system.
        pub fn tick(&mut self, duration: Duration, system: &mut dyn System<In = (), Out = ()>) {
            // Simulate time passing by updating the simulation time resource
            {
                let mut time = self.world.resource_mut::<Time>();
                let last_update = time.last_update().unwrap();
                time.update_with_instant(last_update + duration);
            }

            // Reset world-related change detection
            self.world.clear_trackers();
            assert!(!self.transform().is_changed());

            // Tick system
            system.run((), &mut self.world);

            // Update events after system ticked, in case system emitted some events
            let mut events = self.world.resource_mut::<Events<TweenCompleted>>();
            events.update();
        }

        /// Get the animator for the transform.
        pub fn animator(&self) -> &Animator<Transform> {
            self.world
                .entity(self.entity)
                .get::<Animator<Transform>>()
                .unwrap()
        }

        /// Get the transform component.
        pub fn transform(&mut self) -> Mut<Transform> {
            self.world.get_mut::<Transform>(self.entity).unwrap()
        }

        /// Get the emitted event count since last tick.
        pub fn event_count(&self) -> usize {
            let events = self.world.resource::<Events<TweenCompleted>>();
            events.get_reader().len(&events)
        }
    }

    #[test]
    fn change_detect_component() {
        let tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_completed_event(0);

        let mut env = TestEnv::new(Animator::new(tween));

        // After being inserted, components are always considered changed
        let transform = env.transform();
        assert!(transform.is_changed());

        //fn nit() {}
        //let mut system = IntoSystem::into_system(nit);
        let mut system = IntoSystem::into_system(component_animator_system::<Transform>);
        system.initialize(env.world_mut());

        env.tick(Duration::ZERO, &mut system);

        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 0);
        let transform = env.transform();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));

        env.tick(Duration::from_millis(500), &mut system);

        assert_eq!(env.event_count(), 0);
        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 0);
        let transform = env.transform();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.5), 1e-5));

        env.tick(Duration::from_millis(500), &mut system);

        assert_eq!(env.event_count(), 1);
        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 1);
        let transform = env.transform();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));

        env.tick(Duration::from_millis(100), &mut system);

        assert_eq!(env.event_count(), 0);
        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 1);
        let transform = env.transform();
        assert!(!transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));
    }
}
