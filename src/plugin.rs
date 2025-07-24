use bevy::prelude::*;

use crate::{AnimCompletedEvent, CycleCompletedEvent, TweenAnim, TweenResolver};

/// Plugin to register the [`TweenAnimator`] and the system playing animations.
///
/// This plugin registers the common resources and events used by 🍃 Bevy
/// Tweening as well as the core animation system which executes all pending
/// tweenable animations. That system runs in the
/// [`AnimationSystem::AnimationUpdate`] system set, during the [`Update`]
/// schedule.
#[derive(Debug, Clone, Copy)]
pub struct TweeningPlugin;

impl Plugin for TweeningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TweenResolver>()
            .add_event::<CycleCompletedEvent>()
            .add_event::<AnimCompletedEvent>()
            .add_systems(
                Update,
                animator_system.in_set(AnimationSystem::AnimationUpdate),
            );
    }
}

/// Label enum for the systems relating to animations
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, SystemSet)]
#[non_exhaustive]
pub enum AnimationSystem {
    /// Ticks all animations. This executes during the [`Update`] schedule.
    AnimationUpdate,
}

/// Core animation system ticking all queued animations.
///
/// This calls [`TweenAnim::step_all()`] using a value of the animation timestep
/// `delta_time` equal to [`Time::delta()`].
pub(crate) fn animator_system(world: &mut World) {
    let delta_time = world.resource::<Time>().delta();
    let _ = TweenAnim::step_all(world, delta_time);
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

    use crate::{lens::TransformPositionLens, test_utils::TestEnv, *};

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
        .with_completed_event(true);

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
        .with_completed_event(true);

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
