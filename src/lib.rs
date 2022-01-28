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
//! ```rust
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
//!         // Loop animation back and forth over 1 second, with a 0.5 second
//!         // pause after each cycle (start -> end -> start).
//!         TweeningType::PingPong {
//!             duration: Duration::from_secs(1),
//!             pause: Some(Duration::from_millis(500)),
//!         },
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
pub use plugin::TweeningPlugin;

/// How should this easing loop repeat
#[derive(Clone, Copy)]
pub enum TweeningType {
    /// Only happen once
    Once {
        /// duration of the easing
        duration: Duration,
    },
    /// Looping, restarting from the start once finished
    Loop {
        /// duration of the easing
        duration: Duration,
        /// duration of the pause between two loops
        pause: Option<Duration>,
    },
    /// Repeat the animation back and forth
    PingPong {
        /// duration of the easing
        duration: Duration,
        /// duration of the pause before starting again in the other direction
        pause: Option<Duration>,
    },
}

/// Playback state of an animator.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum AnimatorState {
    /// The animation is playing.
    Playing,
    /// The animation is paused/stopped.
    Paused,
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

/// Single tweening animation instance.
pub struct Tween<T> {
    ease_function: EaseMethod,
    timer: Timer,
    paused: bool,
    tweening_type: TweeningType,
    direction: TweeningDirection,
    lens: Box<dyn Lens<T> + Send + Sync + 'static>,
}

impl<T> Tween<T> {
    /// Create a new tween animation.
    pub fn new<L>(
        ease_function: impl Into<EaseMethod>,
        tweening_type: TweeningType,
        lens: L,
    ) -> Self
    where
        L: Lens<T> + Send + Sync + 'static,
    {
        Tween {
            ease_function: ease_function.into(),
            timer: match tweening_type {
                TweeningType::Once { duration } => Timer::new(duration, false),
                TweeningType::Loop { duration, .. } => Timer::new(duration, false),
                TweeningType::PingPong { duration, .. } => Timer::new(duration, false),
            },
            paused: false,
            tweening_type,
            direction: TweeningDirection::Forward,
            lens: Box::new(lens),
        }
    }

    /// A boolean indicating whether the animation is currently in the pause phase of a loop.
    ///
    /// The [`TweeningType::Loop`] and [`TweeningType::PingPong`] tweening types are looping over
    /// infinitely, with an optional pause between each loop. This function returns `true` if the
    /// animation is currently under such pause. For [`TweeningType::Once`], which has no pause,
    /// this always returns `false`.
    pub fn is_paused(&self) -> bool {
        self.paused
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

    fn tick(&mut self, delta: Duration, target: &mut T) {
        self.timer.tick(delta);
        if self.paused {
            if self.timer.just_finished() {
                match self.tweening_type {
                    TweeningType::Once { duration } => {
                        self.timer.set_duration(duration);
                    }
                    TweeningType::Loop { duration, .. } => {
                        self.timer.set_duration(duration);
                    }
                    TweeningType::PingPong { duration, .. } => {
                        self.timer.set_duration(duration);
                    }
                }
                self.timer.reset();
                self.paused = false;
            }
        } else {
            if self.timer.duration().as_secs_f32() != 0. {
                let progress = self.progress();
                let factor = self.ease_function.sample(progress);
                self.apply(target, factor);
            }
            if self.timer.finished() {
                match self.tweening_type {
                    TweeningType::Once { .. } => {
                        //commands.entity(entity).remove::<Animator>();
                    }
                    TweeningType::Loop { pause, .. } => {
                        if let Some(pause) = pause {
                            self.timer.set_duration(pause);
                            self.paused = true;
                        }
                        self.timer.reset();
                    }
                    TweeningType::PingPong { pause, .. } => {
                        if let Some(pause) = pause {
                            self.timer.set_duration(pause);
                            self.paused = true;
                        }
                        self.timer.reset();
                        self.direction = !self.direction;
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn apply(&mut self, target: &mut T, ratio: f32) {
        self.lens.lerp(target, ratio);
    }
}

struct Sequence<T> {
    tweens: Vec<Tween<T>>,
    index: usize,
}

impl<T> Sequence<T> {
    pub fn new<I>(tweens: I) -> Self
    where
        I: IntoIterator<Item = Tween<T>>,
    {
        Sequence {
            tweens: tweens.into_iter().collect(),
            index: 0,
        }
    }

    pub fn from_single(tween: Tween<T>) -> Self {
        Sequence {
            tweens: vec![tween],
            index: 0,
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
}

struct Tracks<T> {
    tracks: Vec<Sequence<T>>,
}

/// Component to control the animation of another component.
#[derive(Component)]
pub struct Animator<T: Component> {
    /// Control if this animation is played or not.
    pub state: AnimatorState,
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
        lens: L,
    ) -> Self
    where
        L: Lens<T> + Send + Sync + 'static,
    {
        let tween = Tween::new(ease_function, tweening_type, lens);
        Animator {
            state: AnimatorState::Playing,
            tracks: Tracks {
                tracks: vec![Sequence::from_single(tween)],
            },
        }
    }

    /// Create a new animator component from a single tween instance.
    pub fn new_single(tween: Tween<T>) -> Self {
        Animator {
            state: AnimatorState::Playing,
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
            tracks: Tracks {
                tracks: vec![Sequence::new(tweens)],
            },
        }
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
        lens: L,
    ) -> Self
    where
        L: Lens<T> + Send + Sync + 'static,
    {
        let tween = Tween::new(ease_function, tweening_type, lens);
        AssetAnimator {
            state: AnimatorState::Playing,
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
            tracks: Tracks {
                tracks: vec![Sequence::new(tweens)],
            },
            handle,
        }
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
    #[test]
    fn tween_tick() {
        let mut tween = Tween {
            ease_function: EaseMethod::Linear,
            timer: Timer::from_seconds(1.0, false),
            paused: false,
            tweening_type: TweeningType::Once {
                duration: Duration::from_secs_f32(1.0),
            },
            direction: TweeningDirection::Forward,
            lens: Box::new(TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            }),
        };
        let mut transform = Transform::default();
        tween.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform, Transform::from_translation(Vec3::splat(0.2)));
        tween.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform, Transform::from_translation(Vec3::splat(0.4)));
        tween.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform, Transform::from_translation(Vec3::splat(0.6)));
        tween.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform, Transform::from_translation(Vec3::splat(0.8)));
        tween.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform, Transform::from_translation(Vec3::splat(1.0)));
        tween.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform, Transform::from_translation(Vec3::splat(1.0)));
    }

    #[test]
    fn seq_tick() {
        let tween1 = Tween {
            ease_function: EaseMethod::Linear,
            timer: Timer::from_seconds(1.0, false),
            paused: false,
            tweening_type: TweeningType::Once {
                duration: Duration::from_secs_f32(1.0),
            },
            direction: TweeningDirection::Forward,
            lens: Box::new(TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            }),
        };
        let tween2 = Tween {
            ease_function: EaseMethod::Linear,
            timer: Timer::from_seconds(1.0, false),
            paused: false,
            tweening_type: TweeningType::Once {
                duration: Duration::from_secs_f32(1.0),
            },
            direction: TweeningDirection::Forward,
            lens: Box::new(TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_x(180_f32.to_radians()),
            }),
        };
        let mut seq = Sequence::new([tween1, tween2]);
        let mut transform = Transform::default();
        // First, translation alone (0->1)
        seq.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform, Transform::from_translation(Vec3::splat(0.2)));
        seq.tick(Duration::from_secs_f32(0.8), &mut transform);
        assert_eq!(transform, Transform::from_translation(Vec3::splat(1.0)));
        // Then, rotation alone, on top of final translation (1->2)
        seq.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform.translation, Vec3::splat(1.0));
        assert!(transform
            .rotation
            .abs_diff_eq(Quat::from_rotation_x(36_f32.to_radians()), 1e-5));
        seq.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform.translation, Vec3::splat(1.0));
        assert!(transform
            .rotation
            .abs_diff_eq(Quat::from_rotation_x(72_f32.to_radians()), 1e-5));
        seq.tick(Duration::from_secs_f32(0.6), &mut transform);
        assert_eq!(transform.translation, Vec3::splat(1.0));
        assert!(transform
            .rotation
            .abs_diff_eq(Quat::from_rotation_x(180_f32.to_radians()), 1e-5));
        seq.tick(Duration::from_secs_f32(0.2), &mut transform);
        assert_eq!(transform.translation, Vec3::splat(1.0));
        assert!(transform
            .rotation
            .abs_diff_eq(Quat::from_rotation_x(180_f32.to_radians()), 1e-5));
    }

    #[test]
    fn animator_new() {
        let animator = Animator::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong {
                duration: std::time::Duration::from_secs(1),
                pause: Some(std::time::Duration::from_millis(500)),
            },
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 2.),
            },
        );
        let tracks = animator.tracks();
        assert_eq!(tracks.tracks.len(), 1);
        let seq = &tracks.tracks[0];
        assert_eq!(seq.tweens.len(), 1);
        let tween = &seq.tweens[0];
        assert_eq!(tween.is_paused(), false);
        assert_eq!(tween.direction(), TweeningDirection::Forward);
        assert_eq!(tween.progress(), 0.);
    }

    #[test]
    fn asset_animator_new() {
        let animator = AssetAnimator::new(
            Handle::<ColorMaterial>::default(),
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong {
                duration: std::time::Duration::from_secs(1),
                pause: Some(std::time::Duration::from_millis(500)),
            },
            ColorMaterialColorLens {
                start: Color::RED,
                end: Color::BLUE,
            },
        );
        let tracks = animator.tracks();
        assert_eq!(tracks.tracks.len(), 1);
        let seq = &tracks.tracks[0];
        assert_eq!(seq.tweens.len(), 1);
        let tween = &seq.tweens[0];
        assert_eq!(tween.is_paused(), false);
        assert_eq!(tween.direction(), TweeningDirection::Forward);
        assert_eq!(tween.progress(), 0.);
    }
}
