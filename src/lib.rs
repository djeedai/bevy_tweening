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

use bevy::{asset::Asset, prelude::*};

use interpolation::Ease as IEase;
pub use interpolation::EaseFunction;
pub use interpolation::Lerp;

mod lens;
mod plugin;
mod tweenable;

pub use lens::{
    ColorMaterialColorLens, Lens, SpriteColorLens, TextColorLens, TransformPositionLens,
    TransformRotationLens, TransformScaleLens, UiPositionLens,
};
pub use plugin::{asset_animator_system, component_animator_system, TweeningPlugin};
pub use tweenable::{Delay, Sequence, Tracks, Tween, Tweenable};

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

    /// Animator::with_state()
    #[test]
    fn animator_with_state() {
        for state in [AnimatorState::Playing, AnimatorState::Paused] {
            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::PingPong,
                std::time::Duration::from_secs(1),
                TransformRotationLens {
                    start: Quat::IDENTITY,
                    end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 2.),
                },
            );
            let animator = Animator::new(tween).with_state(state);
            assert_eq!(animator.state, state);
        }
    }

    /// Animator::default() + Animator::set_tweenable()
    #[test]
    fn animator_default() {
        let mut animator = Animator::<Transform>::default();
        assert!(animator.tweenable().is_none());
        assert!(animator.tweenable_mut().is_none());

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            std::time::Duration::from_secs(1),
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_axis_angle(Vec3::Z, std::f32::consts::PI / 2.),
            },
        );
        animator.set_tweenable(tween);
        assert!(animator.tweenable().is_some());
        assert!(animator.tweenable_mut().is_some());
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

    /// AssetAnimator::with_state()
    #[test]
    fn asset_animator_with_state() {
        for state in [AnimatorState::Playing, AnimatorState::Paused] {
            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                TweeningType::PingPong,
                std::time::Duration::from_secs(1),
                ColorMaterialColorLens {
                    start: Color::RED,
                    end: Color::BLUE,
                },
            );
            let animator =
                AssetAnimator::new(Handle::<ColorMaterial>::default(), tween).with_state(state);
            assert_eq!(animator.state, state);
        }
    }

    /// AssetAnimator::default() + AssetAnimator::set_tweenable()
    #[test]
    fn asset_animator_default() {
        let mut animator = AssetAnimator::<ColorMaterial>::default();
        assert!(animator.tweenable().is_none());
        assert!(animator.tweenable_mut().is_none());
        assert_eq!(animator.handle(), Handle::<ColorMaterial>::default());

        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            TweeningType::PingPong,
            std::time::Duration::from_secs(1),
            ColorMaterialColorLens {
                start: Color::RED,
                end: Color::BLUE,
            },
        );
        animator.set_tweenable(tween);
        assert!(animator.tweenable().is_some());
        assert!(animator.tweenable_mut().is_some());
        assert_eq!(animator.handle(), Handle::<ColorMaterial>::default());
    }
}
