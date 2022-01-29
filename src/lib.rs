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
//! commands
//!     // Spawn a Sprite entity to animate the position of
//!     .spawn_bundle(SpriteBundle {
//!         sprite: Sprite {
//!             color: Color::RED,
//!             custom_size: Some(Vec2::new(size, size)),
//!             ..Default::default()
//!         },
//!         ..Default::default()
//!     })
//!     // Add an Animator component to perform the animation
//!     .insert(Animator::new(
//!         // Use a quadratic easing on both endpoints
//!         EaseFunction::QuadraticInOut,
//!         // Loop animation back and forth
//!         TweeningType::PingPong,
//!         // Animation time (one way only; for ping-pong it takes 2 seconds
//!         // to come back to start)
//!         Duration::from_secs(1),
//!         // The lens gives access to the Transform component of the Sprite,
//!         // for the Animator to animate it. It also contains the start and
//!         // end values associated with the animation ratios 0. and 1.
//!         TransformPositionLens {
//!             start: Vec3::new(0., 0., 0.),
//!             end: Vec3::new(1., 2., -4.),
//!         },
//!     ));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TweenState {
    /// Not animated.
    Stopped,
    /// Animating.
    Running,
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

    /// Current animation progress ratio between 0 and 1.
    ///
    /// For reversed playback ([`TweeningDirection::Backward`]), the ratio goes from 0 at the
    /// end point (beginning of backward playback) to 1 at the start point (end of backward
    /// playback).
    pub fn progress(&self) -> f32 {
        match self.direction {
            TweeningDirection::Forward => self.timer.percent(),
            TweeningDirection::Backward => self.timer.percent_left(),
        }
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

    fn tick(&mut self, delta: Duration, target: &mut T) {
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
            // This is always true for non ping-pong, and is true for ping-pong when
            // coming back to start after a full cycle start -> end -> start.
            if self.direction == TweeningDirection::Forward {
                if let Some(cb) = &mut self.on_ended {
                    cb();
                }
            }
        }
    }

    fn stop(&mut self) {
        if self.state == TweenState::Running {
            self.state = TweenState::Stopped;
            self.timer.reset();
        }
    }
}

struct Sequence<T> {
    tweens: Vec<Tween<T>>,
    index: usize,
    state: TweenState,
}

impl<T> Sequence<T> {
    pub fn new<I>(tweens: I) -> Self
    where
        I: IntoIterator<Item = Tween<T>>,
    {
        Sequence {
            tweens: tweens.into_iter().collect(),
            index: 0,
            state: TweenState::Stopped,
        }
    }

    pub fn from_single(tween: Tween<T>) -> Self {
        Sequence {
            tweens: vec![tween],
            index: 0,
            state: TweenState::Stopped,
        }
    }

    fn tick(&mut self, delta: Duration, target: &mut T) {
        if self.index < self.tweens.len() {
            let tween = &mut self.tweens[self.index];
            tween.tick(delta, target);
            if tween.progress() >= 1.0 {
                self.index += 1;
            }
        }
    }

    fn stop(&mut self) {
        if self.state == TweenState::Running {
            self.state = TweenState::Stopped;
            if self.index < self.tweens.len() {
                let tween = &mut self.tweens[self.index];
                tween.stop();
            }
        }
    }
}

struct Tracks<T> {
    tracks: Vec<Sequence<T>>,
}

/// Component to control the animation of another component.
#[derive(Component)]
pub struct Animator<T: Component> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
    prev_state: AnimatorState,
    tracks: Tracks<T>,
}

impl<T: Component + std::fmt::Debug> std::fmt::Debug for Animator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Animator")
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Component> Animator<T> {
    /// Create a new animator component from an easing function, tweening type, and a lens.
    /// The type `T` of the component to animate can generally be deducted from the lens type itself.
    /// This creates a new [`Tween`] instance then assign it to a newly created animator.
    pub fn new<L>(
        ease_function: impl Into<EaseMethod>,
        tweening_type: TweeningType,
        duration: Duration,
        lens: L,
    ) -> Self
    where
        L: Lens<T> + Send + Sync + 'static,
    {
        let tween = Tween::new(ease_function, tweening_type, duration, lens);
        Animator {
            state: AnimatorState::default(),
            prev_state: AnimatorState::default(),
            tracks: Tracks {
                tracks: vec![Sequence::from_single(tween)],
            },
        }
    }

    /// Create a new animator component from a single tween instance.
    pub fn new_single(tween: Tween<T>) -> Self {
        Animator {
            state: AnimatorState::default(),
            prev_state: AnimatorState::default(),
            tracks: Tracks {
                tracks: vec![Sequence::from_single(tween)],
            },
        }
    }

    /// Create a new animator component from a sequence of tween instances.
    /// The tweens are played in order, one after the other. They all must be non-looping.
    pub fn new_seq(tweens: Vec<Tween<T>>) -> Self {
        for t in &tweens {
            assert!(matches!(t.tweening_type, TweeningType::Once { .. }));
        }
        Animator {
            state: AnimatorState::Playing,
            prev_state: AnimatorState::Playing,
            tracks: Tracks {
                tracks: vec![Sequence::new(tweens)],
            },
        }
    }

    /// Set the initial state of the animator.
    pub fn with_state(mut self, state: AnimatorState) -> Self {
        self.state = state;
        self.prev_state = state;
        self
    }

    #[allow(dead_code)]
    fn tracks(&self) -> &Tracks<T> {
        &self.tracks
    }

    fn tracks_mut(&mut self) -> &mut Tracks<T> {
        &mut self.tracks
    }
}

/// Component to control the animation of an asset.
#[derive(Component)]
pub struct AssetAnimator<T: Asset> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
    prev_state: AnimatorState,
    tracks: Tracks<T>,
    handle: Handle<T>,
}

impl<T: Asset + std::fmt::Debug> std::fmt::Debug for AssetAnimator<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetAnimator")
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Asset> AssetAnimator<T> {
    /// Create a new asset animator component from an easing function, tweening type, and a lens.
    /// The type `T` of the asset to animate can generally be deducted from the lens type itself.
    /// The component can be attached on any entity.
    pub fn new<L>(
        handle: Handle<T>,
        ease_function: impl Into<EaseMethod>,
        tweening_type: TweeningType,
        duration: Duration,
        lens: L,
    ) -> Self
    where
        L: Lens<T> + Send + Sync + 'static,
    {
        let tween = Tween::new(ease_function, tweening_type, duration, lens);
        AssetAnimator {
            state: AnimatorState::Playing,
            prev_state: AnimatorState::Playing,
            tracks: Tracks {
                tracks: vec![Sequence::from_single(tween)],
            },
            handle,
        }
    }

    /// Create a new animator component from a single tween instance.
    pub fn new_single(handle: Handle<T>, tween: Tween<T>) -> Self {
        AssetAnimator {
            state: AnimatorState::Playing,
            prev_state: AnimatorState::Playing,
            tracks: Tracks {
                tracks: vec![Sequence::from_single(tween)],
            },
            handle,
        }
    }

    /// Create a new animator component from a sequence of tween instances.
    /// The tweens are played in order, one after the other. They all must be non-looping.
    pub fn new_seq(handle: Handle<T>, tweens: Vec<Tween<T>>) -> Self {
        for t in &tweens {
            assert!(matches!(t.tweening_type, TweeningType::Once { .. }));
        }
        AssetAnimator {
            state: AnimatorState::Playing,
            prev_state: AnimatorState::Playing,
            tracks: Tracks {
                tracks: vec![Sequence::new(tweens)],
            },
            handle,
        }
    }

    /// Set the initial state of the animator.
    pub fn with_state(mut self, state: AnimatorState) -> Self {
        self.state = state;
        self.prev_state = state;
        self
    }

    fn handle(&self) -> Handle<T> {
        self.handle.clone()
    }

    #[allow(dead_code)]
    fn tracks(&self) -> &Tracks<T> {
        &self.tracks
    }

    fn tracks_mut(&mut self) -> &mut Tracks<T> {
        &mut self.tracks
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
        let mut seq = Sequence::new([tween1, tween2]);
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
        let animator = Animator::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            std::time::Duration::from_secs(1),
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 2.),
            },
        );
        assert_eq!(animator.state, AnimatorState::default());
        let tracks = animator.tracks();
        assert_eq!(tracks.tracks.len(), 1);
        let seq = &tracks.tracks[0];
        assert_eq!(seq.tweens.len(), 1);
        let tween = &seq.tweens[0];
        assert_eq!(tween.direction(), TweeningDirection::Forward);
        assert_eq!(tween.progress(), 0.);
    }

    /// AssetAnimator::new()
    #[test]
    fn asset_animator_new() {
        let animator = AssetAnimator::new(
            Handle::<ColorMaterial>::default(),
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            std::time::Duration::from_secs(1),
            ColorMaterialColorLens {
                start: Color::RED,
                end: Color::BLUE,
            },
        );
        assert_eq!(animator.state, AnimatorState::default());
        let tracks = animator.tracks();
        assert_eq!(tracks.tracks.len(), 1);
        let seq = &tracks.tracks[0];
        assert_eq!(seq.tweens.len(), 1);
        let tween = &seq.tweens[0];
        assert_eq!(tween.direction(), TweeningDirection::Forward);
        assert_eq!(tween.progress(), 0.);
    }
}
