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
//! This library provides interpolation-based animation between ("tweening") two values, for a variety
//! of Bevy components and assets. Each field of a component or asset can be animated via a collection
//! or predefined easing functions, or providing a custom animation curve.
//!
//! # Example
//!
//! Add the tweening plugin to your app:
//!
//! ```rust,no_run
//! # use bevy::prelude::*;
//! # use bevy_tweening::*;
//! App::default()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(TweeningPlugin)
//!     .run();
//! ```
//!
//! Animate the position ([`Transform::translation`]) of an [`Entity`]:
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_tweening::*;
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
//!     // The lens gives access to the Transform component of the Sprite,
//!     // for the Animator to animate it. It also contains the start and
//!     // end values associated with the animation ratios 0. and 1.
//!     TransformPositionLens {
//!         start: Vec3::new(0., 0., 0.),
//!         end: Vec3::new(1., 2., -4.),
//!     },
//! );
//! 
//! commands
//!     // Spawn a Sprite entity to animate the position of.
//!     .spawn_bundle(SpriteBundle {
//!         sprite: Sprite {
//!             color: Color::RED,
//!             custom_size: Some(Vec2::new(size, size)),
//!             ..Default::default()
//!         },
//!         ..Default::default()
//!     })
//!     // Add an Animator component to control and execute the animation.
//!     .insert(Animator::new(tween));
//! # }
//! ```
//!
//! # Animators and lenses
//!
//! Bevy components and assets are animated with tweening animator components. Those animators determine
//! the fields to animate using lenses.
//!
//! ## Components animation
//!
//! Components are animated with the [`Animator`] component, which is generic over the type of component
//! it animates. This is a restriction imposed by Bevy, to access the animated component as a mutable
//! reference via a [`Query`] and comply with the ECS rules.
//!
//! The [`Animator`] itself is not generic over the subset of fields of the components it animates.
//! This limits the proliferation of generic types when animating e.g. both the position and rotation
//! of an entity.
//!
//! ## Assets animation
//!
//! Assets are animated in a similar way to component, via the [`AssetAnimator`] component. Because assets
//! are typically shared, and the animation applies to the asset itself, all users of the asset see the
//! animation. For example, animating the color of a [`ColorMaterial`] will change the color of all the
//! 2D meshes using that material.
//!
//! ## Lenses
//!
//! Both [`Animator`] and [`AssetAnimator`] access the field(s) to animate via a lens, a type that implements
//! the [`Lens`] trait. Several predefined lenses are provided for the most commonly animated fields, like the
//! components of a [`Transform`]. A custom lens can also be created by implementing the trait, allowing to
//! animate virtually any field of any Bevy component or asset.
//!
//! [`Transform::translation`]: https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html#structfield.translation
//! [`Entity`]: https://docs.rs/bevy/0.6.0/bevy/ecs/entity/struct.Entity.html
//! [`Query`]: https://docs.rs/bevy/0.6.0/bevy/ecs/system/struct.Query.html
//! [`ColorMaterial`]: https://docs.rs/bevy/0.6.0/bevy/sprite/struct.ColorMaterial.html
//! [`Sprite`]: https://docs.rs/bevy/0.6.0/bevy/sprite/struct.Sprite.html
//! [`Transform`]: https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html

use std::time::Duration;

use bevy::{asset::Asset, prelude::*};

use interpolation::Ease as IEase;
pub use interpolation::EaseFunction;
pub use interpolation::Lerp;

mod lens;
mod plugin;

pub use lens::{
    ColorMaterialColorLens, Lens, SpriteColorLens, TextColorLens, TransformPositionLens,
    TransformRotationLens, TransformScaleLens, UiPositionLens,
};
pub use plugin::{asset_animator_system, component_animator_system, TweeningPlugin};

/// Type of looping for a tween animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweeningType {
    /// Run the animation once from state to end only.
    Once,
    /// Looping, restarting from the start once finished.
    Loop,
    /// Repeat the animation back and forth.
    PingPong,
}

/// Playback state of an animator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatorState {
    /// The animation is playing. This is the default state.
    Playing,
    /// The animation is paused/stopped.
    Paused,
}

impl Default for AnimatorState {
    fn default() -> Self {
        AnimatorState::Playing
    }
}

impl std::ops::Not for AnimatorState {
    type Output = AnimatorState;

    fn not(self) -> Self::Output {
        match self {
            AnimatorState::Paused => AnimatorState::Playing,
            AnimatorState::Playing => AnimatorState::Paused,
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
    fn sample(self, x: f32) -> f32 {
        match self {
            EaseMethod::EaseFunction(function) => x.calc(function),
            EaseMethod::Linear => x,
            EaseMethod::Discrete(limit) => {
                if x > limit {
                    1.
                } else {
                    0.
                }
            }
            EaseMethod::CustomFunction(function) => function(x),
        }
    }
}

impl Into<EaseMethod> for EaseFunction {
    fn into(self) -> EaseMethod {
        EaseMethod::EaseFunction(self)
    }
}

/// Direction a tweening animation is playing.
///
/// For all but [`TweeningType::PingPong`] this is always [`TweeningDirection::Forward`]. For the
/// [`TweeningType::PingPong`] tweening type, this is either forward (from start to end; ping) or
/// backward (from end to start; pong).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweeningDirection {
    /// Animation playing from start to end.
    Forward,
    /// Animation playing from end to start.
    Backward,
}

impl std::ops::Not for TweeningDirection {
    type Output = TweeningDirection;

    fn not(self) -> Self::Output {
        match self {
            TweeningDirection::Forward => TweeningDirection::Backward,
            TweeningDirection::Backward => TweeningDirection::Forward,
        }
    }
}

/// Playback state of a [`Tweenable`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweenState {
    /// Not animated.
    Stopped,
    /// Animating.
    Running,
    /// Animation ended (but stop not called).
    Ended,
}

/// An animatable entity, either a single [`Tween`] or a collection of them.
pub trait Tweenable<T>: Send + Sync {
    /// Get the total duration of the animation.
    fn duration(&self) -> Duration;

    /// Get the current progress in \[0:1\] of the animation.
    fn progress(&self) -> f32;

    /// Tick the animation, advancing it by the given delta time and mutating the
    /// given target component or asset.
    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState;

    /// Stop the animation.
    fn stop(&mut self);
}

impl<T> Tweenable<T> for Box<dyn Tweenable<T> + Send + Sync + 'static> {
    fn duration(&self) -> Duration {
        self.as_ref().duration()
    }
    fn progress(&self) -> f32 {
        self.as_ref().progress()
    }
    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState {
        self.as_mut().tick(delta, target)
    }
    fn stop(&mut self) {
        self.as_mut().stop()
    }
}

/// Trait for boxing a [`Tweenable`] trait object.
pub trait IntoBoxDynTweenable<T> {
    /// Convert the current object into a boxed [`Tweenable`].
    fn into_box_dyn(this: Self) -> Box<dyn Tweenable<T> + Send + Sync + 'static>;
}

impl<T, U: Tweenable<T> + Send + Sync + 'static> IntoBoxDynTweenable<T> for U {
    fn into_box_dyn(this: U) -> Box<dyn Tweenable<T> + Send + Sync + 'static> {
        Box::new(this)
    }
}

/// Single tweening animation instance.
pub struct Tween<T> {
    ease_function: EaseMethod,
    timer: Timer,
    state: TweenState,
    tweening_type: TweeningType,
    direction: TweeningDirection,
    lens: Box<dyn Lens<T> + Send + Sync + 'static>,
    on_started: Option<Box<dyn FnMut() + Send + Sync + 'static>>,
    on_ended: Option<Box<dyn FnMut() + Send + Sync + 'static>>,
}

impl<T: 'static> Tween<T> {
    /// Chain another [`Tweenable`] after this tween, making a sequence with the two.
    pub fn then(self, tween: impl Tweenable<T> + Send + Sync + 'static) -> Sequence<T> {
        Sequence::from_single(self).then(tween)
    }
}

impl<T> Tween<T> {
    /// Create a new tween animation.
    pub fn new<L>(
        ease_function: impl Into<EaseMethod>,
        tweening_type: TweeningType,
        duration: Duration,
        lens: L,
    ) -> Self
    where
        L: Lens<T> + Send + Sync + 'static,
    {
        Tween {
            ease_function: ease_function.into(),
            timer: Timer::new(duration, tweening_type != TweeningType::Once),
            state: TweenState::Stopped,
            tweening_type,
            direction: TweeningDirection::Forward,
            lens: Box::new(lens),
            on_started: None,
            on_ended: None,
        }
    }

    /// The current animation direction.
    ///
    /// See [`TweeningDirection`] for details.
    pub fn direction(&self) -> TweeningDirection {
        self.direction
    }

    /// Set a callback invoked when the animation starts.
    pub fn set_started<C>(&mut self, callback: C)
    where
        C: FnMut() + Send + Sync + 'static,
    {
        self.on_started = Some(Box::new(callback));
    }

    /// Clear the callback invoked when the animation starts.
    pub fn clear_started(&mut self) {
        self.on_started = None;
    }

    /// Set a callback invoked when the animation ends.
    pub fn set_ended<C>(&mut self, callback: C)
    where
        C: FnMut() + Send + Sync + 'static,
    {
        self.on_ended = Some(Box::new(callback));
    }

    /// Clear the callback invoked when the animation ends.
    pub fn clear_ended(&mut self) {
        self.on_ended = None;
    }

    /// Is the animation playback looping?
    pub fn is_looping(&self) -> bool {
        self.tweening_type != TweeningType::Once
    }
}

impl<T> Tweenable<T> for Tween<T> {
    fn duration(&self) -> Duration {
        self.timer.duration()
    }

    /// Current animation progress ratio between 0 and 1.
    ///
    /// For reversed playback ([`TweeningDirection::Backward`]), the ratio goes from 0 at the
    /// end point (beginning of backward playback) to 1 at the start point (end of backward
    /// playback).
    fn progress(&self) -> f32 {
        match self.direction {
            TweeningDirection::Forward => self.timer.percent(),
            TweeningDirection::Backward => self.timer.percent_left(),
        }
    }

    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState {
        let old_state = self.state;
        if old_state == TweenState::Stopped {
            self.state = TweenState::Running;
            if let Some(cb) = &mut self.on_started {
                cb();
            }
        }

        self.timer.tick(delta);

        // Toggle direction immediately, so self.progress() returns the correct ratio
        if self.timer.just_finished() && self.tweening_type == TweeningType::PingPong {
            self.direction = !self.direction;
        }

        let progress = self.progress();
        let factor = self.ease_function.sample(progress);
        self.lens.lerp(target, factor);

        if self.timer.just_finished() {
            self.state = TweenState::Ended;
            // This is always true for non ping-pong, and is true for ping-pong when
            // coming back to start after a full cycle start -> end -> start.
            if self.direction == TweeningDirection::Forward {
                if let Some(cb) = &mut self.on_ended {
                    cb();
                }
            }
        }

        self.state
    }

    fn stop(&mut self) {
        self.state = TweenState::Stopped;
        self.timer.reset();
    }
}

/// A sequence of tweens played back in order one after the other.
pub struct Sequence<T> {
    tweens: Vec<Box<dyn Tweenable<T> + Send + Sync + 'static>>,
    index: usize,
    state: TweenState,
    duration: Duration,
    time: Duration,
}

impl<T> Sequence<T> {
    /// Create a new sequence of tweens.
    pub fn new(items: impl IntoIterator<Item = impl IntoBoxDynTweenable<T>>) -> Self {
        let tweens: Vec<_> = items
            .into_iter()
            .map(IntoBoxDynTweenable::into_box_dyn)
            .collect();
        let duration = tweens.iter().map(|t| t.duration()).sum();
        Sequence {
            tweens,
            index: 0,
            state: TweenState::Stopped,
            duration,
            time: Duration::from_secs(0),
        }
    }

    /// Create a new sequence containing a single tween.
    pub fn from_single(tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        let duration = tween.duration();
        Sequence {
            tweens: vec![Box::new(tween)],
            index: 0,
            state: TweenState::Stopped,
            duration,
            time: Duration::from_secs(0),
        }
    }

    /// Append a [`Tweenable`] to this sequence.
    pub fn then(mut self, tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        self.duration += tween.duration();
        self.tweens.push(Box::new(tween));
        self
    }

    /// Index of the current active tween in the sequence.
    pub fn index(&self) -> usize {
        self.index.min(self.tweens.len() - 1)
    }

    /// Get the current active tween in the sequence.
    pub fn current(&self) -> &dyn Tweenable<T> {
        self.tweens[self.index()].as_ref()
    }
}

impl<T> Tweenable<T> for Sequence<T> {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn progress(&self) -> f32 {
        self.time.as_secs_f32() / self.duration.as_secs_f32()
    }

    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState {
        if self.index < self.tweens.len() {
            self.time = (self.time + delta).min(self.duration);
            let tween = &mut self.tweens[self.index];
            let state = tween.tick(delta, target);
            if state == TweenState::Ended {
                tween.stop();
                self.index += 1;
                if self.index >= self.tweens.len() {
                    self.state = TweenState::Ended;
                }
            }
        }
        self.state
    }

    fn stop(&mut self) {
        if self.state != TweenState::Stopped {
            self.state = TweenState::Stopped;
            if self.index < self.tweens.len() {
                let tween = &mut self.tweens[self.index];
                tween.stop();
            }
        }
    }
}

/// A collection of [`Tweenable`] executing in parallel.
pub struct Tracks<T> {
    tracks: Vec<Box<dyn Tweenable<T> + Send + Sync + 'static>>,
    duration: Duration,
    time: Duration,
}

impl<T> Tracks<T> {
    /// Create a new [`Tracks`] from an iterator over a collection of [`Tweenable`].
    pub fn new(items: impl IntoIterator<Item = impl IntoBoxDynTweenable<T>>) -> Self {
        let tracks: Vec<_> = items
            .into_iter()
            .map(IntoBoxDynTweenable::into_box_dyn)
            .collect();
        let duration = tracks.iter().map(|t| t.duration()).max().unwrap();
        Tracks {
            tracks,
            duration,
            time: Duration::from_secs(0),
        }
    }
}

impl<T> Tweenable<T> for Tracks<T> {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn progress(&self) -> f32 {
        self.time.as_secs_f32() / self.duration.as_secs_f32()
    }

    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState {
        let mut any_running = true;
        for tweenable in &mut self.tracks {
            any_running = any_running && (tweenable.tick(delta, target) == TweenState::Running);
        }
        if any_running {
            self.time = (self.time + delta).min(self.duration);
            TweenState::Running
        } else {
            TweenState::Ended
        }
    }

    fn stop(&mut self) {
        for seq in &mut self.tracks {
            seq.stop();
        }
    }
}

/// Component to control the animation of another component.
#[derive(Component)]
pub struct Animator<T: Component> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
    prev_state: AnimatorState,
    tweenable: Option<Box<dyn Tweenable<T> + Send + Sync + 'static>>,
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
        Animator {
            state: Default::default(),
            prev_state: Default::default(),
            tweenable: None,
        }
    }
}

impl<T: Component> Animator<T> {
    /// Create a new animator component from a single [`Tween`] or [`Sequence`].
    pub fn new(tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        Animator {
            tweenable: Some(Box::new(tween)),
            ..Default::default()
        }
    }

    /// Set the initial state of the animator.
    pub fn with_state(mut self, state: AnimatorState) -> Self {
        self.state = state;
        self.prev_state = state;
        self
    }

    /// Set the top-level tweenable item this animator controls.
    pub fn set_tweenable(&mut self, tween: impl Tweenable<T> + Send + Sync + 'static) {
        self.tweenable = Some(Box::new(tween));
    }

    /// Get the collection of sequences forming the parallel tracks of animation.
    pub fn tweenable(&self) -> Option<&(dyn Tweenable<T> + Send + Sync + 'static)> {
        if let Some(tweenable) = &self.tweenable {
            Some(tweenable.as_ref())
        } else {
            None
        }
    }

    /// Get the mutable collection of sequences forming the parallel tracks of animation.
    pub fn tweenable_mut(&mut self) -> Option<&mut (dyn Tweenable<T> + Send + Sync + 'static)> {
        if let Some(tweenable) = &mut self.tweenable {
            Some(tweenable.as_mut())
        } else {
            None
        }
    }
}

/// Component to control the animation of an asset.
#[derive(Component)]
pub struct AssetAnimator<T: Asset> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
    prev_state: AnimatorState,
    tweenable: Option<Box<dyn Tweenable<T> + Send + Sync + 'static>>,
    handle: Handle<T>,
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
        AssetAnimator {
            state: Default::default(),
            prev_state: Default::default(),
            tweenable: None,
            handle: Default::default(),
        }
    }
}

impl<T: Asset> AssetAnimator<T> {
    /// Create a new animator component from a single [`Tween`] or [`Sequence`].
    pub fn new(handle: Handle<T>, tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        AssetAnimator {
            tweenable: Some(Box::new(tween)),
            handle,
            ..Default::default()
        }
    }

    /// Set the initial state of the animator.
    pub fn with_state(mut self, state: AnimatorState) -> Self {
        self.state = state;
        self.prev_state = state;
        self
    }

    /// Set the top-level tweenable item this animator controls.
    pub fn set_tweenable(&mut self, tween: impl Tweenable<T> + Send + Sync + 'static) {
        self.tweenable = Some(Box::new(tween));
    }

    /// Get the collection of sequences forming the parallel tracks of animation.
    pub fn tweenable(&self) -> Option<&(dyn Tweenable<T> + Send + Sync + 'static)> {
        if let Some(tweenable) = &self.tweenable {
            Some(tweenable.as_ref())
        } else {
            None
        }
    }

    /// Get the mutable collection of sequences forming the parallel tracks of animation.
    pub fn tweenable_mut(&mut self) -> Option<&mut (dyn Tweenable<T> + Send + Sync + 'static)> {
        if let Some(tweenable) = &mut self.tweenable {
            Some(tweenable.as_mut())
        } else {
            None
        }
    }

    fn handle(&self) -> Handle<T> {
        self.handle.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Utility to compare floating-point values with a tolerance.
    fn abs_diff_eq(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() < tol
    }

    /// Test ticking of a single tween in isolation.
    #[test]
    fn tween_tick() {
        for tweening_type in &[
            TweeningType::Once,
            TweeningType::Loop,
            TweeningType::PingPong,
        ] {
            // Create a linear tween over 1 second
            let mut tween = Tween::new(
                EaseMethod::Linear,
                *tweening_type,
                Duration::from_secs_f32(1.0),
                TransformPositionLens {
                    start: Vec3::ZERO,
                    end: Vec3::ONE,
                },
            );

            // Register callbacks to count started/ended events
            let started_count = Arc::new(Mutex::new(0));
            let ended_count = Arc::new(Mutex::new(0));
            let sc = Arc::clone(&started_count);
            let ec = Arc::clone(&ended_count);
            tween.set_started(move || {
                let mut sc = sc.lock().unwrap();
                *sc += 1;
            });
            tween.set_ended(move || {
                let mut ec = ec.lock().unwrap();
                *ec += 1;
            });
            assert_eq!(*started_count.lock().unwrap(), 0);
            assert_eq!(*ended_count.lock().unwrap(), 0);

            // Loop over 2.2 seconds, so greater than one ping-pong loop
            let mut transform = Transform::default();
            let tick_duration = Duration::from_secs_f32(0.2);
            for i in 1..=11 {
                // Calculate expected values
                let (ratio, ec, dir) = match tweening_type {
                    TweeningType::Once => {
                        let r = (i as f32 * 0.2).min(1.0);
                        let ec = if i >= 5 { 1 } else { 0 };
                        (r, ec, TweeningDirection::Forward)
                    }
                    TweeningType::Loop => {
                        let r = (i as f32 * 0.2).fract();
                        let ec = i / 5;
                        (r, ec, TweeningDirection::Forward)
                    }
                    TweeningType::PingPong => {
                        let i10 = i % 10;
                        let r = if i10 >= 5 {
                            (10 - i10) as f32 * 0.2
                        } else {
                            i10 as f32 * 0.2
                        };
                        let ec = i / 10;
                        let dir = if i10 >= 5 {
                            TweeningDirection::Backward
                        } else {
                            TweeningDirection::Forward
                        };
                        (r, ec, dir)
                    }
                };
                println!("Expected; r={} ec={} dir={:?}", ratio, ec, dir);

                // Tick the tween
                tween.tick(tick_duration, &mut transform);

                // Check actual values
                assert_eq!(tween.direction(), dir);
                assert!(abs_diff_eq(tween.progress(), ratio, 1e-5));
                assert!(transform.translation.abs_diff_eq(Vec3::splat(ratio), 1e-5));
                assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));
                assert_eq!(*started_count.lock().unwrap(), 1);
                assert_eq!(*ended_count.lock().unwrap(), ec);
            }
        }
    }

    /// Test ticking a sequence of tweens.
    #[test]
    fn seq_tick() {
        let tween1 = Tween::new(
            EaseMethod::Linear,
            TweeningType::Once,
            Duration::from_secs_f32(1.0),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let tween2 = Tween::new(
            EaseMethod::Linear,
            TweeningType::Once,
            Duration::from_secs_f32(1.0),
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_x(180_f32.to_radians()),
            },
        );
        let mut seq = Sequence::from_single(tween1).then(tween2);
        let mut transform = Transform::default();
        for i in 1..=11 {
            seq.tick(Duration::from_secs_f32(0.2), &mut transform);
            if i <= 5 {
                let r = i as f32 * 0.2;
                assert_eq!(transform, Transform::from_translation(Vec3::splat(r)));
            } else if i <= 10 {
                let alpha_deg = (36 * (i - 5)) as f32;
                assert!(transform.translation.abs_diff_eq(Vec3::splat(1.), 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(alpha_deg.to_radians()), 1e-5));
            } else {
                assert!(transform.translation.abs_diff_eq(Vec3::splat(1.), 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(180_f32.to_radians()), 1e-5));
            }
        }
    }

    /// Animator::new()
    #[test]
    fn animator_new() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            std::time::Duration::from_secs(1),
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 2.),
            },
        );
        let animator = Animator::new(tween);
        assert_eq!(animator.state, AnimatorState::default());
        let tween = animator.tweenable().unwrap();
        assert_eq!(tween.progress(), 0.);
    }

    /// AssetAnimator::new()
    #[test]
    fn asset_animator_new() {
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            std::time::Duration::from_secs(1),
            ColorMaterialColorLens {
                start: Color::RED,
                end: Color::BLUE,
            },
        );
        let animator = AssetAnimator::new(Handle::<ColorMaterial>::default(), tween);
        assert_eq!(animator.state, AnimatorState::default());
        let tween = animator.tweenable().unwrap();
        assert_eq!(tween.progress(), 0.);
    }
}
