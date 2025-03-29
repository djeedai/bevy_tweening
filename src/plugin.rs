use bevy::{ecs::component::Mutable, prelude::*};

#[cfg(feature = "bevy_asset")]
use crate::{tweenable::AssetTarget, AssetAnimator};
use crate::{tweenable::ComponentTarget, Animator, AnimatorState, TweenCompleted};

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
/// [`Transform`]: https://docs.rs/bevy/0.15.0/bevy/transform/components/struct.Transform.html
/// [`TextColor`]: https://docs.rs/bevy/0.15.0/bevy/text/struct.TextColor.html
/// [`Node`]: https://docs.rs/bevy/0.15.0/bevy/ui/struct.Node.html
/// [`Sprite`]: https://docs.rs/bevy/0.15.0/bevy/sprite/struct.Sprite.html
/// [`ColorMaterial`]: https://docs.rs/bevy/0.15.0/bevy/sprite/struct.ColorMaterial.html
#[derive(Debug, Clone, Copy)]
pub struct TweeningPlugin;

impl Plugin for TweeningPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TweenCompleted>().add_systems(
            Update,
            component_animator_system::<Transform>.in_set(AnimationSystem::AnimationUpdate),
        );

        #[cfg(feature = "bevy_ui")]
        app.add_systems(
            Update,
            component_animator_system::<Node>.in_set(AnimationSystem::AnimationUpdate),
        );
        #[cfg(feature = "bevy_ui")]
        app.add_systems(
            Update,
            component_animator_system::<BackgroundColor>.in_set(AnimationSystem::AnimationUpdate),
        );

        #[cfg(feature = "bevy_sprite")]
        app.add_systems(
            Update,
            component_animator_system::<Sprite>.in_set(AnimationSystem::AnimationUpdate),
        );

        #[cfg(all(feature = "bevy_sprite", feature = "bevy_asset"))]
        app.add_systems(
            Update,
            asset_animator_system::<ColorMaterial, MeshMaterial2d<ColorMaterial>>
                .in_set(AnimationSystem::AnimationUpdate),
        );

        #[cfg(feature = "bevy_text")]
        app.add_systems(
            Update,
            component_animator_system::<TextColor>.in_set(AnimationSystem::AnimationUpdate),
        );
    }
}

/// Label enum for the systems relating to animations
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemSet)]
pub enum AnimationSystem {
    /// Ticks animations
    AnimationUpdate,
}

/// Animator system for components.
///
/// This system extracts all components of type `T` with an [`Animator<T>`]
/// attached to the same entity, and tick the animator to animate the component.
pub fn component_animator_system<T: Component<Mutability = Mutable>>(
    time: Res<Time>,
    mut animator_query: Query<(Entity, &mut Animator<T>)>,
    mut target_query: Query<&mut T>,
    events: ResMut<Events<TweenCompleted>>,
    mut commands: Commands,
) {
    let mut events: Mut<Events<TweenCompleted>> = events.into();
    for (animator_entity, mut animator) in animator_query.iter_mut() {
        if animator.state != AnimatorState::Paused {
            let speed = animator.speed();
            let entity = animator.target.unwrap_or(animator_entity);
            let Ok(target) = target_query.get_mut(entity) else {
                continue;
            };
            let mut target = ComponentTarget::new(target);
            animator.tweenable_mut().tick(
                time.delta().mul_f32(speed),
                &mut target,
                entity,
                &mut events,
                &mut commands,
            );
        }
    }
}

#[cfg(feature = "bevy_asset")]
use std::ops::Deref;

/// Animator system for assets.
///
/// This system ticks all [`AssetAnimator<T>`] components to animate their
/// associated asset.
///
/// This requires the `bevy_asset` feature (enabled by default).
#[cfg(feature = "bevy_asset")]
pub fn asset_animator_system<T, M>(
    time: Res<Time>,
    mut assets: ResMut<Assets<T>>,
    mut query: Query<(Entity, &M, &mut AssetAnimator<T>)>,
    events: ResMut<Events<TweenCompleted>>,
    mut commands: Commands,
) where
    T: Asset,
    M: Component + Deref<Target = Handle<T>>,
{
    let mut events: Mut<Events<TweenCompleted>> = events.into();
    let mut target = AssetTarget::new(assets.reborrow());
    for (entity, handle, mut animator) in query.iter_mut() {
        if animator.state != AnimatorState::Paused {
            target.handle = handle.clone_weak();
            if !target.is_valid() {
                continue;
            }
            let speed = animator.speed();
            animator.tweenable_mut().tick(
                time.delta().mul_f32(speed),
                &mut target,
                entity,
                &mut events,
                &mut commands,
            );
        }
    }
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
        animator_entity: Entity,
        target_entity: Option<Entity>,
        _phantom: PhantomData<T>,
    }

    impl<T: Component + Default> TestEnv<T> {
        /// Create a new test environment containing a single entity with a
        /// [`Transform`], and add the given animator on that same entity.
        pub fn new(animator: Animator<T>) -> Self {
            let mut world = World::new();
            world.init_resource::<Events<TweenCompleted>>();
            world.init_resource::<Time>();

            let entity = world.spawn((T::default(), animator)).id();

            Self {
                world,
                animator_entity: entity,
                target_entity: None,
                _phantom: PhantomData,
            }
        }

        /// Like [`TestEnv::new`], but the component is placed on a separate entity.
        pub fn new_separated(animator: Animator<T>) -> Self {
            let mut world = World::new();
            world.init_resource::<Events<TweenCompleted>>();
            world.init_resource::<Time>();

            let target = world.spawn(T::default()).id();
            let entity = world.spawn(animator.with_target(target)).id();

            Self {
                world,
                animator_entity: entity,
                target_entity: Some(target),
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
        }

        /// Get the animator for the component.
        pub fn animator(&self) -> &Animator<T> {
            self.world
                .entity(self.animator_entity)
                .get::<Animator<T>>()
                .unwrap()
        }

        /// Get the component.
        pub fn component_mut(&mut self) -> Mut<T> {
            self.world
                .get_mut::<T>(self.target_entity.unwrap_or(self.animator_entity))
                .unwrap()
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
        .with_completed_event(0);
        let mut env = TestEnv::new_separated(Animator::new(tween));
        let mut system = IntoSystem::into_system(component_animator_system::<Transform>);
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
        .with_completed_event(0);

        let mut env = TestEnv::new(Animator::new(tween));

        // After being inserted, components are always considered changed
        let transform = env.component_mut();
        assert!(transform.is_changed());

        // fn nit() {}
        // let mut system = IntoSystem::into_system(nit);
        let mut system = IntoSystem::into_system(component_animator_system::<Transform>);
        system.initialize(env.world_mut());

        env.tick(Duration::ZERO, &mut system);

        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 0);
        let transform = env.component_mut();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));

        env.tick(Duration::from_millis(500), &mut system);

        assert_eq!(env.event_count(), 0);
        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 0);
        let transform = env.component_mut();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.5), 1e-5));

        env.tick(Duration::from_millis(500), &mut system);

        assert_eq!(env.event_count(), 1);
        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 1);
        let transform = env.component_mut();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));

        env.tick(Duration::from_millis(100), &mut system);

        assert_eq!(env.event_count(), 0);
        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 1);
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
        fn lerp(&mut self, target: &mut dyn Targetable<DummyComponent>, ratio: f32) {
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
        .with_completed_event(0);

        let mut env = TestEnv::new(Animator::new(tween));

        // After being inserted, components are always considered changed
        let component = env.component_mut();
        assert!(component.is_changed());

        let mut system = IntoSystem::into_system(component_animator_system::<DummyComponent>);
        system.initialize(env.world_mut());

        assert!(!defer.load(Ordering::SeqCst));

        // Mutation disabled
        env.tick(Duration::ZERO, &mut system);

        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 0);
        let component = env.component_mut();
        assert!(!component.is_changed());
        assert!((component.value - 0.).abs() <= 1e-5);

        // Zero-length tick should not change the component
        env.tick(Duration::from_millis(0), &mut system);

        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 0);
        let component = env.component_mut();
        assert!(!component.is_changed());
        assert!((component.value - 0.).abs() <= 1e-5);

        // New tick, but lens mutation still disabled
        env.tick(Duration::from_millis(200), &mut system);

        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 0);
        let component = env.component_mut();
        assert!(!component.is_changed());
        assert!((component.value - 0.).abs() <= 1e-5);

        // Enable lens mutation
        defer.store(true, Ordering::SeqCst);

        // The current time is already at t=0.2s, so even if we don't increment it, for
        // a tween duration of 1s the ratio is t=0.2, so the lens will actually
        // increment the component's value.
        env.tick(Duration::from_millis(0), &mut system);

        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 0);
        let component = env.component_mut();
        assert!(component.is_changed());
        assert!((component.value - 0.2).abs() <= 1e-5);

        // 0.2s + 0.3s = 0.5s
        // t = 0.5s / 1s = 0.5
        // value += 0.5
        // value == 0.7
        env.tick(Duration::from_millis(300), &mut system);

        let animator = env.animator();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_eq!(animator.tweenable().times_completed(), 0);
        let component = env.component_mut();
        assert!(component.is_changed());
        assert!((component.value - 0.7).abs() <= 1e-5);
    }
}
