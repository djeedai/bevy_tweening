#![deny(
    warnings,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    missing_docs
)]

//! Tweening animation plugin for the Bevy game engine
//!
//! 🍃 Bevy Tweening provides interpolation-based animation between ("tweening")
//! two values, for Bevy components and assets. Each field of a component or
//! asset can be animated via a collection or predefined easing functions,
//! or providing a custom animation curve. Custom components and assets are also
//! supported.
//!
//! # Example
//!
//! Add the tweening plugin to your app:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_tweening::*;
//!
//! App::default()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(TweeningPlugin)
//!     .run();
//! ```
//!
//! Animate the position ([`Transform::translation`]) of an [`Entity`]:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_tweening::{lens::*, *};
//! # use std::time::Duration;
//! # fn system(mut commands: Commands) {
//! # let size = 16.;
//! // Create a single animation (tween) to move an entity.
//! let tween = Tween::new(
//!     // Use a quadratic easing on both endpoints.
//!     EaseFunction::QuadraticInOut,
//!     // Loop animation back and forth.
//!     TweeningType::PingPong,
//!     // Animation time (one way only; for ping-pong it takes 2 seconds
//!     // to come back to start).
//!     Duration::from_secs(1),
//!     // The lens gives access to the Transform component of the Entity,
//!     // for the Animator to animate it. It also contains the start and
//!     // end values respectively associated with the progress ratios 0. and 1.
//!     TransformPositionLens {
//!         start: Vec3::ZERO,
//!         end: Vec3::new(1., 2., -4.),
//!     },
//! );
//!
//! commands
//!     // Spawn an entity to animate the position of.
//!     .spawn_bundle(TransformBundle::default())
//!     // Add an Animator component to control and execute the animation.
//!     .insert(Animator::new(tween));
//! # }
//! ```
//!
//! # Tweenables
//!
//! 🍃 Bevy Tweening supports several types of _tweenables_, building blocks
//! that can be combined to form complex animations. A tweenable is a type
//! implementing the [`Tweenable`] trait.
//!
//! - [`Tween`] - A simple tween (easing) animation between two values.
//! - [`Sequence`] - A series of tweenables executing in series, one after the
//!   other.
//! - [`Tracks`] - A collection of tweenables executing in parallel.
//! - [`Delay`] - A time delay.
//!
//! ## Chaining animations
//!
//! Most tweenables can be chained with the `then()` operator to produce a
//! [`Sequence`] tweenable:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_tweening::{lens::*, *};
//! # use std::time::Duration;
//! let tween1 = Tween::new(
//!     // [...]
//! #    EaseFunction::BounceOut,
//! #    TweeningType::Once,
//! #    Duration::from_secs(2),
//! #    TransformScaleLens {
//! #        start: Vec3::ZERO,
//! #        end: Vec3::ONE,
//! #    },
//! );
//! let tween2 = Tween::new(
//!     // [...]
//! #    EaseFunction::QuadraticInOut,
//! #    TweeningType::Once,
//! #    Duration::from_secs(1),
//! #    TransformPositionLens {
//! #        start: Vec3::ZERO,
//! #        end: Vec3::new(3.5, 0., 0.),
//! #    },
//! );
//! // Produce a Sequence executing first 'tween1' then 'tween2'
//! let seq = tween1.then(tween2);
//! ```
//!
//! # Animators and lenses
//!
//! Bevy components and assets are animated with tweening _animator_ components,
//! which take a tweenable and apply it to another component on the same
//! [`Entity`]. Those animators determine that other component and its fields to
//! animate using a _lens_.
//!
//! ## Components animation
//!
//! Components are animated with the [`Animator`] component, which is generic
//! over the type of component it animates. This is a restriction imposed by
//! Bevy, to access the animated component as a mutable reference via a
//! [`Query`] and comply with the ECS rules.
//!
//! The [`Animator`] itself is not generic over the subset of fields of the
//! components it animates. This limits the proliferation of generic types when
//! animating e.g. both the position and rotation of an entity.
//!
//! ## Assets animation
//!
//! Assets are animated in a similar way to component, via the [`AssetAnimator`]
//! component.
//!
//! Because assets are typically shared, and the animation applies to the asset
//! itself, all users of the asset see the animation. For example, animating the
//! color of a [`ColorMaterial`] will change the color of all the
//! 2D meshes using that material.
//!
//! ## Lenses
//!
//! Both [`Animator`] and [`AssetAnimator`] access the field(s) to animate via a
//! lens, a type that implements the [`Lens`] trait.
//!
//! Several predefined lenses are provided in the [`lens`] module for the most
//! commonly animated fields, like the components of a [`Transform`]. A custom
//! lens can also be created by implementing the trait, allowing to animate
//! virtually any field of any Bevy component or asset.
//!
//! [`Transform::translation`]: https://docs.rs/bevy/0.7.0/bevy/transform/components/struct.Transform.html#structfield.translation
//! [`Entity`]: https://docs.rs/bevy/0.7.0/bevy/ecs/entity/struct.Entity.html
//! [`Query`]: https://docs.rs/bevy/0.7.0/bevy/ecs/system/struct.Query.html
//! [`ColorMaterial`]: https://docs.rs/bevy/0.7.0/bevy/sprite/struct.ColorMaterial.html
//! [`Sprite`]: https://docs.rs/bevy/0.7.0/bevy/sprite/struct.Sprite.html
//! [`Transform`]: https://docs.rs/bevy/0.7.0/bevy/transform/components/struct.Transform.html

use bevy::{asset::Asset, prelude::*};
use std::time::Duration;

use interpolation::Ease as IEase;
pub use interpolation::{EaseFunction, Lerp};

pub mod lens;
mod plugin;
mod tweenable;

pub use lens::Lens;
pub use plugin::{
    asset_animator_system, component_animator_system, AnimationSystem, TweeningPlugin,
};
pub use tweenable::{
    BoxedTweenable, Delay, Sequence, Tracks, Tween, TweenCompleted, TweenState, Tweenable,
};

/// Type of looping for a tween animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweeningType {
    /// Run the animation once from start to end only.
    Once,
    /// Loop the animation indefinitely, restarting from the start each time the
    /// end is reached.
    Loop,
    /// Loop the animation back and forth, changing direction each time an
    /// endpoint is reached. A complete cycle start -> end -> start always
    /// counts as 2 loop iterations for the various operations where looping
    /// matters.
    PingPong,
}

impl Default for TweeningType {
    fn default() -> Self {
        Self::Once
    }
}

/// Playback state of an animator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatorState {
    /// The animation is playing. This is the default state.
    Playing,
    /// The animation is paused in its current state.
    Paused,
}

impl Default for AnimatorState {
    fn default() -> Self {
        Self::Playing
    }
}

impl std::ops::Not for AnimatorState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Paused => Self::Playing,
            Self::Playing => Self::Paused,
        }
    }
}

/// Describe how eased value should be computed.
#[derive(Clone, Copy)]
pub enum EaseMethod {
    /// Follow `EaseFunction`.
    EaseFunction(EaseFunction),
    /// Linear interpolation, with no function.
    Linear,
    /// Discrete interpolation, eased value will jump from start to end when
    /// stepping over the discrete limit.
    Discrete(f32),
    /// Use a custom function to interpolate the value.
    CustomFunction(fn(f32) -> f32),
}

impl EaseMethod {
    #[must_use]
    fn sample(self, x: f32) -> f32 {
        match self {
            Self::EaseFunction(function) => x.calc(function),
            Self::Linear => x,
            Self::Discrete(limit) => {
                if x > limit {
                    1.
                } else {
                    0.
                }
            }
            Self::CustomFunction(function) => function(x),
        }
    }
}

impl Default for EaseMethod {
    fn default() -> Self {
        Self::Linear
    }
}

impl From<EaseFunction> for EaseMethod {
    fn from(ease_function: EaseFunction) -> Self {
        Self::EaseFunction(ease_function)
    }
}

/// Direction a tweening animation is playing.
///
/// When playing a tweenable forward, the progress values `0` and `1` are
/// respectively mapped to the start and end bounds of the lens(es) being used.
/// Conversely, when playing backward, this mapping is reversed, such that a
/// progress value of `0` corresponds to the state of the target at the end
/// bound of the lens, while a progress value of `1` corresponds to the state of
/// that target at the start bound of the lens, effectively making the animation
/// play backward.
///
/// For all but [`TweeningType::PingPong`] this is always
/// [`TweeningDirection::Forward`], unless manually configured with
/// [`Tween::set_direction()`] in which case the value is constant equal
/// to the value set. For the [`TweeningType::PingPong`] tweening type, this is
/// either forward (from start to end; ping) or backward (from end to start;
/// pong), depending on the current iteration of the loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweeningDirection {
    /// Animation playing from start to end.
    Forward,
    /// Animation playing from end to start, in reverse.
    Backward,
}

impl TweeningDirection {
    /// Is the direction equal to [`TweeningDirection::Forward`]?
    #[must_use]
    pub fn is_forward(&self) -> bool {
        *self == Self::Forward
    }

    /// Is the direction equal to [`TweeningDirection::Backward`]?
    #[must_use]
    pub fn is_backward(&self) -> bool {
        *self == Self::Backward
    }
}

impl Default for TweeningDirection {
    fn default() -> Self {
        Self::Forward
    }
}

impl std::ops::Not for TweeningDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

macro_rules! animator_impl {
    () => {
        /// Set the initial playback state of the animator.
        #[must_use]
        pub fn with_state(mut self, state: AnimatorState) -> Self {
            self.state = state;
            self
        }

        /// Set the initial speed of the animator. See [`Animator::set_speed`] for
        /// details.
        #[must_use]
        pub fn with_speed(mut self, speed: f32) -> Self {
            self.speed = speed;
            self
        }

        /// Set the animation speed. Defaults to 1.
        ///
        /// A speed of 2 means the animation will run twice as fast while a speed of 0.1
        /// will result in a 10x slowed animation.
        pub fn set_speed(&mut self, speed: f32) {
            self.speed = speed;
        }

        /// Set the top-level tweenable item this animator controls.
        pub fn set_tweenable(&mut self, tween: impl Tweenable<T> + Send + Sync + 'static) {
            self.tweenable = Some(Box::new(tween));
        }

        /// Get the top-level tweenable this animator is currently controlling.
        #[must_use]
        pub fn tweenable(&self) -> Option<&(dyn Tweenable<T> + Send + Sync + 'static)> {
            if let Some(tweenable) = &self.tweenable {
                Some(tweenable.as_ref())
            } else {
                None
            }
        }

        /// Get the top-level mutable tweenable this animator is currently controlling.
        #[must_use]
        pub fn tweenable_mut(&mut self) -> Option<&mut (dyn Tweenable<T> + Send + Sync + 'static)> {
            if let Some(tweenable) = &mut self.tweenable {
                Some(tweenable.as_mut())
            } else {
                None
            }
        }

        /// Set the current animation playback progress.
        ///
        /// See [`progress()`] for details on the meaning.
        ///
        /// [`progress()`]: Animator::progress
        pub fn set_progress(&mut self, progress: f32) {
            if let Some(tweenable) = &mut self.tweenable {
                tweenable.set_progress(progress)
            }
        }

        /// Get the current progress in \[0:1\] (non-looping) or \[0:1\[ (looping) of
        /// the animation.
        ///
        /// For looping animations, this reports the progress of the current iteration,
        /// in the current direction:
        /// - [`TweeningType::Loop`] is 0 at start and 1 at end. The exact value 1.0 is
        ///   never reached, since the tweenable loops over to 0.0 immediately.
        /// - [`TweeningType::PingPong`] is 0 at the source endpoint and 1 and the
        ///   destination one, which are respectively the start/end for
        ///   [`TweeningDirection::Forward`], or the end/start for
        ///   [`TweeningDirection::Backward`]. The exact value 1.0 is never reached,
        ///   since the tweenable loops over to 0.0 immediately when it changes
        ///   direction at either endpoint.
        ///
        /// For sequences, the progress is measured over the entire sequence, from 0 at
        /// the start of the first child tweenable to 1 at the end of the last one.
        ///
        /// For tracks (parallel execution), the progress is measured like a sequence
        /// over the longest "path" of child tweenables. In other words, this is the
        /// current elapsed time over the total tweenable duration.
        #[must_use]
        pub fn progress(&self) -> f32 {
            if let Some(tweenable) = &self.tweenable {
                tweenable.progress()
            } else {
                0.
            }
        }

        /// Ticks the tween, if present. See [`Tweenable::tick`] for details.
        pub fn tick(
            &mut self,
            delta: Duration,
            target: &mut T,
            entity: Entity,
            event_writer: &mut EventWriter<TweenCompleted>,
        ) -> Option<TweenState> {
            if let Some(tweenable) = &mut self.tweenable {
                Some(tweenable.tick(delta.mul_f32(self.speed), target, entity, event_writer))
            } else {
                None
            }
        }

        /// Stop animation playback and rewind the animation.
        ///
        /// This changes the animator state to [`AnimatorState::Paused`] and rewind its
        /// tweenable.
        pub fn stop(&mut self) {
            self.state = AnimatorState::Paused;
            self.rewind();
        }

        /// Rewind animation playback to its initial state.
        ///
        /// This does not change the playback state (playing/paused).
        pub fn rewind(&mut self) {
            if let Some(tweenable) = &mut self.tweenable {
                tweenable.rewind();
            }
        }
    };
}

/// Component to control the animation of another component.
#[derive(Component)]
pub struct Animator<T: Component> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
    tweenable: Option<BoxedTweenable<T>>,
    speed: f32,
}

impl<T: Component + std::fmt::Debug> std::fmt::Debug for Animator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Animator")
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Component> Default for Animator<T> {
    fn default() -> Self {
        Self {
            state: default(),
            tweenable: None,
            speed: 1.,
        }
    }
}

impl<T: Component> Animator<T> {
    /// Create a new animator component from a single tweenable.
    #[must_use]
    pub fn new(tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        Self {
            tweenable: Some(Box::new(tween)),
            ..default()
        }
    }

    animator_impl!();
}

/// Component to control the animation of an asset.
#[derive(Component)]
pub struct AssetAnimator<T: Asset> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
    tweenable: Option<BoxedTweenable<T>>,
    handle: Handle<T>,
    speed: f32,
}

impl<T: Asset + std::fmt::Debug> std::fmt::Debug for AssetAnimator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetAnimator")
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Asset> Default for AssetAnimator<T> {
    fn default() -> Self {
        Self {
            state: default(),
            tweenable: None,
            handle: default(),
            speed: 1.,
        }
    }
}

impl<T: Asset> AssetAnimator<T> {
    /// Create a new asset animator component from a single tweenable.
    #[must_use]
    pub fn new(handle: Handle<T>, tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        Self {
            tweenable: Some(Box::new(tween)),
            handle,
            ..default()
        }
    }

    animator_impl!();

    #[must_use]
    fn handle(&self) -> Handle<T> {
        self.handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{lens::*, *};
    use bevy::reflect::TypeUuid;

    struct DummyLens {
        start: f32,
        end: f32,
    }

    #[derive(Component)]
    struct DummyComponent {
        value: f32,
    }

    #[derive(Reflect, TypeUuid)]
    #[uuid = "a33abc11-264e-4bbb-82e8-b87226bb4383"]
    struct DummyAsset {
        value: f32,
    }

    impl Lens<DummyComponent> for DummyLens {
        fn lerp(&mut self, target: &mut DummyComponent, ratio: f32) {
            target.value = self.start.lerp(&self.end, &ratio);
        }
    }

    impl Lens<DummyAsset> for DummyLens {
        fn lerp(&mut self, target: &mut DummyAsset, ratio: f32) {
            target.value = self.start.lerp(&self.end, &ratio);
        }
    }

    #[test]
    fn tweening_type() {
        let tweening_type = TweeningType::default();
        assert_eq!(tweening_type, TweeningType::Once);
    }

    #[test]
    fn tweening_direction() {
        let tweening_direction = TweeningDirection::default();
        assert_eq!(tweening_direction, TweeningDirection::Forward);
    }

    #[test]
    fn animator_state() {
        let mut state = AnimatorState::default();
        assert_eq!(state, AnimatorState::Playing);
        state = !state;
        assert_eq!(state, AnimatorState::Paused);
        state = !state;
        assert_eq!(state, AnimatorState::Playing);
    }

    #[test]
    fn ease_method() {
        let ease = EaseMethod::default();
        assert!(matches!(ease, EaseMethod::Linear));

        let ease = EaseMethod::EaseFunction(EaseFunction::QuadraticIn);
        assert_eq!(0., ease.sample(0.));
        assert_eq!(0.25, ease.sample(0.5));
        assert_eq!(1., ease.sample(1.));

        let ease = EaseMethod::Linear;
        assert_eq!(0., ease.sample(0.));
        assert_eq!(0.5, ease.sample(0.5));
        assert_eq!(1., ease.sample(1.));

        let ease = EaseMethod::Discrete(0.3);
        assert_eq!(0., ease.sample(0.));
        assert_eq!(1., ease.sample(0.5));
        assert_eq!(1., ease.sample(1.));

        let ease = EaseMethod::CustomFunction(|f| 1. - f);
        assert_eq!(0., ease.sample(1.));
        assert_eq!(0.5, ease.sample(0.5));
        assert_eq!(1., ease.sample(0.));
    }

    /// Animator::new()
    #[test]
    fn animator_new() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let animator = Animator::<DummyComponent>::new(tween);
        assert_eq!(animator.state, AnimatorState::default());
        let tween = animator.tweenable().unwrap();
        assert_eq!(tween.progress(), 0.);
    }

    /// Animator::with_state()
    #[test]
    fn animator_with_state() {
        for state in [AnimatorState::Playing, AnimatorState::Paused] {
            let tween = Tween::<DummyComponent>::new(
                EaseFunction::QuadraticInOut,
                TweeningType::PingPong,
                Duration::from_secs(1),
                DummyLens { start: 0., end: 1. },
            );
            let animator = Animator::new(tween).with_state(state);
            assert_eq!(animator.state, state);
        }
    }

    /// Animator::default() + Animator::set_tweenable()
    #[test]
    fn animator_default() {
        let mut animator = Animator::<DummyComponent>::default();
        assert!(animator.tweenable().is_none());
        assert!(animator.tweenable_mut().is_none());

        let tween = Tween::<DummyComponent>::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        animator.set_tweenable(tween);
        assert!(animator.tweenable().is_some());
        assert!(animator.tweenable_mut().is_some());
    }

    /// Animator control playback
    #[test]
    fn animator_controls() {
        let tween = Tween::<DummyComponent>::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut animator = Animator::new(tween);
        assert_eq!(animator.state, AnimatorState::Playing);
        assert!(animator.progress().abs() <= 1e-5);

        animator.stop();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert!(animator.progress().abs() <= 1e-5);

        animator.set_progress(0.5);
        assert_eq!(animator.state, AnimatorState::Paused);
        assert!((animator.progress() - 0.5).abs() <= 1e-5);

        animator.rewind();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert!(animator.progress().abs() <= 1e-5);

        animator.set_progress(0.5);
        animator.state = AnimatorState::Playing;
        assert_eq!(animator.state, AnimatorState::Playing);
        assert!((animator.progress() - 0.5).abs() <= 1e-5);

        animator.rewind();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert!(animator.progress().abs() <= 1e-5);

        animator.stop();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert!(animator.progress().abs() <= 1e-5);
    }

    /// AssetAnimator::new()
    #[test]
    fn asset_animator_new() {
        let tween = Tween::<DummyAsset>::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let animator = AssetAnimator::new(Handle::<DummyAsset>::default(), tween);
        assert_eq!(animator.state, AnimatorState::default());
        let tween = animator.tweenable().unwrap();
        assert_eq!(tween.progress(), 0.);
    }

    /// AssetAnimator::with_state()
    #[test]
    fn asset_animator_with_state() {
        for state in [AnimatorState::Playing, AnimatorState::Paused] {
            let tween = Tween::<DummyAsset>::new(
                EaseFunction::QuadraticInOut,
                TweeningType::PingPong,
                Duration::from_secs(1),
                DummyLens { start: 0., end: 1. },
            );
            let animator =
                AssetAnimator::new(Handle::<DummyAsset>::default(), tween).with_state(state);
            assert_eq!(animator.state, state);
        }
    }

    /// AssetAnimator::default() + AssetAnimator::set_tweenable()
    #[test]
    fn asset_animator_default() {
        let mut animator = AssetAnimator::<DummyAsset>::default();
        assert!(animator.tweenable().is_none());
        assert!(animator.tweenable_mut().is_none());
        assert_eq!(animator.handle(), Handle::<DummyAsset>::default());

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        animator.set_tweenable(tween);
        assert!(animator.tweenable().is_some());
        assert!(animator.tweenable_mut().is_some());
        assert_eq!(animator.handle(), Handle::<DummyAsset>::default());
    }

    /// AssetAnimator control playback
    #[test]
    fn asset_animator_controls() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut animator = AssetAnimator::new(Handle::<DummyAsset>::default(), tween);
        assert_eq!(animator.state, AnimatorState::Playing);
        assert!(animator.progress().abs() <= 1e-5);

        animator.stop();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert!(animator.progress().abs() <= 1e-5);

        animator.set_progress(0.5);
        assert_eq!(animator.state, AnimatorState::Paused);
        assert!((animator.progress() - 0.5).abs() <= 1e-5);

        animator.rewind();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert!(animator.progress().abs() <= 1e-5);

        animator.set_progress(0.5);
        animator.state = AnimatorState::Playing;
        assert_eq!(animator.state, AnimatorState::Playing);
        assert!((animator.progress() - 0.5).abs() <= 1e-5);

        animator.rewind();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert!(animator.progress().abs() <= 1e-5);

        animator.stop();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert!(animator.progress().abs() <= 1e-5);
    }
}
