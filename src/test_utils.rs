use std::{marker::PhantomData, time::Duration};

use bevy::{
    ecs::{
        change_detection::DetectChanges as _,
        component::{Component, Mutable},
        entity::Entity,
        event::{Event, Events},
        system::{IntoSystem, System},
        world::{Mut, World},
    },
    math::{Quat, Vec3, Vec4},
    time::Time,
    transform::components::Transform,
};

pub(crate) trait AbsDiffEq: std::fmt::Debug {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool;
    fn delta(a: Self, b: Self) -> Self;
}

impl AbsDiffEq for f32 {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool {
        (a - b).abs() < tol
    }
    fn delta(a: Self, b: Self) -> Self {
        (a - b).abs()
    }
}

impl AbsDiffEq for f64 {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool {
        (a - b).abs() < tol as f64
    }
    fn delta(a: Self, b: Self) -> Self {
        (a - b).abs()
    }
}

impl AbsDiffEq for Vec3 {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool {
        Vec3::abs_diff_eq(a, b, tol)
    }
    fn delta(a: Self, b: Self) -> Self {
        (a - b).abs()
    }
}

impl AbsDiffEq for Quat {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool {
        Quat::abs_diff_eq(a, b, tol)
    }
    fn delta(a: Self, b: Self) -> Self {
        Quat::from_vec4((Vec4::from(a) - Vec4::from(b)).abs())
    }
}

impl AbsDiffEq for Transform {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool {
        a.translation.abs_diff_eq(b.translation, tol)
            && a.rotation.abs_diff_eq(b.rotation, tol)
            && a.scale.abs_diff_eq(b.scale, tol)
    }
    fn delta(a: Self, b: Self) -> Self {
        Transform {
            translation: AbsDiffEq::delta(a.translation, b.translation),
            rotation: AbsDiffEq::delta(a.rotation, b.rotation),
            scale: AbsDiffEq::delta(a.scale, b.scale),
        }
    }
}

/// Assert that two floating-point quantities are approximately equal.
///
/// This macro asserts that the absolute difference between the two first
/// arguments is strictly less than a tolerance factor, which can be explicitly
/// passed as third argument or implicitly defaults to `1e-5`.
///
/// # Usage
///
/// ```
/// let x = 3.500009;
/// assert_approx_eq!(x, 3.5); // default tolerance 1e-5
///
/// let x = 3.509;
/// assert_approx_eq!(x, 3.5, 0.01); // explicit tolerance
/// ```
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr $(,)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                assert!(
                    crate::test_utils::AbsDiffEq::abs_diff_eq(*left_val, *right_val, 1e-5),
                    "assertion failed: expected={:?} actual={:?} delta={:?} tol=1e-5(default)",
                    left_val,
                    right_val,
                    crate::test_utils::AbsDiffEq::delta(*left_val, *right_val),
                );
            }
        }
    };
    ($left:expr, $right:expr, $tol:expr $(,)?) => {
        match (&$left, &$right, &$tol) {
            (left_val, right_val, tol_val) => {
                assert!(
                    crate::test_utils::AbsDiffEq::abs_diff_eq(*left_val, *right_val, *tol_val),
                    "assertion failed: expected={:?} actual={:?} delta={:?} tol={}",
                    left_val,
                    right_val,
                    crate::test_utils::AbsDiffEq::delta(*left_val, *right_val),
                    tol_val
                );
            }
        }
    };
}

pub(crate) use assert_approx_eq;

use crate::{
    AnimCompletedEvent, CycleCompletedEvent, TweenAnim, TweenAnimator, Tweenable,
    WorldTargetExtensions as _,
};

/// A simple isolated test environment with a [`World`] and a single
/// [`Entity`] in it.
pub(crate) struct TestEnv<T: Component> {
    pub world: World,
    pub entity: Entity,
    pub tween_id: Entity,
    system: Box<dyn System<In = (), Out = ()>>,
    _phantom: PhantomData<T>,
}

impl<T: Component<Mutability = Mutable> + Default> TestEnv<T> {
    /// Create a new test environment containing a single entity with a `T`
    /// component, and add the given animator on that same entity.
    pub fn new(tweenable: impl Tweenable + 'static) -> Self {
        let mut world = World::new();
        world.init_resource::<Time>();
        world.init_resource::<Events<CycleCompletedEvent>>();
        world.init_resource::<Events<AnimCompletedEvent>>();
        world.init_resource::<TweenAnimator>();

        let mut system = IntoSystem::into_system(crate::plugin::animator_system);
        system.initialize(&mut world);

        let entity = world.spawn(T::default()).id();
        let tween_id = world.resource_scope(|world, mut animator: Mut<'_, TweenAnimator>| {
            let target = world.get_anim_component_target::<T>(entity).unwrap();
            animator.add_component_target(target, tweenable)
        });

        Self {
            world,
            entity,
            tween_id,
            system: Box::new(system),
            _phantom: PhantomData,
        }
    }
}

impl<T: Component<Mutability = Mutable>> TestEnv<T> {
    /// Get the test world.
    #[allow(unused)]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Tick the test environment, updating the simulation time and executing
    /// the built-in system which calls [`TweenAnimator::step_all()`].
    pub fn step_all(&mut self, duration: Duration) {
        // Simulate time passing by updating the simulation time resource
        {
            let mut time = self.world.resource_mut::<Time>();
            time.advance_by(duration);
        }

        // Reset world-related change detection
        self.world.clear_trackers();
        assert!(!self.component_mut().is_changed());

        // Tick system
        self.system.run((), &mut self.world);

        // Update events after system ticked, in case system emitted some events
        let mut events = self.world.resource_mut::<Events<CycleCompletedEvent>>();
        events.update();
        let mut events = self.world.resource_mut::<Events<AnimCompletedEvent>>();
        events.update();
    }

    /// Get the animator for the component.
    pub fn animator(&self) -> &TweenAnimator {
        self.world.resource::<TweenAnimator>()
    }

    /// Get the animator for the component.
    #[allow(dead_code)]
    pub fn animator_mut(&mut self) -> Mut<TweenAnimator> {
        self.world.resource_mut::<TweenAnimator>()
    }

    /// Get the animation.
    pub fn anim(&self) -> Option<&TweenAnim> {
        self.world.resource::<TweenAnimator>().get(self.tween_id)
    }

    /// Apply a mutating closure to the animation.
    pub fn anim_scope(&mut self, func: impl FnOnce(&mut TweenAnim)) {
        let tween_id = self.tween_id;
        let mut animator = self.world.resource_mut::<TweenAnimator>();
        if let Some(anim) = animator.get_mut(tween_id) {
            func(anim);
        }
    }

    /// Get the component.
    pub fn component(&self) -> &T {
        self.world.get::<T>(self.entity).unwrap()
    }

    /// Get the component.
    pub fn component_mut(&mut self) -> Mut<T> {
        self.world.get_mut::<T>(self.entity).unwrap()
    }

    /// Get the emitted event count since last tick.
    pub fn event_count<E: Event>(&self) -> usize {
        let events = self.world.resource::<Events<E>>();
        events.get_cursor().len(events)
    }
}
