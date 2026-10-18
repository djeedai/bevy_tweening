use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{AnimCompletedEvent, CycleCompletedEvent, TweenAnim, TweenResolver};

/// Plugin to register the 🍃 Bevy Tweening animation framework.
///
/// This plugin registers the common resources and events used by 🍃 Bevy
/// Tweening as well as the core animation system which steps all pending
/// tweenable animations. That system runs in the
/// [`AnimationSystem::AnimationUpdate`] system set, during the [`Update`]
/// schedule.
///
/// The type parameter `T` selects which Bevy [`Time`] context drives
/// animations:
///
/// | `T`       | Time source          | Affected by `Time<Virtual>::pause()`? |
/// |-----------|----------------------|---------------------------------------|
/// | `()`      | `Time<()>` (virtual) | **Yes** — animations freeze on pause  |
/// | `Real`    | `Time<Real>`         | **No**  — animations run at wall-clock speed |
///
/// The default (`T = ()`) preserves the original behaviour.
///
/// # Examples
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_tweening::*;
///
/// // Virtual time (default) — animations pause when the game pauses.
/// App::default()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(TweeningPlugin::<()>::default())
///     .run();
/// ```
///
/// ```no_run
/// use bevy::prelude::*;
/// use bevy::time::Real;
/// use bevy_tweening::*;
///
/// // Real time — animations keep running even when virtual time is paused.
/// App::default()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(TweeningPlugin::<Real>::default())
///     .run();
/// ```
pub struct TweeningPlugin<T: Default + Send + Sync + 'static = ()> {
    _marker: PhantomData<T>,
}

impl<T: Default + Send + Sync + 'static> Default for TweeningPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

// Manual impls because the derive macros would add unnecessary `T: Trait` bounds.
impl<T: Default + Send + Sync + 'static> std::fmt::Debug for TweeningPlugin<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TweeningPlugin").finish()
    }
}

impl<T: Default + Send + Sync + 'static> Clone for TweeningPlugin<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Default + Send + Sync + 'static> Copy for TweeningPlugin<T> {}

impl<T: Default + Send + Sync + 'static> Plugin for TweeningPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<TweenResolver>()
            .add_message::<CycleCompletedEvent>()
            .add_message::<AnimCompletedEvent>()
            .add_systems(
                Update,
                animator_system::<T>.in_set(AnimationSystem::AnimationUpdate),
            );
    }
}

/// Label enum for the systems relating to animations
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemSet)]
#[non_exhaustive]
pub enum AnimationSystem {
    /// Steps all animations. This executes during the [`Update`] schedule.
    AnimationUpdate,
}

/// Core animation system ticking all queued animations.
///
/// This calls [`TweenAnim::step_all()`] using the delta from [`Time<T>`].
/// With the default `T = ()` this reads virtual time; with `T = Real` it
/// reads wall-clock time.
pub(crate) fn animator_system<T: Default + Send + Sync + 'static>(world: &mut World) {
    let delta_time = world.resource::<Time<T>>().delta();
    TweenAnim::step_all(world, delta_time);
}

#[cfg(test)]
mod tests {
    use std::{
        ops::DerefMut,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };

    use bevy::time::TimePlugin;

    use crate::{lens::TransformPositionLens, test_utils::TestEnv, *};

    #[test]
    fn app() {
        let mut app = App::default();
        app.add_plugins((TimePlugin, TweeningPlugin::<()>::default()));
        app.finish();
        app.update();
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
        .with_cycle_completed_event(true);
        let mut env = TestEnv::<Transform>::new(tween);

        env.step_all(Duration::ZERO);
        let transform = env.component_mut();
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));

        env.step_all(Duration::from_millis(500));
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
        .with_cycle_completed_event(true);

        let mut env = TestEnv::<Transform>::new(tween);

        // After being inserted, components are always considered changed
        let transform = env.component_mut();
        assert!(transform.is_changed());

        env.step_all(Duration::ZERO);

        let anim = env.anim().unwrap();
        assert_eq!(anim.playback_state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let transform = env.component_mut();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));

        env.step_all(Duration::from_millis(500));

        assert_eq!(env.event_count::<CycleCompletedEvent>(), 0);
        let anim = env.anim().unwrap();
        assert_eq!(anim.playback_state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let transform = env.component_mut();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.5), 1e-5));

        env.step_all(Duration::from_millis(500));

        // The animation is done now, and was deleted from the animator queue.
        // The final state was still applied before deleting the animation,
        // so the component is changed.

        assert_eq!(env.event_count::<CycleCompletedEvent>(), 1);
        let anim = env.anim();
        assert!(anim.is_none()); // done and deleted
        let transform = env.component_mut();
        assert!(transform.is_changed());
        assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));

        // We can continue to tick as much as we want, this doesn't change anything
        env.step_all(Duration::from_millis(100));

        assert_eq!(env.event_count::<CycleCompletedEvent>(), 0);
        let anim = env.anim();
        assert!(anim.is_none()); // done and deleted
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
        .with_cycle_completed_event(true);

        let mut env = TestEnv::<DummyComponent>::new(tween);

        // After being inserted, components are always considered changed
        let component = env.component_mut();
        assert!(component.is_changed());

        assert!(!defer.load(Ordering::SeqCst));

        // Mutation disabled
        env.step_all(Duration::ZERO);

        let anim = env.anim().unwrap();
        assert_eq!(anim.playback_state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(!component.is_changed());
        assert!(((*component).value - 0.).abs() <= 1e-5);

        // Zero-length tick should not change the component
        env.step_all(Duration::ZERO);

        let anim = env.anim().unwrap();
        assert_eq!(anim.playback_state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(!component.is_changed());
        assert!(((*component).value - 0.).abs() <= 1e-5);

        // New tick, but lens mutation still disabled
        env.step_all(Duration::from_millis(200));

        let anim = env.anim().unwrap();
        assert_eq!(anim.playback_state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(!component.is_changed());
        assert!(((*component).value - 0.).abs() <= 1e-5);

        // Enable lens mutation
        defer.store(true, Ordering::SeqCst);

        // The current time is already at t=0.2s, so even if we don't increment it, for
        // a tween duration of 1s the ratio is t=0.2, so the lens will actually
        // increment the component's value.
        env.step_all(Duration::ZERO);

        let anim = env.anim().unwrap();
        assert_eq!(anim.playback_state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(component.is_changed());
        assert!(((*component).value - 0.2).abs() <= 1e-5);

        // 0.2s + 0.3s = 0.5s
        // t = 0.5s / 1s = 0.5
        // value += 0.5
        // value == 0.7
        env.step_all(Duration::from_millis(300));

        let anim = env.anim().unwrap();
        assert_eq!(anim.playback_state, PlaybackState::Playing);
        assert_eq!(anim.tweenable.cycles_completed(), 0);
        let component = env.component_mut();
        assert!(component.is_changed());
        assert!(((*component).value - 0.7).abs() <= 1e-5);
    }
}
