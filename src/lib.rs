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
//!     // Animation time.
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
//! - [`Delay`] - A time delay. This doesn't animate anything.
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
//! #    Duration::from_secs(2),
//! #    TransformScaleLens {
//! #        start: Vec3::ZERO,
//! #        end: Vec3::ONE,
//! #    },
//! );
//! let tween2 = Tween::new(
//!     // [...]
//! #    EaseFunction::QuadraticInOut,
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
//! component. This requires the `bevy_asset` feature (enabled by default).
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
//! [`Transform::translation`]: https://docs.rs/bevy/0.8.0/bevy/transform/components/struct.Transform.html#structfield.translation
//! [`Entity`]: https://docs.rs/bevy/0.8.0/bevy/ecs/entity/struct.Entity.html
//! [`Query`]: https://docs.rs/bevy/0.8.0/bevy/ecs/system/struct.Query.html
//! [`ColorMaterial`]: https://docs.rs/bevy/0.8.0/bevy/sprite/struct.ColorMaterial.html
//! [`Sprite`]: https://docs.rs/bevy/0.8.0/bevy/sprite/struct.Sprite.html
//! [`Transform`]: https://docs.rs/bevy/0.8.0/bevy/transform/components/struct.Transform.html

use std::time::Duration;

#[cfg(feature = "bevy_asset")]
use bevy::asset::Asset;
use bevy::prelude::*;
use interpolation::Ease as IEase;
pub use interpolation::{EaseFunction, Lerp};

pub use lens::Lens;
#[cfg(feature = "bevy_asset")]
pub use plugin::asset_animator_system;
pub use plugin::{component_animator_system, AnimationSystem, TweeningPlugin};
pub use tweenable::{
    BoxedTweenable, Delay, Sequence, Targetable, Tracks, Tween, TweenCompleted, TweenState,
    Tweenable,
};

pub mod lens;
mod plugin;
mod tweenable;

#[cfg(test)]
mod test_utils;

/// How many times to repeat a tween animation. See also: [`RepeatStrategy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatCount {
    /// Run the animation N times.
    Finite(u32),
    /// Run the animation for some amount of time.
    For(Duration),
    /// Loop the animation indefinitely.
    Infinite,
}

/// What to do when a tween animation needs to be repeated.
///
/// Only applicable when [`RepeatCount`] is greater than the animation duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatStrategy {
    /// Reset the animation back to its starting position.
    Repeat,
    /// Follow a ping-pong pattern, changing the direction each time an endpoint
    /// is reached.
    ///
    /// A complete cycle start -> end -> start always counts as 2 loop
    /// iterations for the various operations where looping matters. That
    /// is, a 1 second animation will take 2 seconds to end up back where it
    /// started.
    MirroredRepeat,
}

impl Default for RepeatCount {
    fn default() -> Self {
        Self::Finite(1)
    }
}

impl Default for RepeatStrategy {
    fn default() -> Self {
        Self::Repeat
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
/// For all but [`RepeatStrategy::MirroredRepeat`] this is always
/// [`TweeningDirection::Forward`], unless manually configured with
/// [`Tween::set_direction()`] in which case the value is constant equal to the
/// value set. When using [`RepeatStrategy::MirroredRepeat`], this is either
/// forward (from start to end; ping) or backward (from end to start; pong),
/// depending on the current iteration of the loop.
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

        /// Get the animation speed.
        ///
        /// See [`set_speed()`] for a definition of what the animation speed is.
        ///
        /// [`set_speed()`]: Animator::speed
        pub fn speed(&self) -> f32 {
            self.speed
        }

        /// Set the top-level tweenable item this animator controls.
        pub fn set_tweenable(&mut self, tween: impl Tweenable<T> + Send + Sync + 'static) {
            self.tweenable = Box::new(tween);
        }

        /// Get the top-level tweenable this animator is currently controlling.
        #[must_use]
        pub fn tweenable(&self) -> &(dyn Tweenable<T> + Send + Sync + 'static) {
            self.tweenable.as_ref()
        }

        /// Get the top-level mutable tweenable this animator is currently controlling.
        #[must_use]
        pub fn tweenable_mut(&mut self) -> &mut (dyn Tweenable<T> + Send + Sync + 'static) {
            self.tweenable.as_mut()
        }

        /// Stop animation playback and rewind the animation.
        ///
        /// This changes the animator state to [`AnimatorState::Paused`] and rewind its
        /// tweenable.
        pub fn stop(&mut self) {
            self.state = AnimatorState::Paused;
            self.tweenable_mut().rewind();
        }
    };
}

/// Component to control the animation of another component.
#[derive(Component)]
pub struct Animator<T: Component> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
    tweenable: BoxedTweenable<T>,
    speed: f32,
}

impl<T: Component + std::fmt::Debug> std::fmt::Debug for Animator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Animator")
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Component> Animator<T> {
    /// Create a new animator component from a single tweenable.
    #[must_use]
    pub fn new(tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        Self {
            state: default(),
            tweenable: Box::new(tween),
            speed: 1.,
        }
    }

    animator_impl!();
}

/// Component to control the animation of an asset.
#[cfg(feature = "bevy_asset")]
#[derive(Component)]
pub struct AssetAnimator<T: Asset> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
    tweenable: BoxedTweenable<T>,
    handle: Handle<T>,
    speed: f32,
}

#[cfg(feature = "bevy_asset")]
impl<T: Asset + std::fmt::Debug> std::fmt::Debug for AssetAnimator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetAnimator")
            .field("state", &self.state)
            .finish()
    }
}

#[cfg(feature = "bevy_asset")]
impl<T: Asset> AssetAnimator<T> {
    /// Create a new asset animator component from a single tweenable.
    #[must_use]
    pub fn new(handle: Handle<T>, tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        Self {
            state: default(),
            tweenable: Box::new(tween),
            handle,
            speed: 1.,
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
    #[cfg(feature = "bevy_asset")]
    use bevy::reflect::TypeUuid;

    use super::*;
    use crate::test_utils::*;

    struct DummyLens {
        start: f32,
        end: f32,
    }

    #[derive(Debug, Default, Component)]
    struct DummyComponent {
        value: f32,
    }

    #[cfg(feature = "bevy_asset")]
    #[derive(Debug, Default, Reflect, TypeUuid)]
    #[uuid = "a33abc11-264e-4bbb-82e8-b87226bb4383"]
    struct DummyAsset {
        value: f32,
    }

    impl Lens<DummyComponent> for DummyLens {
        fn lerp(&mut self, target: &mut DummyComponent, ratio: f32) {
            target.value = self.start.lerp(&self.end, &ratio);
        }
    }

    #[test]
    fn dummy_lens_component() {
        let mut c = DummyComponent::default();
        let mut l = DummyLens { start: 0., end: 1. };
        for r in [0_f32, 0.01, 0.3, 0.5, 0.9, 0.999, 1.] {
            l.lerp(&mut c, r);
            assert_approx_eq!(c.value, r);
        }
    }

    #[cfg(feature = "bevy_asset")]
    impl Lens<DummyAsset> for DummyLens {
        fn lerp(&mut self, target: &mut DummyAsset, ratio: f32) {
            target.value = self.start.lerp(&self.end, &ratio);
        }
    }

    #[cfg(feature = "bevy_asset")]
    #[test]
    fn dummy_lens_asset() {
        let mut a = DummyAsset::default();
        let mut l = DummyLens { start: 0., end: 1. };
        for r in [0_f32, 0.01, 0.3, 0.5, 0.9, 0.999, 1.] {
            l.lerp(&mut a, r);
            assert_approx_eq!(a.value, r);
        }
    }

    #[test]
    fn repeat_count() {
        let count = RepeatCount::default();
        assert_eq!(count, RepeatCount::Finite(1));
    }

    #[test]
    fn repeat_strategy() {
        let strategy = RepeatStrategy::default();
        assert_eq!(strategy, RepeatStrategy::Repeat);
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

    #[test]
    fn animator_new() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let animator = Animator::<DummyComponent>::new(tween);
        assert_eq!(animator.state, AnimatorState::default());
        assert_eq!(animator.tweenable().progress(), 0.);
    }

    #[test]
    fn animator_with_state() {
        for state in [AnimatorState::Playing, AnimatorState::Paused] {
            let tween = Tween::<DummyComponent>::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(1),
                DummyLens { start: 0., end: 1. },
            );
            let animator = Animator::new(tween).with_state(state);
            assert_eq!(animator.state, state);

            // impl Debug
            let debug_string = format!("{:?}", animator);
            assert_eq!(
                debug_string,
                format!("Animator {{ state: {:?} }}", animator.state)
            );
        }
    }

    #[test]
    fn animator_controls() {
        let tween = Tween::<DummyComponent>::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut animator = Animator::new(tween);
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_approx_eq!(animator.tweenable().progress(), 0.);

        animator.stop();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert_approx_eq!(animator.tweenable().progress(), 0.);

        animator.tweenable_mut().set_progress(0.5);
        assert_eq!(animator.state, AnimatorState::Paused);
        assert_approx_eq!(animator.tweenable().progress(), 0.5);

        animator.tweenable_mut().rewind();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert_approx_eq!(animator.tweenable().progress(), 0.);

        animator.tweenable_mut().set_progress(0.5);
        animator.state = AnimatorState::Playing;
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_approx_eq!(animator.tweenable().progress(), 0.5);

        animator.tweenable_mut().rewind();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_approx_eq!(animator.tweenable().progress(), 0.);

        animator.stop();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert_approx_eq!(animator.tweenable().progress(), 0.);
    }

    #[test]
    fn animator_speed() {
        let tween = Tween::<DummyComponent>::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );

        let mut animator = Animator::new(tween);
        assert_approx_eq!(animator.speed(), 1.); // default speed

        animator.set_speed(2.4);
        assert_approx_eq!(animator.speed(), 2.4);

        let tween = Tween::<DummyComponent>::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );

        let animator = Animator::new(tween).with_speed(3.5);
        assert_approx_eq!(animator.speed(), 3.5);
    }

    #[test]
    fn animator_set_tweenable() {
        let tween = Tween::<DummyComponent>::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut animator = Animator::new(tween);

        let tween2 = Tween::<DummyComponent>::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(2),
            DummyLens { start: 0., end: 1. },
        );
        animator.set_tweenable(tween2);

        assert_eq!(animator.tweenable().duration(), Duration::from_secs(2));
    }

    #[cfg(feature = "bevy_asset")]
    #[test]
    fn asset_animator_new() {
        let tween = Tween::<DummyAsset>::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let animator = AssetAnimator::new(Handle::<DummyAsset>::default(), tween);
        assert_eq!(animator.state, AnimatorState::default());
        assert_eq!(animator.handle(), Handle::<DummyAsset>::default());
        let tween = animator;
        assert_eq!(tween.tweenable().progress(), 0.);
    }

    #[cfg(feature = "bevy_asset")]
    #[test]
    fn asset_animator_with_state() {
        for state in [AnimatorState::Playing, AnimatorState::Paused] {
            let tween = Tween::<DummyAsset>::new(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(1),
                DummyLens { start: 0., end: 1. },
            );
            let animator =
                AssetAnimator::new(Handle::<DummyAsset>::default(), tween).with_state(state);
            assert_eq!(animator.state, state);

            // impl Debug
            let debug_string = format!("{:?}", animator);
            assert_eq!(
                debug_string,
                format!("AssetAnimator {{ state: {:?} }}", animator.state)
            );
        }
    }

    #[cfg(feature = "bevy_asset")]
    #[test]
    fn asset_animator_controls() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut animator = AssetAnimator::new(Handle::<DummyAsset>::default(), tween);
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_approx_eq!(animator.tweenable().progress(), 0.);

        animator.stop();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert_approx_eq!(animator.tweenable().progress(), 0.);

        animator.tweenable_mut().set_progress(0.5);
        assert_eq!(animator.state, AnimatorState::Paused);
        assert_approx_eq!(animator.tweenable().progress(), 0.5);

        animator.tweenable_mut().rewind();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert_approx_eq!(animator.tweenable().progress(), 0.);

        animator.tweenable_mut().set_progress(0.5);
        animator.state = AnimatorState::Playing;
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_approx_eq!(animator.tweenable().progress(), 0.5);

        animator.tweenable_mut().rewind();
        assert_eq!(animator.state, AnimatorState::Playing);
        assert_approx_eq!(animator.tweenable().progress(), 0.);

        animator.stop();
        assert_eq!(animator.state, AnimatorState::Paused);
        assert_approx_eq!(animator.tweenable().progress(), 0.);
    }

    #[cfg(feature = "bevy_asset")]
    #[test]
    fn asset_animator_speed() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );

        let mut animator = AssetAnimator::new(Handle::<DummyAsset>::default(), tween);
        assert_approx_eq!(animator.speed(), 1.); // default speed

        animator.set_speed(2.4);
        assert_approx_eq!(animator.speed(), 2.4);

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );

        let animator = AssetAnimator::new(Handle::<DummyAsset>::default(), tween).with_speed(3.5);
        assert_approx_eq!(animator.speed(), 3.5);
    }

    #[cfg(feature = "bevy_asset")]
    #[test]
    fn asset_animator_set_tweenable() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut animator = AssetAnimator::new(Handle::<DummyAsset>::default(), tween);

        let tween2 = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(2),
            DummyLens { start: 0., end: 1. },
        );
        animator.set_tweenable(tween2);

        assert_eq!(animator.tweenable().duration(), Duration::from_secs(2));
    }
}
