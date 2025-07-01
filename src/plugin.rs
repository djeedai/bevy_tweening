use bevy::prelude::*;

use crate::{AnimCompleted, TweenAnimator, TweenCompleted};

/// Plugin to add systems related to tweening of common components and assets.
///
/// This plugin adds systems for a predefined set of components and assets, to
/// allow their respective animators to be updated each frame:
/// - [`Transform`]
/// - [`TextColor`]
/// - [`Node`]
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
/// [`Transform`]: https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.Transform.html
/// [`TextColor`]: https://docs.rs/bevy/0.16.0/bevy/text/struct.TextColor.html
/// [`Node`]: https://docs.rs/bevy/0.16.0/bevy/ui/struct.Node.html
/// [`Sprite`]: https://docs.rs/bevy/0.16.0/bevy/sprite/struct.Sprite.html
/// [`ColorMaterial`]: https://docs.rs/bevy/0.16.0/bevy/sprite/struct.ColorMaterial.html
#[derive(Debug, Clone, Copy)]
pub struct TweeningPlugin;

impl Plugin for TweeningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TweenAnimator>()
            .add_event::<TweenCompleted>()
            .add_event::<AnimCompleted>()
            .add_systems(
                Update,
                animator_system.in_set(AnimationSystem::AnimationUpdate),
            );
    }
}

/// Label enum for the systems relating to animations
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemSet)]
pub enum AnimationSystem {
    /// Ticks animations
    AnimationUpdate,
}

/// TODO
pub fn animator_system(world: &mut World) {
    let delta_time = world.resource::<Time>().delta();
    // TODO: Use SystemState to cache all of that...
    world.resource_scope(|world, events: Mut<Events<TweenCompleted>>| {
        world.resource_scope(|world, anim_events: Mut<Events<AnimCompleted>>| {
            world.resource_scope(|world, mut animator: Mut<TweenAnimator>| {
                animator.play(world, delta_time, events, anim_events);
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use std::{
        marker::PhantomData,
        ops::DerefMut,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };

    use bevy::ecs::component::Mutable;

    use crate::{lens::TransformPositionLens, *};

    /// A simple isolated test environment with a [`World`] and a single
    /// [`Entity`] in it.
    struct TestEnv<T: Component> {
        world: World,
        entity: Entity,
        tween_id: TweenId,
        _phantom: PhantomData<T>,
    }

    impl<T: Component + Default> TestEnv<T> {
        /// Create a new test environment containing a single entity with a
        /// [`Transform`], and add the given animator on that same entity.
        pub fn new(tweenable: impl Tweenable + 'static) -> Self {
            let mut world = World::new();
            world.init_resource::<Time>();
            world.init_resource::<Events<TweenCompleted>>();
            world.init_resource::<Events<AnimCompleted>>();
            world.init_resource::<TweenAnimator>();

            let entity = world.spawn(T::default()).id();
            let tween_id = world.resource_scope(|_, mut animator: Mut<'_, TweenAnimator>| {
                animator.add(entity, tweenable)
            });

            Self {
                world,
                entity,
                tween_id,
                _phantom: PhantomData,
            }
        }
    }

    impl<T: Component<Mutability = Mutable>> TestEnv<T> {
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
                time.advance_by(duration);
            }

            // Reset world-related change detection
            self.world.clear_trackers();
            assert!(!self.component_mut().is_changed());

            // Tick system
            system.run((), &mut self.world);

            // Update events after system ticked, in case system emitted some events
            let mut events = self.world.resource_mut::<Events<TweenCompleted>>();
            events.update();
            let mut events = self.world.resource_mut::<Events<AnimCompleted>>();
            events.update();
        }

        /// Get the animator for the component.
        pub fn animator(&self) -> &TweenAnimator {
            self.world.resource::<TweenAnimator>()
        }

        /// Get the component.
        pub fn component_mut(&mut self) -> Mut<T> {
            self.world.get_mut::<T>(self.entity).unwrap()
        }

        /// Get the emitted event count since last tick.
        pub fn event_count(&self) -> usize {
            let events = self.world.resource::<Events<TweenCompleted>>();
            events.get_cursor().len(events)
        }
    }

    #[test]
    fn custom_target_entity() {
        let tween = Tween::new(
            EaseMethod::EaseFunction(EaseFunction::Linear),
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_completed_event(true);
        let mut env = TestEnv::<Transform>::new(tween);
        let mut system = IntoSystem::into_system(plugin::animator_system);
        system.initialize(env.world_mut());

        env.tick(Duration::ZERO, &mut system);
        let transform = env.component_mut();
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));

        env.tick(Duration::from_millis(500), &mut system);
        let transform = env.component_mut();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.5), 1e-5));
    }

    #[test]
    fn change_detect_component() {
        let tween = Tween::new(
            EaseMethod::default(),
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_completed_event(true);

        let mut env = TestEnv::<Transform>::new(tween);

        // After being inserted, components are always considered changed
        let transform = env.component_mut();
        assert!(transform.is_changed());

        let mut system = IntoSystem::into_system(plugin::animator_system);
        system.initialize(env.world_mut());

        env.tick(Duration::ZERO, &mut system);

        let animator = env.animator();
        let anim = animator.get(env.tween_id).unwrap();
        assert_eq!(anim.state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let transform = env.component_mut();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));

        env.tick(Duration::from_millis(500), &mut system);

        assert_eq!(env.event_count(), 0);
        let animator = env.animator();
        let anim = animator.get(env.tween_id).unwrap();
        assert_eq!(anim.state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let transform = env.component_mut();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.5), 1e-5));

        env.tick(Duration::from_millis(500), &mut system);

        // The animation is done now, and was deleted from the animator queue.
        // The final state was still applied before deleting the animation,
        // so the component is changed.

        assert_eq!(env.event_count(), 1);
        let animator = env.animator();
        assert!(animator.get(env.tween_id).is_none()); // done and deleted
        let transform = env.component_mut();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));

        // We can continue to tick as much as we want, this doesn't change anything
        env.tick(Duration::from_millis(100), &mut system);

        assert_eq!(env.event_count(), 0);
        let animator = env.animator();
        assert!(animator.get(env.tween_id).is_none()); // done and deleted
        let transform = env.component_mut();
        assert!(!transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));
    }

    #[derive(Debug, Default, Clone, Copy, Component)]
    struct DummyComponent {
        value: f32,
    }

    /// Test [`Lens`] which only access mutably the target component if `defer`
    /// is `true`.
    struct ConditionalDeferLens {
        pub defer: Arc<AtomicBool>,
    }

    impl Lens<DummyComponent> for ConditionalDeferLens {
        fn lerp(&mut self, mut target: Mut<DummyComponent>, ratio: f32) {
            if self.defer.load(Ordering::SeqCst) {
                target.deref_mut().value += ratio;
            }
        }
    }

    #[test]
    fn change_detect_component_conditional() {
        let defer = Arc::new(AtomicBool::new(false));
        let tween = Tween::new(
            EaseMethod::default(),
            Duration::from_secs(1),
            ConditionalDeferLens {
                defer: Arc::clone(&defer),
            },
        )
        .with_completed_event(true);

        let mut env = TestEnv::<DummyComponent>::new(tween);

        // After being inserted, components are always considered changed
        let component = env.component_mut();
        assert!(component.is_changed());

        let mut system = IntoSystem::into_system(plugin::animator_system);
        system.initialize(env.world_mut());

        assert!(!defer.load(Ordering::SeqCst));

        // Mutation disabled
        env.tick(Duration::ZERO, &mut system);

        let animator = env.animator();
        let anim = animator.get(env.tween_id).unwrap();
        assert_eq!(anim.state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(!component.is_changed());
        assert!(((*component).value - 0.).abs() <= 1e-5);

        // Zero-length tick should not change the component
        env.tick(Duration::from_millis(0), &mut system);

        let animator = env.animator();
        let anim = animator.get(env.tween_id).unwrap();
        assert_eq!(anim.state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(!component.is_changed());
        assert!(((*component).value - 0.).abs() <= 1e-5);

        // New tick, but lens mutation still disabled
        env.tick(Duration::from_millis(200), &mut system);

        let animator = env.animator();
        let anim = animator.get(env.tween_id).unwrap();
        assert_eq!(anim.state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(!component.is_changed());
        assert!(((*component).value - 0.).abs() <= 1e-5);

        // Enable lens mutation
        defer.store(true, Ordering::SeqCst);

        // The current time is already at t=0.2s, so even if we don't increment it, for
        // a tween duration of 1s the ratio is t=0.2, so the lens will actually
        // increment the component's value.
        env.tick(Duration::from_millis(0), &mut system);

        let animator = env.animator();
        let anim = animator.get(env.tween_id).unwrap();
        assert_eq!(anim.state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(component.is_changed());
        assert!(((*component).value - 0.2).abs() <= 1e-5);

        // 0.2s + 0.3s = 0.5s
        // t = 0.5s / 1s = 0.5
        // value += 0.5
        // value == 0.7
        env.tick(Duration::from_millis(300), &mut system);

        let animator = env.animator();
        let anim = animator.get(env.tween_id).unwrap();
        assert_eq!(anim.state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(component.is_changed());
        assert!(((*component).value - 0.7).abs() <= 1e-5);
    }
}
