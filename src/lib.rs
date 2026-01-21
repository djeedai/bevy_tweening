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

//! Tweening animation plugin for the Bevy game engine.
//!
//! 🍃 Bevy Tweening provides interpolation-based animation between ("tweening")
//! two values, to animate any field of any component, resource, or asset,
//! including both built-in Bevy ones and custom user-defined ones. Each field
//! of a component, resource, or asset, can be animated via a collection of
//! predefined easing functions, or providing a custom animation curve. The
//! library supports any number of animations queued in parallel, even on the
//! same component, resource, or asset type, and allows runtime control over
//! playback and animation speed.
//!
//! # Quick start
//!
//! Look at the documentation for:
//! - [`Tween`] -- the description of a tweening animation and the explanation
//!   of some core concepts
//! - [`TweenAnim`] -- the component representing the runtime animation
//! - [`AnimTarget`] -- the component defining the target that the animation
//!   mutates
//! - [`TweeningPlugin`] -- the plugin to add to your app
//! - [`EntityCommandsTweeningExtensions`] -- the simplest way to spawn
//!   animations
//!
//! # Example
//!
//! Add the [`TweeningPlugin`] to your app:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_tweening::*;
//!
//! App::default()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(TweeningPlugin)
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
//! // Create a single animation (tween) to move an entity.
//! let tween = Tween::new(
//!     // Use a quadratic easing on both endpoints.
//!     EaseFunction::QuadraticInOut,
//!     // It takes 1 second to go from start to end points.
//!     Duration::from_secs(1),
//!     // The lens gives access to the Transform component of the Entity,
//!     // for the TweenAnimator to animate it. It also contains the start and
//!     // end values respectively associated with the progress ratios 0. and 1.
//!     TransformPositionLens {
//!         start: Vec3::ZERO,
//!         end: Vec3::new(1., 2., -4.),
//!     },
//! );
//!
//! // Spawn an entity to animate the position of.
//! commands.spawn((
//!     Transform::default(),
//!     // Create a tweenable animation targetting the current entity. Without AnimTarget,
//!     // the target is implicitly a component on this same entity. The exact component
//!     // type is derived from the type of the Lens used by the Tweenable itself.
//!     TweenAnim::new(tween),
//! ));
//! # }
//! ```
//!
//! If the target of the animation is not a component on the current entity,
//! then an [`AnimTarget`] component is necessary to specify that target. Note
//! that **[`AnimTarget`] is always mandatory for resource and asset
//! animations**.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_tweening::{lens::*, *};
//! # use std::time::Duration;
//! # fn make_tween<R: Resource>() -> Tween { unimplemented!() }
//! #[derive(Resource)]
//! struct MyResource;
//!
//! # fn system(mut commands: Commands) {
//! // Create a single animation (tween) to animate a resource.
//! let tween = make_tween::<MyResource>();
//!
//! // Spawn an entity to own the resource animation.
//! commands.spawn((
//!     TweenAnim::new(tween),
//!     // The AnimTarget is necessary here:
//!     AnimTarget::resource::<MyResource>(),
//! ));
//! # }
//! ```
//!
//! This example shows the general pattern to add animations for any component,
//! resource, or asset. Since moving the position of an object is a very common
//! task, 🍃 Bevy Tweening provides a shortcut for it. The above example can be
//! rewritten more concicely as:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_tweening::{lens::*, *};
//! # use std::time::Duration;
//! # fn system(mut commands: Commands) {
//! commands
//!     // Spawn an entity to animate the position of.
//!     .spawn((Transform::default(),))
//!     // Create a new Transform::translation animation
//!     .move_to(
//!         Vec3::new(1., 2., -4.),
//!         Duration::from_secs(1),
//!         EaseFunction::QuadraticInOut,
//!     );
//! # }
//! ```
//!
//! The [`move_to()`] extension is convenient helper for animations, which
//! creates a [`Tween`] that animates the [`Transform::translation`]. It has the
//! added benefit that the starting point is automatically read from the
//! component itself; you only need to specify the end position. See the
//! [`EntityCommandsTweeningExtensions`] extension trait defining helpers for
//! other common animations.
//!
//! # Ready to animate
//!
//! Unlike previous versions of 🍃 Bevy Tweening, **you don't need any
//! particular system setup** aside from adding the [`TweeningPlugin`] to your
//! [`App`]. In particular, per-component-type and per-asset-type systems are
//! gone. Instead, the plugin adds a _single_ system executing during the
//! [`Update`] schedule, which calls [`TweenAnim::step_all()`]. Each
//! [`TweenAnim`] acts as a controller for one animation, and mutates its
//! target.
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
//! - [`Delay`] - A time delay. This doesn't animate anything.
//!
//! To execute multiple animations in parallel (like the `Tracks` tweenable used
//! to do in older versions of 🍃 Bevy Tweening; it's now removed), simply
//! enqueue each animation independently. This require careful selection of
//! individual timings though if you want to synchronize those animations.
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
//! Note that some tweenable animations can be of infinite duration; this is the
//! case for example when using [`RepeatCount::Infinite`]. If you add such an
//! infinite animation in a sequence, and append more tweenables after it,
//! **those tweenables will never play** because playback will be stuck forever
//! repeating the first animation. You're responsible for creating sequences
//! that make sense. In general, only use infinite tweenable animations alone or
//! as the last element of a sequence (for example, move to position and then
//! rotate forever on self).
//!
//! # `TweenAnim`
//!
//! Bevy components, resources, and assets, are animated with the [`TweenAnim`]
//! component. This component acts as a controller for a single animation. It
//! determines the target component, resource, or asset, to animate, via an
//! [`AnimTarget`], and accesses the field(s) of that target using a [`Lens`].
//!
//! - Components are animated via the [`AnimTargetKind::Component`], which
//!   identifies a component instance on an entity via the [`Entity`] itself. If
//!   that target entity is the same as the one owning the [`TweenAnim`], then
//!   the [`AnimTarget`] can be omitted, for convenience.
//! - Resources are animated via the [`AnimTargetKind::Resource`].
//! - Assets are animated via the [`AnimTargetKind::Asset`] which identifies an
//!   asset via the type of its [`Assets`] collection (and so indirectly the
//!   type of asset itself) and the [`AssetId`] referencing that asset inside
//!   that collection.
//!
//! Because assets are typically shared, and the animation applies to the asset
//! itself, all users of the asset see the animation. For example, animating the
//! color of a [`ColorMaterial`] will change the color of all the 2D meshes
//! using that material. If you want to animate the color of a single mesh, you
//! need to duplicate the asset and assign a unique copy to that mesh,
//! then animate that copy alone.
//!
//! After that, you can use the [`TweenAnim`] component to control the animation
//! playback:
//!
//! ```no_run
//! # use bevy::prelude::Single;
//! # use bevy_tweening::*;
//! fn my_system(mut anim: Single<&mut TweenAnim>) {
//!     anim.speed = 0.8; // 80% playback speed
//! }
//! ```
//!
//! ## Lenses
//!
//! The [`AnimTarget`] references the target (component, resource, or asset)
//! being animated. However, only a part of that target is generally animated.
//! To that end, the [`TweenAnim`] (or, more exactly, the [`Tweenable`] it uses)
//! accesses the field(s) to animate via a _lens_, a type that implements the
//! [`Lens`] trait and allows mapping a target to the actual value(s) animated.
//!
//! For example, the [`TransformPositionLens`] uses a [`Transform`] component as
//! input, and animates its [`Transform::translation`] field only, leaving the
//! rotation and scale unchanged.
//!
//! ```no_run
//! # use bevy::{prelude::{Transform, Vec3}, ecs::change_detection::Mut};
//! # use bevy_tweening::Lens;
//! # struct TransformPositionLens { start: Vec3, end: Vec3 };
//! impl Lens<Transform> for TransformPositionLens {
//!     fn lerp(&mut self, mut target: Mut<Transform>, ratio: f32) {
//!         target.translation = self.start.lerp(self.end, ratio);
//!     }
//! }
//! ```
//!
//! Several built-in lenses are provided in the [`lens`] module for the most
//! commonly animated fields, like the components of a [`Transform`]. Those are
//! provided for convenience and mainly as examples. 🍃 Bevy Tweening expects
//! you to write your own lenses by implementing the [`Lens`] trait, which as
//! you can see above is very simple. This allows animating virtually any field
//! of any component, resource, or asset, whether shipped with Bevy or defined
//! by the user.
//!
//! # Tweening vs. keyframed animation
//!
//! 🍃 Bevy Tweening is a "tweening" animation library. It focuses on simple
//! animations often used in applications and games to breathe life into a user
//! interface or the objects of a game world. The API design favors simplicity,
//! often for quick one-shot animations created from code. This type of
//! animation is inherently simpler than a full-blown animation solution, like
//! `bevy_animation`, which typically works with complex keyframe-based
//! animation curves authored via Digital Content Creation (DCC) tools like 3D
//! modellers and exported as assets, and whose most common usage is skeletal
//! animation of characters. There's a grey area between those two approaches,
//! and you can use both to achieve most animations, but 🍃 Bevy Tweening will
//! shine for simpler animations while `bevy_animation` while offer a more
//! extensive support for larger, more complex ones.
//!
//! [`Transform::translation`]: https://docs.rs/bevy/0.17/bevy/transform/components/struct.Transform.html#structfield.translation
//! [`Entity`]: https://docs.rs/bevy/0.17/bevy/ecs/entity/struct.Entity.html
//! [`ColorMaterial`]: https://docs.rs/bevy/0.17/bevy/sprite/struct.ColorMaterial.html
//! [`Transform`]: https://docs.rs/bevy/0.17/bevy/transform/components/struct.Transform.html
//! [`TransformPositionLens`]: crate::lens::TransformPositionLens
//! [`move_to()`]: crate::EntityCommandsTweeningExtensions::move_to

use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
    time::Duration,
};

use bevy::{
    asset::UntypedAssetId,
    ecs::{
        change_detection::MutUntyped,
        component::{ComponentId, Components, Mutable},
    },
    platform::collections::HashMap,
    prelude::*,
};
pub use lens::Lens;
use lens::{
    TransformRotateAdditiveXLens, TransformRotateAdditiveYLens, TransformRotateAdditiveZLens,
};
pub use plugin::{AnimationSystem, TweeningPlugin};
use thiserror::Error;
pub use tweenable::{
    BoxedTweenable, CycleCompletedEvent, Delay, IntoBoxedTweenable, Sequence, TotalDuration, Tween,
    TweenState, Tweenable,
};

use crate::{
    lens::{TransformPositionLens, TransformScaleLens},
    tweenable::TweenConfig,
};

pub mod lens;
mod plugin;
mod tweenable;

#[cfg(test)]
mod test_utils;

/// How many times to repeat a tweenable animation.
///
/// See also [`RepeatStrategy`].
///
/// Default: `Finite(1)`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatCount {
    /// Run the animation an exact number of times.
    ///
    /// The total playback duration is the tweenable animation duration times
    /// this number of iterations.
    Finite(u32),
    /// Run the animation for some duration.
    ///
    /// If this duration is not a multiple of the tweenable animation duration,
    /// then the animation will get stopped midway through its playback,
    /// possibly even before finishing a single loop.
    For(Duration),
    /// Loop the animation indefinitely.
    Infinite,
}

impl Default for RepeatCount {
    fn default() -> Self {
        Self::Finite(1)
    }
}

impl From<u32> for RepeatCount {
    fn from(value: u32) -> Self {
        Self::Finite(value)
    }
}

impl From<Duration> for RepeatCount {
    fn from(value: Duration) -> Self {
        Self::For(value)
    }
}

impl RepeatCount {
    /// Calculate the total duration for this repeat count.
    pub fn total_duration(&self, cycle_duration: Duration) -> TotalDuration {
        match self {
            RepeatCount::Finite(count) => TotalDuration::Finite(cycle_duration * *count),
            RepeatCount::For(duration) => TotalDuration::Finite(*duration),
            RepeatCount::Infinite => TotalDuration::Infinite,
        }
    }
}

/// Repeat strategy for animation cycles.
///
/// Only applicable when [`RepeatCount`] is greater than the total duration of
/// the tweenable animation.
///
/// Default: `Repeat`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum RepeatStrategy {
    /// Reset the cycle back to its starting position.
    ///
    /// When playback reaches the end of the animation cycle, it jumps directly
    /// back to the cycle start. This can create discontinuities if the
    /// animation is not authored to be looping.
    #[default]
    Repeat,

    /// Follow a ping-pong pattern, changing the cycle direction each time an
    /// endpoint is reached.
    ///
    /// A complete loop start -> end -> start always counts as 2 cycles for the
    /// various operations where cycle count matters. That is, an animation with
    /// a 1-second cycle and a mirrored repeat strategy will take 2 seconds
    /// to end up back in the state where it started.
    ///
    /// This strategy ensures that there's no discontinuity in the animation,
    /// since there's no jump.
    MirroredRepeat,
}

/// Playback state of a [`TweenAnim`].
///
/// Default: `Playing`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// The animation is playing. This is the default state.
    #[default]
    Playing,
    /// The animation is paused in its current state.
    Paused,
}

impl std::ops::Not for PlaybackState {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Paused => Self::Playing,
            Self::Playing => Self::Paused,
        }
    }
}

/// Describe how eased value should be computed.
///
/// This function is applied to the cycle fraction `t` representing the playback
/// position over the cycle duration. The result is used to interpolate the
/// animation target.
///
/// In general a [`Lens`] should perform a linear interpolation over its target,
/// and the non-linear behavior (for example, bounciness, etc.) comes from this
/// function. This ensures the same [`Lens`] can be reused in multiple contexts,
/// while the "shape" of the animation is controlled independently.
///
/// Default: `EaseFunction::Linear`.
#[derive(Debug, Clone, Copy)]
pub enum EaseMethod {
    /// Follow [`EaseFunction`].
    EaseFunction(EaseFunction),
    /// Discrete interpolation. The eased value will jump from start to end when
    /// stepping over the discrete limit, which must be value between 0 and 1.
    Discrete(f32),
    /// Use a custom function to interpolate the value. The function is called
    /// with the cycle ratio, in `[0:1]`, as parameter, and must return the
    /// easing factor, typically also in `[0:1]`. Note that values outside this
    /// unit range may not work well with some animations; for example if
    /// animating a color, a negative red values have no meaning.
    CustomFunction(fn(f32) -> f32),
}

impl EaseMethod {
    #[must_use]
    fn sample(self, x: f32) -> f32 {
        match self {
            Self::EaseFunction(function) => EasingCurve::new(0.0, 1.0, function).sample(x).unwrap(),
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
        Self::EaseFunction(EaseFunction::Linear)
    }
}

impl From<EaseFunction> for EaseMethod {
    fn from(ease_function: EaseFunction) -> Self {
        Self::EaseFunction(ease_function)
    }
}

/// Direction a tweening animation is playing.
///
/// The playback direction determines if the delta animation time passed to
/// [`Tweenable::step()`] is added or subtracted to the current time position on
/// the animation's timeline.
/// - In `Forward` direction, time passes forward from `t=0` to the total
///   duration of the animation.
/// - Conversely, in `Backward` direction, time passes backward from the total
///   duration back to `t=0`.
///
/// Note that backward playback is supported for infinite animations (when the
/// repeat count is [`RepeatCount::Infinite`]), but [`Tweenable::rewind()`] is
/// not supported and will panic.
///
/// Default: `Forward`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackDirection {
    /// Animation playing from start to end.
    #[default]
    Forward,
    /// Animation playing from end to start, in reverse.
    Backward,
}

impl PlaybackDirection {
    /// Is the direction equal to [`PlaybackDirection::Forward`]?
    #[must_use]
    pub fn is_forward(&self) -> bool {
        *self == Self::Forward
    }

    /// Is the direction equal to [`PlaybackDirection::Backward`]?
    #[must_use]
    pub fn is_backward(&self) -> bool {
        *self == Self::Backward
    }
}

impl std::ops::Not for PlaybackDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

/// Extension trait for [`EntityCommands`], adding animation functionalities for
/// commonly used tweening animations.
///
/// This trait extends [`EntityCommands`] to provide convenience helpers to
/// common tweening animations like moving the position of an entity by
/// animating its [`Transform::translation`].
///
/// One of the major source of convenience provided by these helpers is the fact
/// that some of the data necessary to create the tween animation is
/// automatically derived from the current value of the component at the time
/// when the command is processed. For example, the [`move_to()`] helper only
/// requires specifying the end position, and will automatically read the start
/// position from the current [`Transform::translation`] value. This avoids
/// having to explicitly access that component to read that value and manually
/// store it into a [`Lens`].
///
/// [`move_to()`]: Self::move_to
pub trait EntityCommandsTweeningExtensions<'a> {
    /// Queue a new tween animation to move the current entity.
    ///
    /// The entity must have a [`Transform`] component. The tween animation will
    /// be initialized with the current [`Transform::translation`] as its
    /// starting point, and the given endpoint, duration, and ease method.
    ///
    /// Note that the starting point position is saved when the command is
    /// applied, generally after the current system when [`apply_deferred()`]
    /// runs. So any change to [`Transform::translation`] between this call and
    /// [`apply_deferred()`] will be taken into account.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands.spawn(Transform::default()).move_to(
    ///     Vec3::new(3.5, 0., 0.),
    ///     Duration::from_secs(1),
    ///     EaseFunction::QuadraticIn,
    /// );
    /// ```
    ///
    /// [`apply_deferred()`]: bevy::ecs::system::System::apply_deferred
    fn move_to(
        self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand>;

    /// Queue a new tween animation to move the current entity.
    ///
    /// The entity must have a [`Transform`] component. The tween animation will
    /// be initialized with the current [`Transform::translation`] as its
    /// ending point, and the given starting point, duration, and ease method.
    ///
    /// Note that the ending point position is saved when the command is
    /// applied, generally after the current system when [`apply_deferred()`]
    /// runs. So any change to [`Transform::translation`] between this call and
    /// [`apply_deferred()`] will be taken into account.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands.spawn(Transform::default()).move_from(
    ///     Vec3::new(3.5, 0., 0.),
    ///     Duration::from_secs(1),
    ///     EaseFunction::QuadraticIn,
    /// );
    /// ```
    ///
    /// [`apply_deferred()`]: bevy::ecs::system::System::apply_deferred
    fn move_from(
        self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand>;

    /// Queue a new tween animation to scale the current entity.
    ///
    /// The entity must have a [`Transform`] component. The tween animation will
    /// be initialized with the current [`Transform::scale`] as its starting
    /// point, and the given endpoint, duration, and ease method.
    ///
    /// Note that the starting point scale is saved when the command is applied,
    /// generally after the current system when [`apply_deferred()`]
    /// runs. So any change to [`Transform::scale`] between this call and
    /// [`apply_deferred()`] will be taken into account.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands.spawn(Transform::default()).scale_to(
    ///     Vec3::splat(2.), // 200% size
    ///     Duration::from_secs(1),
    ///     EaseFunction::QuadraticIn,
    /// );
    /// ```
    ///
    /// [`apply_deferred()`]: bevy::ecs::system::System::apply_deferred
    fn scale_to(
        self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand>;

    /// Queue a new tween animation to scale the current entity.
    ///
    /// The entity must have a [`Transform`] component. The tween animation will
    /// be initialized with the current [`Transform::scale`] as its ending
    /// point, and the given start scale, duration, and ease method.
    ///
    /// Note that the ending point scale is saved when the command is applied,
    /// generally after the current system when [`apply_deferred()`]
    /// runs. So any change to [`Transform::scale`] between this call and
    /// [`apply_deferred()`] will be taken into account.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands.spawn(Transform::default()).scale_from(
    ///     Vec3::splat(0.8), // 80% size
    ///     Duration::from_secs(1),
    ///     EaseFunction::QuadraticIn,
    /// );
    /// ```
    ///
    /// [`apply_deferred()`]: bevy::ecs::system::System::apply_deferred
    fn scale_from(
        self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand>;

    /// Queue a new tween animation to rotate the current entity around its X
    /// axis continuously (repeats forever, linearly).
    ///
    /// The entity must have a [`Transform`] component.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands
    ///     .spawn(Transform::default())
    ///     .rotate_x(Duration::from_secs(1));
    /// ```
    fn rotate_x(self, cycle_duration: Duration) -> AnimatedEntityCommands<'a, impl TweenCommand>;

    /// Queue a new tween animation to rotate the current entity around its Y
    /// axis continuously (repeats forever, linearly).
    ///
    /// The entity must have a [`Transform`] component.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands
    ///     .spawn(Transform::default())
    ///     .rotate_y(Duration::from_secs(1));
    /// ```
    fn rotate_y(self, cycle_duration: Duration) -> AnimatedEntityCommands<'a, impl TweenCommand>;

    /// Queue a new tween animation to rotate the current entity around its Z
    /// axis continuously (repeats forever, linearly).
    ///
    /// The entity must have a [`Transform`] component.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands
    ///     .spawn(Transform::default())
    ///     .rotate_z(Duration::from_secs(1));
    /// ```
    fn rotate_z(self, cycle_duration: Duration) -> AnimatedEntityCommands<'a, impl TweenCommand>;

    /// Queue a new tween animation to rotate the current entity around its X
    /// axis by a given angle.
    ///
    /// The entity must have a [`Transform`] component. The animation applies a
    /// rotation on top of the value of the [`Transform`] at the time the
    /// animation is queued.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands.spawn(Transform::default()).rotate_x_by(
    ///     std::f32::consts::FRAC_PI_4,
    ///     Duration::from_secs(1),
    ///     EaseFunction::QuadraticIn,
    /// );
    /// ```
    fn rotate_x_by(
        self,
        angle: f32,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand>;

    /// Queue a new tween animation to rotate the current entity around its Y
    /// axis by a given angle.
    ///
    /// The entity must have a [`Transform`] component. The animation applies a
    /// rotation on top of the value of the [`Transform`] at the time the
    /// animation is queued.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands.spawn(Transform::default()).rotate_y_by(
    ///     std::f32::consts::FRAC_PI_4,
    ///     Duration::from_secs(1),
    ///     EaseFunction::QuadraticIn,
    /// );
    /// ```
    fn rotate_y_by(
        self,
        angle: f32,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand>;

    /// Queue a new tween animation to rotate the current entity around its Z
    /// axis by a given angle.
    ///
    /// The entity must have a [`Transform`] component. The animation applies a
    /// rotation on top of the value of the [`Transform`] at the time the
    /// animation is queued.
    ///
    /// This function is a fire-and-forget convenience helper, and doesn't give
    /// access to the [`Entity`] created. To retrieve the entity and control
    /// the animation playback, you should spawn a [`TweenAnim`] component
    /// manually.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::*;
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// commands.spawn(Transform::default()).rotate_z_by(
    ///     std::f32::consts::FRAC_PI_4,
    ///     Duration::from_secs(1),
    ///     EaseFunction::QuadraticIn,
    /// );
    /// ```
    fn rotate_z_by(
        self,
        angle: f32,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand>;
}

/// Helper trait to abstract a tweening animation command.
///
/// This is mostly used internally by the [`AnimatedEntityCommands`] to tweak
/// the current animation, while also abstracting the various commands used to
/// implement the [`EntityCommandsTweeningExtensions`]. In general, you probably
/// don't have any use for that trait.
pub trait TweenCommand: EntityCommand {
    /// Get read-only access to the tween configuration of the command.
    #[allow(unused)]
    fn config(&self) -> &TweenConfig;

    /// Get mutable access to the tween configuration of the command.
    fn config_mut(&mut self) -> &mut TweenConfig;
}

/// Animation command to move an entity to a target position.
#[derive(Clone, Copy)]
pub(crate) struct MoveToCommand {
    end: Vec3,
    config: TweenConfig,
}

impl EntityCommand for MoveToCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(start) = entity.get::<Transform>().map(|tr| tr.translation) {
            let lens = TransformPositionLens {
                start,
                end: self.end,
            };
            let tween = Tween::from_config(self.config, lens);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for MoveToCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Animation command to move an entity from a source position.
#[derive(Clone, Copy)]
pub(crate) struct MoveFromCommand {
    start: Vec3,
    config: TweenConfig,
}

impl EntityCommand for MoveFromCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(end) = entity.get::<Transform>().map(|tr| tr.translation) {
            let lens = TransformPositionLens {
                start: self.start,
                end,
            };
            let tween = Tween::from_config(self.config, lens);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for MoveFromCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Animation command to scale an entity to a target size.
#[derive(Clone, Copy)]
pub(crate) struct ScaleToCommand {
    end: Vec3,
    config: TweenConfig,
}

impl EntityCommand for ScaleToCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(start) = entity.get::<Transform>().map(|tr| tr.scale) {
            let lens = TransformScaleLens {
                start,
                end: self.end,
            };
            let tween = Tween::from_config(self.config, lens);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for ScaleToCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Animation command to scale an entity from a source size.
#[derive(Clone, Copy)]
pub(crate) struct ScaleFromCommand {
    start: Vec3,
    config: TweenConfig,
}

impl EntityCommand for ScaleFromCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(end) = entity.get::<Transform>().map(|tr| tr.scale) {
            let lens = TransformScaleLens {
                start: self.start,
                end,
            };
            let tween = Tween::from_config(self.config, lens);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for ScaleFromCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Animation command to rotate an entity around its X axis.
#[derive(Clone, Copy)]
pub(crate) struct RotateXCommand {
    config: TweenConfig,
}

impl EntityCommand for RotateXCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(base_rotation) = entity.get::<Transform>().map(|tr| tr.rotation) {
            let lens = TransformRotateAdditiveXLens {
                base_rotation,
                start: 0.,
                end: std::f32::consts::TAU,
            };
            let tween = Tween::from_config(self.config, lens)
                .with_repeat(RepeatCount::Infinite, RepeatStrategy::Repeat);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for RotateXCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Animation command to rotate an entity around its Y axis.
#[derive(Clone, Copy)]
pub(crate) struct RotateYCommand {
    config: TweenConfig,
}

impl EntityCommand for RotateYCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(base_rotation) = entity.get::<Transform>().map(|tr| tr.rotation) {
            let lens = TransformRotateAdditiveYLens {
                base_rotation,
                start: 0.,
                end: std::f32::consts::TAU,
            };
            let tween = Tween::from_config(self.config, lens)
                .with_repeat(RepeatCount::Infinite, RepeatStrategy::Repeat);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for RotateYCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Animation command to rotate an entity around its Z axis.
#[derive(Clone, Copy)]
pub(crate) struct RotateZCommand {
    config: TweenConfig,
}

impl EntityCommand for RotateZCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(base_rotation) = entity.get::<Transform>().map(|tr| tr.rotation) {
            let lens = TransformRotateAdditiveZLens {
                base_rotation,
                start: 0.,
                end: std::f32::consts::TAU,
            };
            let tween = Tween::from_config(self.config, lens)
                .with_repeat(RepeatCount::Infinite, RepeatStrategy::Repeat);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for RotateZCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Animation command to rotate an entity around its X axis by a given angle.
#[derive(Clone, Copy)]
pub(crate) struct RotateXByCommand {
    angle: f32,
    config: TweenConfig,
}

impl EntityCommand for RotateXByCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(base_rotation) = entity.get::<Transform>().map(|tr| tr.rotation) {
            let lens = TransformRotateAdditiveXLens {
                base_rotation,
                start: 0.,
                end: self.angle,
            };
            let tween = Tween::from_config(self.config, lens);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for RotateXByCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Animation command to rotate an entity around its Y axis by a given angle.
#[derive(Clone, Copy)]
pub(crate) struct RotateYByCommand {
    angle: f32,
    config: TweenConfig,
}

impl EntityCommand for RotateYByCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(base_rotation) = entity.get::<Transform>().map(|tr| tr.rotation) {
            let lens = TransformRotateAdditiveYLens {
                base_rotation,
                start: 0.,
                end: self.angle,
            };
            let tween = Tween::from_config(self.config, lens);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for RotateYByCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Animation command to rotate an entity around its Z axis by a given angle.
#[derive(Clone, Copy)]
pub(crate) struct RotateZByCommand {
    angle: f32,
    config: TweenConfig,
}

impl EntityCommand for RotateZByCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        if let Some(base_rotation) = entity.get::<Transform>().map(|tr| tr.rotation) {
            let lens = TransformRotateAdditiveZLens {
                base_rotation,
                start: 0.,
                end: self.angle,
            };
            let tween = Tween::from_config(self.config, lens);
            let anim_target = AnimTarget::component::<Transform>(entity.id());
            entity.world_scope(|world| {
                world.spawn((TweenAnim::new(tween), anim_target));
            });
        }
    }
}

impl TweenCommand for RotateZByCommand {
    #[inline]
    fn config(&self) -> &TweenConfig {
        &self.config
    }

    #[inline]
    fn config_mut(&mut self) -> &mut TweenConfig {
        &mut self.config
    }
}

/// Wrapper over an [`EntityCommands`] which stores an animation command.
///
/// The wrapper acts as, and dereferences to, a regular [`EntityCommands`] as
/// _e.g._ returned by [`Commands::spawn()`]. In addition, it stores a pending
/// animation command, which can be further tweaked before being queued into the
/// entity commands queue. This deferred queuing allows fluent patterns like:
///
/// ```
/// # use std::time::Duration;
/// # use bevy::prelude::*;
/// # use bevy_tweening::*;
/// # fn my_system(mut commands: Commands) {
/// commands
///     .spawn(Transform::default())
///     // Consume the EntityCommands, and wrap it into an AnimatedEntityCommands,
///     // which stores an animation command to move an entity.
///     .move_to(
///         Vec3::ONE,
///         Duration::from_millis(400),
///         EaseFunction::QuadraticIn,
///     )
///     // Tweak the stored animation to set the repeat count of the Tween.
///     .with_repeat_count(2);
/// # }
/// ```
///
/// The animation commands always stores the last animation inserted. When the
/// commands is mutably dereferenced, it first flushes the pending animation
/// command, if any, by inserting it into the underlying [`EntityCommands`]
/// queue. It also flushes the animation when dropped, to ensure the last
/// animation is queued too.
///
/// To move from an [`AnimatedEntityCommands`] to its underlying
/// [`EntityCommands`], the former automatically dereferences to the latter.
/// Note however that once you're back on the base [`EntityCommands`], you can
/// only get a new [`AnimatedEntityCommands`] via functions consuming the
/// [`EntityCommands`] by value. In that case, you need to call [`reborrow()`]:
///
/// ```
/// # use std::time::Duration;
/// # use bevy::prelude::*;
/// # use bevy_tweening::*;
/// # fn my_system(mut commands: Commands) {
/// commands
///     .spawn(Transform::default())
///     .move_to(
///         Vec3::ONE,
///         Duration::from_millis(400),
///         EaseFunction::QuadraticIn,
///     )
///     // This call invokes std::ops::DerefMut, and returns a mutable ref
///     // to the underlying EntityCommands
///     .insert(Name::new("my_object"))
///     // Here we need to reborrow() to convert from `&mut EntityCommands`
///     // (by mutable ref) to `EntityCommands` (by value)
///     .reborrow()
///     // This call requires an `EntityCommands` (by value)
///     .scale_to(
///         Vec3::splat(1.1),
///         Duration::from_millis(400),
///         EaseFunction::Linear,
///     );
/// # }
/// ```
///
/// [`reborrow()`]: bevy::prelude::EntityCommands::reborrow
pub struct AnimatedEntityCommands<'a, C: TweenCommand> {
    commands: EntityCommands<'a>,
    cmd: Option<C>,
}

impl<'a, C: TweenCommand> AnimatedEntityCommands<'a, C> {
    /// Wrap an [`EntityCommands`] into an animated one.
    pub fn new(commands: EntityCommands<'a>, cmd: C) -> Self {
        Self {
            commands,
            cmd: Some(cmd),
        }
    }

    /// Set the repeat count of this animation.
    #[inline]
    pub fn with_repeat_count(mut self, repeat_count: impl Into<RepeatCount>) -> Self {
        if let Some(cmd) = self.cmd.as_mut() {
            cmd.config_mut().repeat_count = repeat_count.into();
        }
        self
    }

    /// Set the repeat strategy of this animation.
    #[inline]
    pub fn with_repeat_strategy(mut self, repeat_strategy: RepeatStrategy) -> Self {
        if let Some(cmd) = self.cmd.as_mut() {
            cmd.config_mut().repeat_strategy = repeat_strategy;
        }
        self
    }

    /// Configure the repeat parameters of this animation.
    ///
    /// This is a shortcut for:
    ///
    /// ```no_run
    /// # use bevy_tweening::*;
    /// # struct AnimatedEntityCommands {}
    /// # impl AnimatedEntityCommands {
    /// # fn with_repeat_count(self, r: RepeatCount) -> Self { unimplemented!() }
    /// # fn with_repeat_strategy(self, r: RepeatStrategy) -> Self { unimplemented!() }
    /// # fn xxx(self) -> Self {
    /// # let repeat_count = RepeatCount::Infinite;
    /// # let repeat_strategy = RepeatStrategy::Repeat;
    /// self.with_repeat_count(repeat_count)
    ///     .with_repeat_strategy(repeat_strategy)
    /// # }}
    /// ```
    #[inline]
    pub fn with_repeat(
        self,
        repeat_count: impl Into<RepeatCount>,
        repeat_strategy: RepeatStrategy,
    ) -> Self {
        self.with_repeat_count(repeat_count)
            .with_repeat_strategy(repeat_strategy)
    }

    /// Consume self and return the inner [`EntityCommands`].
    ///
    /// The current animation is inserted into the commands queue, before that
    /// wrapped commands queue is returned.
    pub fn into_inner(mut self) -> EntityCommands<'a> {
        self.flush();
        // Since we already flushed above, we don't need Drop. And trying to keep would
        // allow it to access self.commands after it was stolen (even though we know the
        // implementation doesn't in practice). Still, it's safer to just short-circuit
        // Drop here.
        let this = std::mem::ManuallyDrop::new(self);
        // SAFETY: We have flushed self.cmd which is now None, and we're stealing
        // self.commands, after which the this object is forgotten and never
        // accessed again.
        #[allow(unsafe_code)]
        unsafe {
            std::ptr::read(&this.commands)
        }
    }

    /// Flush the current animation, inserting it into the commands queue.
    ///
    /// This makes it impossible to further tweak the animation. This is
    /// automatically called when a new animation is created and when the
    /// commands queue is dropped with the last animation pending.
    fn flush(&mut self) {
        if let Some(cmd) = self.cmd.take() {
            self.queue(cmd);
        }
    }
}

impl<'a, C: TweenCommand> EntityCommandsTweeningExtensions<'a> for AnimatedEntityCommands<'a, C> {
    #[inline]
    fn move_to(
        self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().move_to(end, duration, ease_method)
    }

    #[inline]
    fn move_from(
        self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().move_from(start, duration, ease_method)
    }

    #[inline]
    fn scale_to(
        self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().scale_to(end, duration, ease_method)
    }

    #[inline]
    fn scale_from(
        self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().scale_from(start, duration, ease_method)
    }

    #[inline]
    fn rotate_x(self, cycle_duration: Duration) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().rotate_x(cycle_duration)
    }

    #[inline]
    fn rotate_y(self, cycle_duration: Duration) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().rotate_y(cycle_duration)
    }

    #[inline]
    fn rotate_z(self, cycle_duration: Duration) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().rotate_z(cycle_duration)
    }

    #[inline]
    fn rotate_x_by(
        self,
        angle: f32,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().rotate_x_by(angle, duration, ease_method)
    }

    #[inline]
    fn rotate_y_by(
        self,
        angle: f32,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().rotate_y_by(angle, duration, ease_method)
    }

    #[inline]
    fn rotate_z_by(
        self,
        angle: f32,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        self.into_inner().rotate_z_by(angle, duration, ease_method)
    }
}

impl<'a, C: TweenCommand> Deref for AnimatedEntityCommands<'a, C> {
    type Target = EntityCommands<'a>;

    fn deref(&self) -> &Self::Target {
        &self.commands
    }
}

impl<C: TweenCommand> DerefMut for AnimatedEntityCommands<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.flush();
        &mut self.commands
    }
}

impl<C: TweenCommand> Drop for AnimatedEntityCommands<'_, C> {
    fn drop(&mut self) {
        self.flush();
    }
}

impl<'a> EntityCommandsTweeningExtensions<'a> for EntityCommands<'a> {
    #[inline]
    fn move_to(
        self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            MoveToCommand {
                end,
                config: TweenConfig {
                    ease_method: ease_method.into(),
                    cycle_duration: duration,
                    ..default()
                },
            },
        )
    }

    #[inline]
    fn move_from(
        self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            MoveFromCommand {
                start,
                config: TweenConfig {
                    ease_method: ease_method.into(),
                    cycle_duration: duration,
                    ..default()
                },
            },
        )
    }

    #[inline]
    fn scale_to(
        self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            ScaleToCommand {
                end,
                config: TweenConfig {
                    ease_method: ease_method.into(),
                    cycle_duration: duration,
                    ..default()
                },
            },
        )
    }

    #[inline]
    fn scale_from(
        self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            ScaleFromCommand {
                start,
                config: TweenConfig {
                    ease_method: ease_method.into(),
                    cycle_duration: duration,
                    ..default()
                },
            },
        )
    }

    #[inline]
    fn rotate_x(self, cycle_duration: Duration) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            RotateXCommand {
                config: TweenConfig {
                    ease_method: EaseFunction::Linear.into(),
                    cycle_duration,
                    ..default()
                },
            },
        )
    }

    #[inline]
    fn rotate_y(self, cycle_duration: Duration) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            RotateYCommand {
                config: TweenConfig {
                    ease_method: EaseFunction::Linear.into(),
                    cycle_duration,
                    ..default()
                },
            },
        )
    }

    #[inline]
    fn rotate_z(self, cycle_duration: Duration) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            RotateZCommand {
                config: TweenConfig {
                    ease_method: EaseFunction::Linear.into(),
                    cycle_duration,
                    ..default()
                },
            },
        )
    }

    #[inline]
    fn rotate_x_by(
        self,
        angle: f32,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            RotateXByCommand {
                angle,
                config: TweenConfig {
                    ease_method: ease_method.into(),
                    cycle_duration: duration,
                    ..default()
                },
            },
        )
    }

    #[inline]
    fn rotate_y_by(
        self,
        angle: f32,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            RotateYByCommand {
                angle,
                config: TweenConfig {
                    ease_method: ease_method.into(),
                    cycle_duration: duration,
                    ..default()
                },
            },
        )
    }

    #[inline]
    fn rotate_z_by(
        self,
        angle: f32,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> AnimatedEntityCommands<'a, impl TweenCommand> {
        AnimatedEntityCommands::new(
            self,
            RotateZByCommand {
                angle,
                config: TweenConfig {
                    ease_method: ease_method.into(),
                    cycle_duration: duration,
                    ..default()
                },
            },
        )
    }
}

/// Event raised when a [`TweenAnim`] completed.
#[derive(Debug, Clone, Copy, EntityEvent, Message)]
pub struct AnimCompletedEvent {
    /// The entity owning the [`TweenAnim`] which completed.
    ///
    /// Note that commonly the [`TweenAnim`] is despawned on completion, so
    /// can't be queried anymore with this entity. You can prevent a completed
    /// animation from being automatically destroyed by
    /// setting [`TweenAnim::destroy_on_completion`] to `false`.
    #[event_target]
    pub anim_entity: Entity,
    /// The animation target.
    ///
    /// This is provided both as a convenience for [`TweenAnim`]s not destroyed
    /// on completion, and because for those animations which are destroyed
    /// on completion the information is not available anymore when this
    /// event is received.
    pub target: AnimTargetKind,
}

/// Errors returned by various animation functions.
#[derive(Debug, Error, Clone, Copy)]
pub enum TweeningError {
    /// The asset resolver for the given asset is not registered.
    #[error("Asset resolver for asset with resource ID {0:?} is not registered.")]
    AssetResolverNotRegistered(ComponentId),
    /// The entity was not found in the World.
    #[error("Entity {0:?} not found in the World.")]
    EntityNotFound(Entity),
    /// The entity should have had a TweenAnim but it was not found.
    #[error("Entity {0:?} doesn't have a TweenAnim.")]
    MissingTweenAnim(Entity),
    /// The component of the given type is not registered.
    #[error("Component of type {0:?} is not registered in the World.")]
    ComponentNotRegistered(TypeId),
    /// The resource of the given type is not registered.
    #[error("Resource of type {0:?} is not registered in the World.")]
    ResourceNotRegistered(TypeId),
    /// The asset container for the given asset type is not registered.
    #[error("Asset container Assets<A> for asset type A = {0:?} is not registered in the World.")]
    AssetNotRegistered(TypeId),
    /// The component of the given type is not registered.
    #[error("Component of type {0:?} is not present on entity {1:?}.")]
    MissingComponent(TypeId, Entity),
    /// The asset cannot be found.
    #[error("Asset ID {0:?} is invalid.")]
    InvalidAssetId(UntypedAssetId),
    /// The asset ID references a different type than expected.
    #[error("Expected type of asset ID to be {expected:?} but got {actual:?} instead.")]
    InvalidAssetIdType {
        /// The expected asset type.
        expected: TypeId,
        /// The actual type the asset ID references.
        actual: TypeId,
    },
    /// Expected [`Tweenable::target_type_id()`] to return a value, but it
    /// returned `None`.
    #[error("Expected a typed Tweenable.")]
    UntypedTweenable,
    /// Invalid [`Entity`].
    #[error("Invalid Entity {0:?}.")]
    InvalidTweenId(Entity),
    /// Cannot change target kind.
    #[error("Unexpected target kind: was component={0}, now component={1}")]
    MismatchingTargetKind(bool, bool),
    /// Cannot change component type.
    #[error("Cannot change component type: was component_id={0:?}, now component_id={1:?}")]
    MismatchingComponentId(ComponentId, ComponentId),
    /// Cannot change asset type.
    #[error("Cannot change asset type: was component_id={0:?}, now component_id={1:?}")]
    MismatchingAssetResourceId(ComponentId, ComponentId),
}

type RegisterAction = dyn Fn(&Components, &mut TweenResolver) + Send + Sync + 'static;

/// Enumeration of the types of animation targets.
///
/// This type holds the minimum amount of data to reference ananimation target,
/// aside from the actual type of the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnimTargetKind {
    /// Component animation target.
    Component {
        /// The entity owning the component instance.
        entity: Entity,
    },
    /// Resource animation target.
    Resource,
    /// Asset animation target.
    Asset {
        /// The asset ID inside the [`Assets`] collection.
        asset_id: UntypedAssetId,
        /// Type ID of the [`Assets`] collection itself.
        assets_type_id: TypeId,
    },
}

/// Component defining the target of an animation.
///
/// References an object used as the target of the animation stored in the
/// [`TweenAnim`] component on the same entity.
#[derive(Component)]
pub struct AnimTarget {
    /// Target kind and additional data to identify it.
    pub kind: AnimTargetKind,

    /// Self-registering action for assets and resources.
    pub(crate) register_action: Option<Box<RegisterAction>>,
}

impl AnimTarget {
    /// Create a target mutating a component on the given entity.
    pub fn component<C: Component<Mutability = Mutable>>(entity: Entity) -> Self {
        Self {
            kind: AnimTargetKind::Component { entity },
            // Components have a complete typeless API, don't need any extra registration for type
            // erasure.
            register_action: None,
        }
    }

    /// Create a target mutating the given resource.
    pub fn resource<R: Resource>() -> Self {
        let register_action = |components: &Components, resolver: &mut TweenResolver| {
            resolver.register_resource_resolver_for::<R>(components);
        };
        Self {
            kind: AnimTargetKind::Resource,
            register_action: Some(Box::new(register_action)),
        }
    }

    /// Create a target mutating the given asset.
    ///
    /// The asset is identified by its type, and its [`AssetId`].
    pub fn asset<A: Asset>(asset_id: impl Into<AssetId<A>>) -> Self {
        let register_action = |components: &Components, resolver: &mut TweenResolver| {
            resolver.register_asset_resolver_for::<A>(components);
        };
        Self {
            kind: AnimTargetKind::Asset {
                asset_id: asset_id.into().untyped(),
                assets_type_id: TypeId::of::<Assets<A>>(),
            },
            register_action: Some(Box::new(register_action)),
        }
    }

    /// Register any resolver for this target.
    pub(crate) fn register(&self, components: &Components, resolver: &mut TweenResolver) {
        if let Some(register_action) = self.register_action.as_ref() {
            register_action(components, resolver);
        }
    }
}

/// Animation controller instance.
///
/// The [`TweenAnim`] represents a single animation instance for a single
/// target (component or resource or asset). Each instance is independent, even
/// if it mutates the same target as another instance. Spawning this component
/// adds an active animation, and destroying it stops that animation. The
/// component can also be used to control the animation playback at runtime,
/// like the playback speed.
///
/// The target is described by the [`AnimTarget`] component. If that component
/// is absent, then the animation implicitly targets a component on the current
/// Entity. The type of the component is derived from the type that the [`Lens`]
/// animates.
///
/// _If you're looking for the basic tweenable animation description, see
/// [`Tween`] instead._
///
/// # Example
///
/// ```
/// # use bevy::prelude::*;
/// # use bevy_tweening::*;
/// # fn make_tweenable<T>() -> Tween { unimplemented!() }
/// fn my_system(mut commands: Commands) {
///     let tweenable = make_tweenable::<Transform>();
///     let id1 = commands
///         .spawn((
///             Transform::default(),
///             // Implicitly targets the current entity's Transform
///             TweenAnim::new(tweenable),
///         ))
///         .id();
///
///     let tweenable2 = make_tweenable::<Transform>();
///     commands.spawn((
///         TweenAnim::new(tweenable2),
///         // Explicitly targets the Transform component of entity 'id1'
///         AnimTarget::component::<Transform>(id1),
///     ));
/// }
/// ```
#[derive(Component)]
pub struct TweenAnim {
    /// The animation itself. Note that the tweenable is stateful, so can't be
    /// shared with another [`TweenAnim`] instance.
    tweenable: BoxedTweenable,
    /// Control if the animation is played or not. Defaults to
    /// [`PlaybackState::Playing`].
    ///
    /// Pausing an animation with [`PlaybackState::Paused`] is functionaly
    /// equivalent to setting its [`speed`] to zero. The two fields remain
    /// independent though, for convenience.
    ///
    /// [`speed`]: Self::speed
    pub playback_state: PlaybackState,
    /// Relative playback speed. Defaults to `1.` (normal speed; 100%).
    ///
    /// Setting a negative or zero speed value effectively pauses the animation
    /// (although the [`playback_state`] remains unchanged). Negative values may
    /// be clamped to 0. when the animation is stepped, but positive or zero
    /// values are never modified by the library.
    ///
    /// # Time precision
    ///
    /// _This note is an implementation detail which can usually be ignored._
    ///
    /// Despite the use of `f64`, setting a playback speed different from `1.`
    /// (100% speed) may produce small inaccuracies in durations, especially
    /// for longer animations. However those are often negligible.
    /// This is due to the very large precision of `Duration` (typically 96
    /// bits or more), even compared to `f64` (64 bits), and the fact this speed
    /// factor is a multiplier whereas most other time quantities are added or
    /// subtracted.
    ///
    /// [`playback_state`]: Self::playback_state
    pub speed: f64,
    /// Destroy the animation once completed. This defaults to `true`, and makes
    /// the stepping functions like [`TweenAnim::step_all()`] destroy this
    /// animation once it completed. To keep the animation queued, and allow
    /// access after it completed, set this to `false`. Note however that
    /// you should avoid leaving all animations queued if they're unused, as
    /// this wastes memory and may degrade performances if too many
    /// completed animations are kept around for no good reason.
    pub destroy_on_completion: bool,
    /// Current tweening completion state.
    tween_state: TweenState,
}

impl TweenAnim {
    /// Create a new tween animation.
    ///
    /// This component represents the runtime animation being played to mutate a
    /// specific target.
    ///
    /// # Panics
    ///
    /// Panics if the tweenable is "typeless", that is
    /// [`Tweenable::target_type_id()`] returns `None`. Animations must
    /// target a concrete component or asset type. This means in particular
    /// that you can't use a single [`Delay`] alone. You can however use a
    /// [`Delay`] or other typeless tweenables as part of a [`Sequence`],
    /// provided there's at least one other typed tweenable in the sequence
    /// to make it typed too.
    #[inline]
    pub fn new(tweenable: impl IntoBoxedTweenable) -> Self {
        let tweenable = tweenable.into_boxed();
        assert!(
            tweenable.target_type_id().is_some(),
            "The top-level Tweenable of a TweenAnim must be typed (Tweenable::target_type_id() returns Some)."
        );
        Self {
            tweenable,
            playback_state: PlaybackState::Playing,
            speed: 1.,
            destroy_on_completion: true,
            tween_state: TweenState::Active,
        }
    }

    /// Configure the playback speed.
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Enable or disable destroying this component on animation completion.
    ///
    /// If enabled, the component is automatically removed from its `Entity`
    /// when the animation completed.
    pub fn with_destroy_on_completed(mut self, destroy_on_completed: bool) -> Self {
        self.destroy_on_completion = destroy_on_completed;
        self
    }

    /// Step a single animation.
    ///
    /// _The [`step_all()`] function is called automatically by the animation
    /// system registered by the [`TweeningPlugin`], you generally don't
    /// need to call this one._
    ///
    /// This is a shortcut for `step_many(world, delta_time, [entity])`, with
    /// the added benefit that it returns some error if the entity is not valid.
    /// See [`step_many()`] for details.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::time::Duration;
    /// # use bevy::prelude::*;
    /// # use bevy_tweening::*;
    /// # fn make_tweenable() -> Tween { unimplemented!() }
    /// #[derive(Component)]
    /// struct MyMarker;
    ///
    /// fn my_system(world: &mut World) -> Result<()> {
    ///     let mut q_anims = world.query_filtered::<Entity, (With<TweenAnim>, With<MyMarker>)>();
    ///     let entity = q_anims.single(world)?;
    ///     let delta_time = Duration::from_millis(200);
    ///     TweenAnim::step_one(world, delta_time, entity);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// This returns an error if the entity is not found or doesn't own a
    /// [`TweenAnim`] component.
    ///
    /// [`step_all()`]: Self::step_all
    /// [`step_many()`]: Self::step_many
    #[inline]
    pub fn step_one(
        world: &mut World,
        delta_time: Duration,
        entity: Entity,
    ) -> Result<(), TweeningError> {
        let num = Self::step_many(world, delta_time, &[entity]);
        if num > 0 {
            Ok(())
        } else {
            Err(TweeningError::EntityNotFound(entity))
        }
    }

    /// Step some animation(s).
    ///
    /// _The [`step_all()`] function is called automatically by the animation
    /// system registered by the [`TweeningPlugin`], you generally don't
    /// need to call this one._
    ///
    /// Step the given animation(s) by a given `delta_time`, which may be
    /// [`Duration::ZERO`]. Passing a zero delta time may be useful to force the
    /// current animation state to be applied to a target, in case you made
    /// change which do not automatically do so (for example, retargeting an
    /// animation). The `anims` are the entities which own a [`TweenAnim`]
    /// component to step; any entity without a [`TweenAnim`] component is
    /// silently ignored.
    ///
    /// The function doesn't check that all input entities are unique. If an
    /// entity is duplicated in `anims`, the behavior is undefined, including
    /// (but not guaranteed) stepping the animation multiple times. You're
    /// responsible for ensuring the input entity slice contains distinct
    /// entities.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::time::Duration;
    /// # use bevy::prelude::*;
    /// # use bevy_tweening::*;
    /// # fn make_tweenable() -> Tween { unimplemented!() }
    /// #[derive(Component)]
    /// struct MyMarker;
    ///
    /// fn my_system(world: &mut World) -> Result<()> {
    ///     let mut q_anims = world.query_filtered::<Entity, (With<TweenAnim>, With<MyMarker>)>();
    ///     let entities = q_anims.iter(world).collect::<Vec<Entity>>();
    ///     let delta_time = Duration::from_millis(200);
    ///     TweenAnim::step_many(world, delta_time, &entities[..]);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns the number of [`TweenAnim`] component found and stepped, which
    /// is always less than or equal to the input `anims` slice length.
    ///
    /// [`step_all()`]: Self::step_all
    pub fn step_many(world: &mut World, delta_time: Duration, anims: &[Entity]) -> usize {
        let mut targets = vec![];
        world.resource_scope(|world, mut resolver: Mut<TweenResolver>| {
            let mut q_anims = world.query::<(Entity, &TweenAnim, Option<&AnimTarget>)>();
            targets.reserve(anims.len());
            for entity in anims {
                if let Ok((entity, anim, maybe_target)) = q_anims.get(world, *entity) {
                    // Lazy registration with resolver if needed
                    if let Some(anim_target) = maybe_target {
                        anim_target.register(world.components(), &mut resolver);
                    }

                    // Actually step the tweenable and update the target
                    if let Ok((target_type_id, component_id, target, is_retargetable)) =
                        Self::resolve_target(
                            world.components(),
                            maybe_target,
                            entity,
                            anim.tweenable(),
                        )
                    {
                        targets.push((
                            entity,
                            target_type_id,
                            component_id,
                            target,
                            is_retargetable,
                        ));
                    }
                }
            }
        });
        Self::step_impl(world, delta_time, &targets[..]);
        targets.len()
    }

    /// Step all animations on the given world.
    ///
    /// _This function is called automatically by the animation system
    /// registered by the [`TweeningPlugin`], you generally don't need to call
    /// it._
    ///
    /// Step all the [`TweenAnim`] components of the input world by a given
    /// `delta_time`, which may be [`Duration::ZERO`]. Passing a zero delta
    /// time may be useful to force the current animation state to be
    /// applied to a target, in case you made change which do not
    /// automatically do so (for example, retargeting an animation).
    pub fn step_all(world: &mut World, delta_time: Duration) {
        let targets = world.resource_scope(|world, mut resolver: Mut<TweenResolver>| {
            let mut q_anims = world.query::<(Entity, &TweenAnim, Option<&AnimTarget>)>();
            q_anims
                .iter(world)
                .filter_map(|(entity, anim, maybe_target)| {
                    // Lazy registration with resolver if needed
                    if let Some(anim_target) = maybe_target {
                        anim_target.register(world.components(), &mut resolver);
                    }

                    // Actually step the tweenable and update the target
                    match Self::resolve_target(
                        world.components(),
                        maybe_target,
                        entity,
                        anim.tweenable(),
                    ) {
                        Ok((target_type_id, component_id, target, is_retargetable)) => Some((
                            entity,
                            target_type_id,
                            component_id,
                            target,
                            is_retargetable,
                        )),
                        Err(err) => {
                            bevy::log::error!(
                                "Error while stepping TweenAnim on entity {:?}: {:?}",
                                entity,
                                err
                            );
                            None
                        }
                    }
                })
                .collect::<Vec<_>>()
        });
        Self::step_impl(world, delta_time, &targets[..]);
    }

    fn resolve_target(
        components: &Components,
        maybe_target: Option<&AnimTarget>,
        anim_entity: Entity,
        tweenable: &dyn Tweenable,
    ) -> Result<(TypeId, ComponentId, AnimTargetKind, bool), TweeningError> {
        let type_id = tweenable
            .target_type_id()
            .ok_or(TweeningError::UntypedTweenable)?;
        if let Some(target) = maybe_target {
            // Target explicitly specified with AnimTarget component
            let component_id = match &target.kind {
                AnimTargetKind::Component { .. } => components
                    .get_id(type_id)
                    .ok_or(TweeningError::ComponentNotRegistered(type_id))?,
                AnimTargetKind::Resource => components
                    .get_resource_id(type_id)
                    .ok_or(TweeningError::ResourceNotRegistered(type_id))?,
                AnimTargetKind::Asset { assets_type_id, .. } => components
                    .get_resource_id(*assets_type_id)
                    .ok_or(TweeningError::AssetNotRegistered(type_id))?,
            };
            let is_retargetable = false; // explicit target
            Ok((type_id, component_id, target.kind, is_retargetable))
        } else {
            // Target implicitly self; this can only be a component target
            let is_retargetable = true;
            if let Some(component_id) = components.get_id(type_id) {
                Ok((
                    type_id,
                    component_id,
                    AnimTargetKind::Component {
                        entity: anim_entity,
                    },
                    is_retargetable,
                ))
            } else {
                // We can't implicitly target an asset without its AssetId
                Err(TweeningError::ComponentNotRegistered(type_id))
            }
        }
    }

    fn step_impl(
        world: &mut World,
        delta_time: Duration,
        anims: &[(Entity, TypeId, ComponentId, AnimTargetKind, bool)],
    ) {
        let mut to_remove = Vec::with_capacity(anims.len());
        world.resource_scope(|world, resolver: Mut<TweenResolver>| {
            world.resource_scope(
                |world, mut cycle_events: Mut<Messages<CycleCompletedEvent>>| {
                    world.resource_scope(
                        |world, mut anim_events: Mut<Messages<AnimCompletedEvent>>| {
                            let anim_comp_id = world.component_id::<TweenAnim>().unwrap();
                            for (
                                anim_entity,
                                target_type_id,
                                component_id,
                                anim_target,
                                is_retargetable,
                            ) in anims
                            {
                                let retain = match anim_target {
                                    AnimTargetKind::Component {
                                        entity: comp_entity,
                                    } => {
                                        let (mut entities, commands) =
                                            world.entities_and_commands();
                                        let ret = if *anim_entity == *comp_entity {
                                            // The TweenAnim animates another component on the same
                                            // entity
                                            let Ok([mut ent]) = entities.get_mut([*anim_entity])
                                            else {
                                                continue;
                                            };
                                            let Ok([anim, target]) =
                                                ent.get_mut_by_id([anim_comp_id, *component_id])
                                            else {
                                                continue;
                                            };
                                            // SAFETY: We fetched the EntityMut from the component
                                            // ID of
                                            // TweenAnim
                                            #[allow(unsafe_code)]
                                            let mut anim = unsafe { anim.with_type::<TweenAnim>() };
                                            anim.step_self(
                                                commands,
                                                *anim_entity,
                                                delta_time,
                                                anim_target,
                                                target,
                                                target_type_id,
                                                cycle_events.reborrow(),
                                                anim_events.reborrow(),
                                            )
                                        } else {
                                            // The TweenAnim animates a component on a different
                                            // entity
                                            let Ok([mut anim, mut target]) =
                                                entities.get_mut([*anim_entity, *comp_entity])
                                            else {
                                                continue;
                                            };
                                            let Some(mut anim) = anim.get_mut::<TweenAnim>() else {
                                                continue;
                                            };
                                            let Ok(target) = target.get_mut_by_id(*component_id)
                                            else {
                                                continue;
                                            };
                                            anim.step_self(
                                                commands,
                                                *anim_entity,
                                                delta_time,
                                                anim_target,
                                                target,
                                                target_type_id,
                                                cycle_events.reborrow(),
                                                anim_events.reborrow(),
                                            )
                                        };
                                        match ret {
                                            Ok(res) => {
                                                if res.needs_retarget {
                                                    assert!(res.retain);
                                                    if *is_retargetable {
                                                        //to_retarget.push(anim_entity);
                                                        //true
                                                        bevy::log::warn!("TODO: Multi-target tweenable sequence is not yet supported. Ensure the animation of the TweenAnim component on entity {:?} targets a single component type.", *anim_entity);
                                                        false
                                                    } else {
                                                        bevy::log::warn!("Multi-target tweenable sequence cannot be used with an explicit single target. Remove the AnimTarget component from entity {:?}, or ensure all tweenables in the sequence target the same component.", *anim_entity);
                                                        false
                                                    }
                                                } else {
                                                    res.retain
                                                }
                                            }
                                            Err(_) => false,
                                        }
                                    }
                                    AnimTargetKind::Resource => resolver
                                        .resolve_resource(
                                            world,
                                            target_type_id,
                                            *component_id,
                                            *anim_entity,
                                            delta_time,
                                            cycle_events.reborrow(),
                                            anim_events.reborrow(),
                                        )
                                        .unwrap_or_else(|err| {
                                            bevy::log::error!(
                                                "Deleting resource animation due to error: {err:?}"
                                            );
                                            false
                                        }),
                                    AnimTargetKind::Asset { asset_id, .. } => resolver
                                        .resolve_asset(
                                            world,
                                            target_type_id,
                                            *component_id,
                                            *asset_id,
                                            *anim_entity,
                                            delta_time,
                                            cycle_events.reborrow(),
                                            anim_events.reborrow(),
                                        )
                                        .unwrap_or_else(|err| {
                                            bevy::log::error!(
                                                "Deleting asset animation due to error: {err:?}"
                                            );
                                            false
                                        }),
                                };

                                if !retain {
                                    to_remove.push(*anim_entity);
                                }
                            }
                        },
                    );
                },
            );
        });

        let mut cmds = world.commands();
        for entity in to_remove.drain(..) {
            cmds.entity(entity).try_remove::<TweenAnim>();
        }

        world.flush();
    }

    #[allow(clippy::too_many_arguments)]
    fn step_self(
        &mut self,
        mut commands: Commands,
        anim_entity: Entity,
        delta_time: Duration,
        target_kind: &AnimTargetKind,
        mut mut_untyped: MutUntyped,
        target_type_id: &TypeId,
        mut cycle_events: Mut<Messages<CycleCompletedEvent>>,
        mut anim_events: Mut<Messages<AnimCompletedEvent>>,
    ) -> Result<StepResult, TweeningError> {
        let mut completed_events = Vec::with_capacity(8);

        // Sanity checks on fields which can be freely modified by the user
        self.speed = self.speed.max(0.);

        // Retain completed animations only if requested
        if self.tween_state == TweenState::Completed {
            let ret = StepResult {
                retain: !self.destroy_on_completion,
                needs_retarget: false,
            };
            return Ok(ret);
        }

        // Skip paused animations (but retain them)
        if self.playback_state == PlaybackState::Paused || self.speed <= 0. {
            let ret = StepResult {
                retain: true,
                needs_retarget: false,
            };
            return Ok(ret);
        }

        // Scale delta time by this animation's speed. Reject negative speeds; use
        // backward playback to play in reverse direction.
        // Note: must use f64 for precision; f32 produces visible roundings.
        let delta_time = delta_time.mul_f64(self.speed);

        // Step the tweenable animation
        let mut notify_completed = || {
            completed_events.push(CycleCompletedEvent {
                anim_entity,
                target: *target_kind,
            });
        };
        let (state, needs_retarget) = self.tweenable.step(
            anim_entity,
            delta_time,
            mut_untyped.reborrow(),
            target_type_id,
            &mut notify_completed,
        );
        self.tween_state = state;

        // Send tween completed events once we reclaimed mut access to world and can get
        // a Commands.
        if !completed_events.is_empty() {
            for event in completed_events.drain(..) {
                // Send buffered event
                cycle_events.write(event);

                // Trigger all entity-scoped observers
                commands.trigger(CycleCompletedEvent {
                    anim_entity,
                    ..event
                });
            }
        }

        // Raise animation completed event
        if state == TweenState::Completed {
            let event: AnimCompletedEvent = AnimCompletedEvent {
                anim_entity,
                target: *target_kind,
            };

            // Send buffered event
            anim_events.write(event);

            // Trigger all entity-scoped observers
            commands.trigger(event);
        }

        let ret = StepResult {
            retain: state == TweenState::Active || !self.destroy_on_completion,
            needs_retarget,
        };
        Ok(ret)
    }

    /// Stop animation playback and rewind the animation.
    ///
    /// This changes the animator state to [`PlaybackState::Paused`] and rewinds
    /// its tweenable.
    ///
    /// # Panics
    ///
    /// Like [`Tweenable::rewind()`], this panics if the current playback
    /// direction is [`PlaybackDirection::Backward`] and the animation is
    /// infinitely repeating.
    pub fn stop(&mut self) {
        self.playback_state = PlaybackState::Paused;
        self.tweenable.rewind();
        self.tween_state = TweenState::Active;
    }

    /// Get the tweenable describing this animation.
    ///
    /// To change the tweenable, use [`TweenAnim::set_tweenable()`].
    #[inline]
    pub fn tweenable(&self) -> &dyn Tweenable {
        self.tweenable.as_ref()
    }

    /// Set a new animation description.
    ///
    /// Attempt to change the tweenable of an animation already spawned.
    ///
    /// If the tweenable is successfully swapped, this resets the
    /// [`tween_state()`] to [`TweenState::Active`], even if the tweenable would
    /// otherwise be completed _e.g._ because its current elapsed time is past
    /// its total duration. Conversely, this doesn't update the target
    /// component or asset, as this function doesn't have mutable access to
    /// it. To force applying the new state to the target without stepping the
    /// animation forward or backward, call one of the stepping functions like
    /// [`TweenAnim::step_one()`] passing a delta time of [`Duration::ZERO`].
    ///
    /// To ensure the old and new animations have the same elapsed time (for
    /// example if they need to be synchronized, if they're variants of each
    /// other), call [`set_elapsed()`] first on the input `tweenable`, with
    /// the duration value of the old tweenable returned by [`elapsed()`].
    ///
    /// ```
    /// # use std::time::Duration;
    /// # use bevy::prelude::*;
    /// # use bevy_tweening::*;
    /// # fn make_tweenable() -> Tween { unimplemented!() }
    /// fn my_system(mut anim: Single<&mut TweenAnim>) {
    ///     let mut tweenable = make_tweenable();
    ///     let elapsed = anim.tweenable().elapsed();
    ///     tweenable.set_elapsed(elapsed);
    ///     anim.set_tweenable(tweenable);
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// On success, returns the previous tweenable which has been swapped out.
    ///
    /// [`tween_state()`]: Self::tween_state
    /// [`set_elapsed()`]: crate::Tweenable::set_elapsed
    /// [`elapsed()`]: crate::Tweenable::elapsed
    /// [`step_one()`]: Self::step_one
    pub fn set_tweenable<T>(&mut self, tweenable: T) -> Result<BoxedTweenable, TweeningError>
    where
        T: Tweenable + 'static,
    {
        let mut old_tweenable: BoxedTweenable = Box::new(tweenable);
        std::mem::swap(&mut self.tweenable, &mut old_tweenable);
        // Reset tweening state, the new tweenable is at t=0
        self.tween_state = TweenState::Active;
        Ok(old_tweenable)
    }

    /// Get the tweening completion state.
    ///
    /// In general this is [`TweenState::Active`], unless the animation
    /// completed and [`destroy_on_completion`] is `false`.
    ///
    /// [`destroy_on_completion`]: Self::destroy_on_completion
    #[inline]
    pub fn tween_state(&self) -> TweenState {
        self.tween_state
    }
}

type ResourceResolver = Box<
    dyn for<'w> Fn(
            &mut World,
            Entity,
            &TypeId,
            Duration,
            Mut<Messages<CycleCompletedEvent>>,
            Mut<Messages<AnimCompletedEvent>>,
        ) -> Result<bool, TweeningError>
        + Send
        + Sync
        + 'static,
>;

type AssetResolver = Box<
    dyn for<'w> Fn(
            &mut World,
            UntypedAssetId,
            Entity,
            &TypeId,
            Duration,
            Mut<Messages<CycleCompletedEvent>>,
            Mut<Messages<AnimCompletedEvent>>,
        ) -> Result<bool, TweeningError>
        + Send
        + Sync
        + 'static,
>;

/// Resolver for resources and assets.
///
/// _This resource is largely an implementation detail. You can safely ignore
/// it._
///
/// Bevy doesn't provide a suitable untyped API to access resources and assets
/// at runtime without knowing their compile-time type.
/// - For resources, most of the API is in place, but unfortunately there's no
///   `World::resource_scope_untyped()` to temporarily extract a resource by ID
///   to allow concurrent mutability of the resource with other parts of the
///   [`World`], in particular the animation target.
/// - For assets, there's simply no untyped API. [`Assets`] doesn't allow
///   untyped asset access.
///
/// To work around those limitations, this resolver resource contains
/// type-erased closures allowing to resolve an animation target definition into
/// a mutable pointer [`MutUntyped`] to that instance, to allow the animation
/// engine to apply the animation on it.
#[derive(Default, Resource)]
pub struct TweenResolver {
    /// Resource resolver allowing to call `World::resource_scope()` to extract
    /// that resource type form the `World` while in parallel accessing mutably
    /// the animation entity itself.
    resource_resolver: HashMap<ComponentId, ResourceResolver>,
    /// Asset resolver allowing to convert a pair of { untyped pointer to
    /// `Assets<A>`, untyped `AssetId` } into an untyped pointer to the asset A
    /// itself. This is necessary because there's no UntypedAssets interface in
    /// Bevy. The TypeId key must be the type of the `Assets<A>` type itself.
    /// The resolver is allowed to fail (return `None`), for example when the
    /// asset ID doesn't reference a valid asset.
    asset_resolver: HashMap<ComponentId, AssetResolver>,
}

impl TweenResolver {
    /// Register a resolver for the given resource type.
    pub(crate) fn register_resource_resolver_for<R: Resource>(&mut self, components: &Components) {
        let resource_id = components.resource_id::<R>().unwrap();
        let resolver = |world: &mut World,
                        entity: Entity,
                        target_type_id: &TypeId,
                        delta_time: Duration,
                        mut cycle_events: Mut<Messages<CycleCompletedEvent>>,
                        mut anim_events: Mut<Messages<AnimCompletedEvent>>|
         -> Result<bool, TweeningError> {
            // First, remove the resource R from the world so we can access it mutably in
            // parallel of the TweenAnim
            world.resource_scope(|world, resource: Mut<R>| {
                let target = AnimTargetKind::Resource;

                let (mut entities, commands) = world.entities_and_commands();

                // Resolve the TweenAnim component
                let Ok([mut ent]) = entities.get_mut([entity]) else {
                    return Err(TweeningError::EntityNotFound(entity));
                };
                let Some(mut anim) = ent.get_mut::<TweenAnim>() else {
                    return Err(TweeningError::MissingTweenAnim(ent.id()));
                };

                // Finally, step the TweenAnim and mutate the target
                let ret = anim.step_self(
                    commands,
                    entity,
                    delta_time,
                    &target,
                    resource.into(),
                    target_type_id,
                    cycle_events.reborrow(),
                    anim_events.reborrow(),
                );
                ret.map(|result| {
                    assert!(!result.needs_retarget, "Cannot use a multi-target sequence of tweenable animations with a resource target.");
                    result.retain
                })
            })
        };
        self.resource_resolver
            .entry(resource_id)
            .or_insert(Box::new(resolver));
    }

    /// Register a resolver for the given asset type.
    pub(crate) fn register_asset_resolver_for<A: Asset>(&mut self, components: &Components) {
        let resource_id = components.resource_id::<Assets<A>>().unwrap();
        let resolver = |world: &mut World,
                        asset_id: UntypedAssetId,
                        entity: Entity,
                        target_type_id: &TypeId,
                        delta_time: Duration,
                        mut cycle_events: Mut<Messages<CycleCompletedEvent>>,
                        mut anim_events: Mut<Messages<AnimCompletedEvent>>|
         -> Result<bool, TweeningError> {
            let asset_id = asset_id.typed::<A>();
            // First, remove the Assets<A> from the world so we can access it mutably in
            // parallel of the TweenAnim
            world.resource_scope(|world, assets: Mut<Assets<A>>| {
                // Next, fetch the asset A itself from its Assets<A> based on its asset ID
                let Some(asset) = assets.filter_map_unchanged(|assets| assets.get_mut(asset_id))
                else {
                    return Err(TweeningError::InvalidAssetId(asset_id.into()));
                };

                let target = AnimTargetKind::Asset {
                    asset_id: asset_id.untyped(),
                    assets_type_id: TypeId::of::<Assets<A>>(),
                };

                let (mut entities, commands) = world.entities_and_commands();

                // Resolve the TweenAnim component
                let Ok([mut ent]) = entities.get_mut([entity]) else {
                    return Err(TweeningError::EntityNotFound(entity));
                };
                let Some(mut anim) = ent.get_mut::<TweenAnim>() else {
                    return Err(TweeningError::MissingTweenAnim(ent.id()));
                };

                // Finally, step the TweenAnim and mutate the target
                let ret = anim.step_self(
                    commands,
                    entity,
                    delta_time,
                    &target,
                    asset.into(),
                    target_type_id,
                    cycle_events.reborrow(),
                    anim_events.reborrow(),
                );
                ret.map(|result| {
                    assert!(!result.needs_retarget, "Cannot use a multi-target sequence of tweenable animations with an asset target.");
                    result.retain
                })
            })
        };
        self.asset_resolver
            .entry(resource_id)
            .or_insert(Box::new(resolver));
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub(crate) fn resolve_resource(
        &self,
        world: &mut World,
        target_type_id: &TypeId,
        resource_id: ComponentId,
        entity: Entity,
        delta_time: Duration,
        cycle_events: Mut<Messages<CycleCompletedEvent>>,
        anim_events: Mut<Messages<AnimCompletedEvent>>,
    ) -> Result<bool, TweeningError> {
        let Some(resolver) = self.resource_resolver.get(&resource_id) else {
            println!("ERROR: resource not registered {:?}", resource_id);
            return Err(TweeningError::AssetResolverNotRegistered(resource_id));
        };
        resolver(
            world,
            entity,
            target_type_id,
            delta_time,
            cycle_events,
            anim_events,
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub(crate) fn resolve_asset(
        &self,
        world: &mut World,
        target_type_id: &TypeId,
        resource_id: ComponentId,
        untyped_asset_id: UntypedAssetId,
        entity: Entity,
        delta_time: Duration,
        cycle_events: Mut<Messages<CycleCompletedEvent>>,
        anim_events: Mut<Messages<AnimCompletedEvent>>,
    ) -> Result<bool, TweeningError> {
        let Some(resolver) = self.asset_resolver.get(&resource_id) else {
            println!("ERROR: asset not registered {:?}", resource_id);
            return Err(TweeningError::AssetResolverNotRegistered(resource_id));
        };
        resolver(
            world,
            untyped_asset_id,
            entity,
            target_type_id,
            delta_time,
            cycle_events,
            anim_events,
        )
    }
}

pub(crate) struct StepResult {
    /// Whether to retain the current [`TweenAnim`]? If `false`, the
    /// [`TweenAnim`] is destroyed unless [`TweenAnim::destroy_on_completion`]
    /// is `false`.
    pub retain: bool,
    /// Whether to recompute the new animation target and step again. This is
    /// used by sequences when the animation target changes type in a sequence.
    pub needs_retarget: bool,
}

#[cfg(test)]
mod tests {
    use std::{
        f32::consts::{FRAC_PI_2, TAU},
        marker::PhantomData,
    };

    use bevy::ecs::{change_detection::MaybeLocation, component::Tick};

    use super::*;
    use crate::test_utils::*;

    struct DummyLens {
        start: f32,
        end: f32,
    }

    struct DummyLens2 {
        start: i32,
        end: i32,
    }

    #[derive(Debug, Default, Clone, Copy, Component)]
    struct DummyComponent {
        value: f32,
    }

    #[derive(Debug, Default, Clone, Copy, Component)]
    struct DummyComponent2 {
        value: i32,
    }

    #[derive(Debug, Default, Clone, Copy, Resource)]
    struct DummyResource {
        value: f32,
    }

    #[derive(Asset, Debug, Default, Reflect)]
    struct DummyAsset {
        value: f32,
    }

    impl Lens<DummyComponent> for DummyLens {
        fn lerp(&mut self, mut target: Mut<DummyComponent>, ratio: f32) {
            target.value = self.start.lerp(self.end, ratio);
        }
    }

    impl Lens<DummyComponent2> for DummyLens2 {
        fn lerp(&mut self, mut target: Mut<DummyComponent2>, ratio: f32) {
            target.value = ((self.start as f32) * (1. - ratio) + (self.end as f32) * ratio) as i32;
        }
    }

    #[test]
    fn dummy_lens_component() {
        let mut c = DummyComponent::default();
        let mut l = DummyLens { start: 0., end: 1. };
        for r in [0_f32, 0.01, 0.3, 0.5, 0.9, 0.999, 1.] {
            {
                let mut added = Tick::new(0);
                let mut last_changed = Tick::new(0);
                let mut caller = MaybeLocation::caller();
                let mut target = Mut::new(
                    &mut c,
                    &mut added,
                    &mut last_changed,
                    Tick::new(0),
                    Tick::new(1),
                    caller.as_mut(),
                );

                l.lerp(target.reborrow(), r);

                assert!(target.is_changed());
            }
            assert_approx_eq!(c.value, r);
        }
    }

    impl Lens<DummyResource> for DummyLens {
        fn lerp(&mut self, mut target: Mut<DummyResource>, ratio: f32) {
            target.value = self.start.lerp(self.end, ratio);
        }
    }

    #[test]
    fn dummy_lens_resource() {
        let mut res = DummyResource::default();
        let mut l = DummyLens { start: 0., end: 1. };
        for r in [0_f32, 0.01, 0.3, 0.5, 0.9, 0.999, 1.] {
            {
                let mut added = Tick::new(0);
                let mut last_changed = Tick::new(0);
                let mut caller = MaybeLocation::caller();
                let mut target = Mut::new(
                    &mut res,
                    &mut added,
                    &mut last_changed,
                    Tick::new(0),
                    Tick::new(0),
                    caller.as_mut(),
                );
                l.lerp(target.reborrow(), r);
            }
            assert_approx_eq!(res.value, r);
        }
    }

    impl Lens<DummyAsset> for DummyLens {
        fn lerp(&mut self, mut target: Mut<DummyAsset>, ratio: f32) {
            target.value = self.start.lerp(self.end, ratio);
        }
    }

    #[test]
    fn dummy_lens_asset() {
        let mut assets = Assets::<DummyAsset>::default();
        let handle = assets.add(DummyAsset::default());

        let mut l = DummyLens { start: 0., end: 1. };
        for r in [0_f32, 0.01, 0.3, 0.5, 0.9, 0.999, 1.] {
            {
                let mut added = Tick::new(0);
                let mut last_changed = Tick::new(0);
                let mut caller = MaybeLocation::caller();
                let asset = assets.get_mut(handle.id()).unwrap();
                let target = Mut::new(
                    asset,
                    &mut added,
                    &mut last_changed,
                    Tick::new(0),
                    Tick::new(0),
                    caller.as_mut(),
                );
                l.lerp(target, r);
            }
            assert_approx_eq!(assets.get(handle.id()).unwrap().value, r);
        }
    }

    #[test]
    fn repeat_count() {
        let cycle_duration = Duration::from_millis(100);

        let repeat = RepeatCount::default();
        assert_eq!(repeat, RepeatCount::Finite(1));
        assert_eq!(
            repeat.total_duration(cycle_duration),
            TotalDuration::Finite(cycle_duration)
        );

        let repeat: RepeatCount = 3u32.into();
        assert_eq!(repeat, RepeatCount::Finite(3));
        assert_eq!(
            repeat.total_duration(cycle_duration),
            TotalDuration::Finite(cycle_duration * 3)
        );

        let duration = Duration::from_secs(5);
        let repeat: RepeatCount = duration.into();
        assert_eq!(repeat, RepeatCount::For(duration));
        assert_eq!(
            repeat.total_duration(cycle_duration),
            TotalDuration::Finite(duration)
        );

        let repeat = RepeatCount::Infinite;
        assert_eq!(
            repeat.total_duration(cycle_duration),
            TotalDuration::Infinite
        );
    }

    #[test]
    fn repeat_strategy() {
        let strategy = RepeatStrategy::default();
        assert_eq!(strategy, RepeatStrategy::Repeat);
    }

    #[test]
    fn playback_direction() {
        let tweening_direction = PlaybackDirection::default();
        assert_eq!(tweening_direction, PlaybackDirection::Forward);
    }

    #[test]
    fn playback_state() {
        let mut state = PlaybackState::default();
        assert_eq!(state, PlaybackState::Playing);
        state = !state;
        assert_eq!(state, PlaybackState::Paused);
        state = !state;
        assert_eq!(state, PlaybackState::Playing);
    }

    #[test]
    fn ease_method() {
        let ease = EaseMethod::default();
        assert!(matches!(
            ease,
            EaseMethod::EaseFunction(EaseFunction::Linear)
        ));

        let ease = EaseMethod::EaseFunction(EaseFunction::QuadraticIn);
        assert_eq!(0., ease.sample(0.));
        assert_eq!(0.25, ease.sample(0.5));
        assert_eq!(1., ease.sample(1.));

        let ease = EaseMethod::EaseFunction(EaseFunction::Linear);
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

    // TweenAnim::playback_state is entirely user-controlled; stepping animations
    // won't change it.
    #[test]
    fn animation_playback_state() {
        for state in [PlaybackState::Playing, PlaybackState::Paused] {
            let tween = Tween::new::<DummyComponent, DummyLens>(
                EaseFunction::QuadraticInOut,
                Duration::from_secs(1),
                DummyLens { start: 0., end: 1. },
            );
            let mut env = TestEnv::<DummyComponent>::new(tween);
            let mut anim = env.anim_mut().unwrap();
            anim.playback_state = state;
            anim.destroy_on_completion = false;

            // Tick once
            let dt = Duration::from_millis(100);
            env.step_all(dt);
            assert_eq!(env.anim().unwrap().tween_state(), TweenState::Active);
            assert_eq!(env.anim().unwrap().playback_state, state);

            // Check elapsed
            let elapsed = match state {
                PlaybackState::Playing => dt,
                PlaybackState::Paused => Duration::ZERO,
            };
            assert_eq!(env.anim().unwrap().tweenable.elapsed(), elapsed);

            // Force playback, otherwise we can't complete
            env.anim_mut().unwrap().playback_state = PlaybackState::Playing;

            // Even after completion, the playback state is untouched
            env.step_all(Duration::from_secs(10) - elapsed);
            assert_eq!(env.anim().unwrap().tween_state(), TweenState::Completed);
            assert_eq!(env.anim().unwrap().playback_state, PlaybackState::Playing);
        }
    }

    #[test]
    fn animation_events() {
        let tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        )
        .with_repeat_count(2)
        .with_cycle_completed_event(true);
        let mut env = TestEnv::<DummyComponent>::new(tween);

        // Tick until one cycle is completed, but not the entire animation
        let dt = Duration::from_millis(1200);
        env.step_all(dt);
        assert_eq!(env.anim().unwrap().tween_state(), TweenState::Active);

        // Check events
        assert_eq!(env.event_count::<CycleCompletedEvent>(), 1);
        assert_eq!(env.event_count::<AnimCompletedEvent>(), 0);

        // Tick until completion
        let dt = Duration::from_millis(1000);
        env.step_all(dt);
        assert!(env.anim().is_none());

        // Check events (note that we didn't clear previous events, so that's a
        // cumulative count).
        assert_eq!(env.event_count::<CycleCompletedEvent>(), 1);
        assert_eq!(env.event_count::<AnimCompletedEvent>(), 1);
    }

    #[derive(Debug, Resource)]
    struct Count<E: Event, T = ()> {
        pub count: i32,
        pub phantom: PhantomData<E>,
        pub phantom2: PhantomData<T>,
    }

    impl<E: Event, T> Default for Count<E, T> {
        fn default() -> Self {
            Self {
                count: 0,
                phantom: PhantomData,
                phantom2: PhantomData,
            }
        }
    }

    struct GlobalMarker;

    #[test]
    fn animation_observe() {
        let tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        )
        .with_repeat_count(2)
        .with_cycle_completed_event(true);
        let mut env = TestEnv::<DummyComponent>::new(tween);

        env.world.init_resource::<Count<CycleCompletedEvent>>();
        assert_eq!(env.world.resource::<Count<CycleCompletedEvent>>().count, 0);
        env.world
            .init_resource::<Count<CycleCompletedEvent, GlobalMarker>>();
        assert_eq!(
            env.world
                .resource::<Count<CycleCompletedEvent, GlobalMarker>>()
                .count,
            0
        );

        fn observe_global(
            _trigger: On<CycleCompletedEvent>,
            mut count: ResMut<Count<CycleCompletedEvent, GlobalMarker>>,
        ) {
            count.count += 1;
        }
        env.world.add_observer(observe_global);

        fn observe_entity(
            _trigger: On<CycleCompletedEvent>,
            mut count: ResMut<Count<CycleCompletedEvent>>,
        ) {
            count.count += 1;
        }
        env.world.entity_mut(env.entity).observe(observe_entity);

        // Tick until one cycle is completed, but not the entire animation
        let dt = Duration::from_millis(1200);
        env.step_all(dt);
        assert_eq!(env.anim().unwrap().tween_state(), TweenState::Active);

        // Check observer system ran
        assert_eq!(env.world.resource::<Count<CycleCompletedEvent>>().count, 1);
        assert_eq!(
            env.world
                .resource::<Count<CycleCompletedEvent, GlobalMarker>>()
                .count,
            1
        );

        // Tick until completion
        let dt = Duration::from_millis(1000);
        env.step_all(dt);
        assert!(env.anim().is_none());

        // Check observer system ran (note that we didn't clear previous events, so
        // that's a cumulative count).
        assert_eq!(env.world.resource::<Count<CycleCompletedEvent>>().count, 2);
        assert_eq!(
            env.world
                .resource::<Count<CycleCompletedEvent, GlobalMarker>>()
                .count,
            2
        );
    }

    // #[test]
    // fn animator_controls() {
    //     let tween = Tween::<DummyComponent>::new(
    //         EaseFunction::QuadraticInOut,
    //         Duration::from_secs(1),
    //         DummyLens { start: 0., end: 1. },
    //     );
    //     let mut animator = Animator::new(tween);
    //     assert_eq!(animator.state, AnimatorState::Playing);
    //     assert_approx_eq!(animator.tweenable().progress(), 0.);

    //     animator.stop();
    //     assert_eq!(animator.state, AnimatorState::Paused);
    //     assert_approx_eq!(animator.tweenable().progress(), 0.);

    //     animator.tweenable_mut().set_progress(0.5);
    //     assert_eq!(animator.state, AnimatorState::Paused);
    //     assert_approx_eq!(animator.tweenable().progress(), 0.5);

    //     animator.tweenable_mut().rewind();
    //     assert_eq!(animator.state, AnimatorState::Paused);
    //     assert_approx_eq!(animator.tweenable().progress(), 0.);

    //     animator.tweenable_mut().set_progress(0.5);
    //     animator.state = AnimatorState::Playing;
    //     assert_eq!(animator.state, AnimatorState::Playing);
    //     assert_approx_eq!(animator.tweenable().progress(), 0.5);

    //     animator.tweenable_mut().rewind();
    //     assert_eq!(animator.state, AnimatorState::Playing);
    //     assert_approx_eq!(animator.tweenable().progress(), 0.);

    //     animator.stop();
    //     assert_eq!(animator.state, AnimatorState::Paused);
    //     assert_approx_eq!(animator.tweenable().progress(), 0.);
    // }

    #[test]
    fn animation_speed() {
        let tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );

        let mut env = TestEnv::<DummyComponent>::new(tween);

        assert_approx_eq!(env.anim().unwrap().speed, 1.); // default speed

        env.anim_mut().unwrap().speed = 2.4;
        assert_approx_eq!(env.anim().unwrap().speed, 2.4);

        env.step_all(Duration::from_millis(100));
        // Here we have enough precision for exact equality, but that may not always be
        // the case for larger durations or speed values.
        assert_eq!(
            env.anim().unwrap().tweenable.elapsed(),
            Duration::from_millis(240)
        );

        env.anim_mut().unwrap().speed = -1.;
        env.step_all(Duration::from_millis(100));
        // Safety: invalid negative speed clamped to 0.
        assert_eq!(env.anim().unwrap().speed, 0.);
        // At zero speed, step is a no-op so elapse() didn't change
        assert_eq!(
            env.anim().unwrap().tweenable.elapsed(),
            Duration::from_millis(240)
        );
    }

    #[test]
    fn animator_set_tweenable() {
        let tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let tween2 = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::SmoothStep,
            Duration::from_secs(2),
            DummyLens { start: 2., end: 3. },
        );

        let mut env = TestEnv::<DummyComponent>::new(tween);
        env.anim_mut().unwrap().destroy_on_completion = false;

        let dt = Duration::from_millis(1500);

        env.step_all(dt);
        assert_eq!(env.component().value, 1.);
        assert_eq!(env.anim().unwrap().tween_state(), TweenState::Completed);

        // Swap tweens
        let old_tweenable = env.anim_mut().unwrap().set_tweenable(tween2).unwrap();

        assert_eq!(env.anim().unwrap().tween_state(), TweenState::Active);
        // The elapsed is stored inside the tweenable
        assert_eq!(old_tweenable.elapsed(), Duration::from_secs(1)); // capped at total_duration()
        assert_eq!(env.anim().unwrap().tweenable.elapsed(), Duration::ZERO);

        env.step_all(dt);
        assert!(env.component().value >= 2. && env.component().value <= 3.);
    }

    // Currently multi-target sequences are not implemented. This _could_ work with
    // implicit targets (so, multiple components on the same entity), but is a bit
    // complex to implement with the current code. So leave that out for now, and
    // test we assert if the user attempts it. The workaround is to create separate
    // animations for each comopnent/target. Anyway multi-target sequence can't work
    // with other target types, since they need an explicit TweenAnim, and we
    // can't have more than one per entity.
    #[test]
    #[should_panic(
        expected = "TODO: Cannot use tweenable animations with different targets inside the same Sequence. Create separate animations for each target."
    )]
    fn seq_multi_target() {
        let tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        )
        .then(Tween::new::<DummyComponent2, DummyLens2>(
            EaseFunction::SmoothStep,
            Duration::from_secs(1),
            DummyLens2 { start: -5, end: 5 },
        ));
        let mut env = TestEnv::<DummyComponent>::new(tween);
        let entity = env.entity;
        env.world
            .entity_mut(entity)
            .insert(DummyComponent2 { value: -42 });
        TweenAnim::step_one(&mut env.world, Duration::from_millis(1100), entity).unwrap();
    }

    // #[test]
    // fn animator_set_target() {
    //     let tween = Tween::new::<DummyComponent, DummyLens>(
    //         EaseFunction::QuadraticInOut,
    //         Duration::from_secs(1),
    //         DummyLens { start: 0., end: 1. },
    //     );
    //     let mut env = TestEnv::<DummyComponent>::new(tween);

    //     // Register our custom asset type
    //     env.world.init_resource::<Assets<DummyAsset>>();

    //     // Invalid ID
    //     {
    //         let entity = env.entity;
    //         let target =
    //
    // ComponentAnimTarget::new::<DummyComponent>(env.world.components(),
    // entity).unwrap();         let err = env
    //             .animator_mut()
    //             .set_target(Entity::PLACEHOLDER, target.into())
    //             .err()
    //             .unwrap();
    //         let TweeningError::InvalidTweenId(err_id) = err else {
    //             panic!();
    //         };
    //         assert_eq!(err_id, Entity::PLACEHOLDER);
    //     }

    //     // Spawn a second entity without any animation
    //     let entity1 = env.entity;
    //     let entity2 = env.world_mut().spawn(DummyComponent { value: 0.
    // }).id();     assert_ne!(entity1, entity2);
    //     assert_eq!(env.component().value, 0.);

    //     // Step the current target
    //     let dt = Duration::from_millis(100);
    //     env.step_all(dt);
    //     assert!(env.component().value > 0.);
    //     assert_eq!(
    //         env.world
    //             .entity(entity2)
    //             .get_components::<&DummyComponent>()
    //             .unwrap()
    //             .value,
    //         0.
    //     );

    //     // Now retarget
    //     let id = env.entity;
    //     let target2 =
    //         ComponentAnimTarget::new::<DummyComponent>(env.world.
    // components(), entity2).unwrap();     let target1 =
    // env.animator_mut().set_target(id, target2.into()).unwrap();
    //     assert!(target1.is_component());
    //     let comp1 = target1.as_component().unwrap();
    //     assert_eq!(comp1.entity, entity1);
    //     assert_eq!(
    //         comp1.component_id,
    //         env.world.component_id::<DummyComponent>().unwrap()
    //     );

    //     // Step the new target
    //     env.step_all(dt);
    //     assert!(env.component().value > 0.);
    //     assert!(
    //         env.world
    //             .entity(entity1)
    //             .get_components::<&DummyComponent>()
    //             .unwrap()
    //             .value
    //             > 0.
    //     );

    //     // Invalid target
    //     {
    //         let target3 =
    //             AssetAnimTarget::new(env.world.components(),
    // Handle::<DummyAsset>::default().id())                 .unwrap();
    //         let err3 = env.animator_mut().set_target(id, target3.into());
    //         assert!(err3.is_err());
    //         let err3 = err3.err().unwrap();
    //         let TweeningError::MismatchingTargetKind(oc, nc) = err3 else {
    //             panic!();
    //         };
    //         assert_eq!(oc, true);
    //         assert_eq!(nc, false);
    //     }
    // }

    #[test]
    fn anim_target_component() {
        let mut env = TestEnv::<Transform>::empty();
        let entity = env.world.spawn(Transform::default()).id();
        let tween = Tween::new::<Transform, TransformPositionLens>(
            EaseFunction::Linear,
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let target = AnimTarget::component::<Transform>(entity);
        let anim_entity = env
            .world
            .spawn((
                TweenAnim::new(tween)
                    .with_speed(2.)
                    .with_destroy_on_completed(true),
                target,
            ))
            .id();

        // Step
        assert!(
            TweenAnim::step_one(&mut env.world, Duration::from_millis(100), anim_entity).is_ok()
        );
        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.translation, Vec3::ONE * 0.2);

        // Complete
        assert_eq!(
            TweenAnim::step_many(&mut env.world, Duration::from_millis(400), &[anim_entity]),
            1
        );

        // Destroyed on completion
        assert!(env.world.entity(anim_entity).get::<TweenAnim>().is_none());
    }

    #[test]
    fn anim_target_resource() {
        let mut env = TestEnv::<Transform>::empty();
        env.world.init_resource::<DummyResource>();
        let tween = Tween::new::<DummyResource, DummyLens>(
            EaseFunction::Linear,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let target = AnimTarget::resource::<DummyResource>();
        let anim_entity = env
            .world
            .spawn((
                TweenAnim::new(tween)
                    .with_speed(2.)
                    .with_destroy_on_completed(true),
                target,
            ))
            .id();

        // Step
        assert!(
            TweenAnim::step_one(&mut env.world, Duration::from_millis(100), anim_entity).is_ok()
        );
        let res = env.world.resource::<DummyResource>();
        assert_eq!(res.value, 0.2);

        // Complete
        assert_eq!(
            TweenAnim::step_many(&mut env.world, Duration::from_millis(400), &[anim_entity]),
            1
        );

        // Destroyed on completion
        assert!(env.world.entity(anim_entity).get::<TweenAnim>().is_none());
    }

    #[test]
    fn anim_target_asset() {
        let mut env = TestEnv::<Transform>::empty();
        let mut assets = Assets::<DummyAsset>::default();
        let handle = assets.add(DummyAsset::default());
        env.world.insert_resource(assets);
        let tween = Tween::new::<DummyAsset, DummyLens>(
            EaseFunction::Linear,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let target = AnimTarget::asset::<DummyAsset>(&handle);
        let anim_entity = env
            .world
            .spawn((
                TweenAnim::new(tween)
                    .with_speed(2.)
                    .with_destroy_on_completed(true),
                target,
            ))
            .id();

        // Step
        assert!(
            TweenAnim::step_one(&mut env.world, Duration::from_millis(100), anim_entity).is_ok()
        );
        let assets = env.world.resource::<Assets<DummyAsset>>();
        let asset = assets.get(&handle).unwrap();
        assert_eq!(asset.value, 0.2);

        // Complete
        assert_eq!(
            TweenAnim::step_many(&mut env.world, Duration::from_millis(400), &[anim_entity]),
            1
        );

        // Destroyed on completion
        assert!(env.world.entity(anim_entity).get::<TweenAnim>().is_none());
    }

    #[test]
    fn animated_entity_commands_common() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .move_to(Vec3::ONE, Duration::from_secs(1), EaseFunction::Linear)
            .with_repeat_count(4)
            .with_repeat_strategy(RepeatStrategy::MirroredRepeat)
            .id();
        let entity2 = env
            .world
            .commands()
            .spawn(Transform::default())
            .move_to(Vec3::ONE, Duration::from_secs(1), EaseFunction::Linear)
            .with_repeat(4, RepeatStrategy::MirroredRepeat)
            .into_inner()
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(3300));

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.translation, Vec3::ONE * 0.7);
        let tr = env.world.entity(entity2).get::<Transform>().unwrap();
        assert_eq!(tr.translation, Vec3::ONE * 0.7);
    }

    #[test]
    fn animated_entity_commands_move_to() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .move_to(Vec3::ONE, Duration::from_secs(1), EaseFunction::Linear)
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(300));

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.translation, Vec3::ONE * 0.3);
    }

    #[test]
    fn animated_entity_commands_move_from() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .move_from(Vec3::ONE, Duration::from_secs(1), EaseFunction::Linear)
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(300));

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.translation, Vec3::ONE * 0.7);
    }

    #[test]
    fn animated_entity_commands_scale_to() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .scale_to(Vec3::ONE * 2., Duration::from_secs(1), EaseFunction::Linear)
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(300));

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.scale, Vec3::ONE * 1.3);
    }

    #[test]
    fn animated_entity_commands_scale_from() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .scale_from(Vec3::ONE * 2., Duration::from_secs(1), EaseFunction::Linear)
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(300));

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.scale, Vec3::ONE * 1.7);
    }

    #[test]
    fn animated_entity_commands_rotate_x() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .rotate_x(Duration::from_secs(1))
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(1300));

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.rotation, Quat::from_rotation_x(TAU * 0.3));
    }

    #[test]
    fn animated_entity_commands_rotate_y() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .rotate_y(Duration::from_secs(1))
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(1300));

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.rotation, Quat::from_rotation_y(TAU * 0.3));
    }

    #[test]
    fn animated_entity_commands_rotate_z() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .rotate_z(Duration::from_secs(1))
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(1300));

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.rotation, Quat::from_rotation_z(TAU * 0.3));
    }

    #[test]
    fn animated_entity_commands_rotate_x_by() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .rotate_x_by(FRAC_PI_2, Duration::from_secs(1), EaseFunction::Linear)
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(1300)); // 130%

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.rotation, Quat::from_rotation_x(FRAC_PI_2)); // 100%
    }

    #[test]
    fn animated_entity_commands_rotate_y_by() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .rotate_y_by(FRAC_PI_2, Duration::from_secs(1), EaseFunction::Linear)
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(1300)); // 130%

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.rotation, Quat::from_rotation_y(FRAC_PI_2)); // 100%
    }

    #[test]
    fn animated_entity_commands_rotate_z_by() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        let entity = env
            .world
            .commands()
            .spawn(Transform::default())
            .rotate_z_by(FRAC_PI_2, Duration::from_secs(1), EaseFunction::Linear)
            .id();
        env.world.flush();

        env.step_all(Duration::from_millis(1300)); // 130%

        let tr = env.world.entity(entity).get::<Transform>().unwrap();
        assert_eq!(tr.rotation, Quat::from_rotation_z(FRAC_PI_2)); // 100%
    }

    #[test]
    fn resolver_resource() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        // Register the resource and create a TweenAnim for it
        env.world.init_resource::<DummyResource>();
        let tween = Tween::new::<DummyResource, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let entity = env.world.commands().spawn(TweenAnim::new(tween)).id();

        // Ensure all commands are applied before starting the test
        env.world.flush();

        let delta_time = Duration::from_millis(200);
        let resource_id = env.world.resource_id::<DummyResource>().unwrap();

        // Resource resolver not registered; fails
        env.world
            .resource_scope(|world, resolver: Mut<TweenResolver>| {
                world.resource_scope(
                    |world, mut cycle_events: Mut<Messages<CycleCompletedEvent>>| {
                        world.resource_scope(
                            |world, mut anim_events: Mut<Messages<AnimCompletedEvent>>| {
                                assert!(resolver
                                    .resolve_resource(
                                        world,
                                        &TypeId::of::<DummyResource>(),
                                        resource_id,
                                        entity,
                                        delta_time,
                                        cycle_events.reborrow(),
                                        anim_events.reborrow(),
                                    )
                                    .is_err());
                            },
                        );
                    },
                );
            });

        // Register the resource resolver
        env.world
            .resource_scope(|world, mut resolver: Mut<TweenResolver>| {
                resolver.register_resource_resolver_for::<DummyResource>(world.components());
            });

        // Resource resolver registered; succeeds
        env.world
            .resource_scope(|world, resolver: Mut<TweenResolver>| {
                world.resource_scope(
                    |world, mut cycle_events: Mut<Messages<CycleCompletedEvent>>| {
                        world.resource_scope(
                            |world, mut anim_events: Mut<Messages<AnimCompletedEvent>>| {
                                assert!(resolver
                                    .resolve_resource(
                                        world,
                                        &TypeId::of::<DummyResource>(),
                                        resource_id,
                                        entity,
                                        delta_time,
                                        cycle_events.reborrow(),
                                        anim_events.reborrow(),
                                    )
                                    .unwrap());
                            },
                        );
                    },
                );
            });
    }

    #[test]
    fn resolver_asset() {
        let dummy_tween = Tween::new::<DummyComponent, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let mut env = TestEnv::<DummyComponent>::new(dummy_tween);

        // Register the asset and create a TweenAnim for it
        let mut assets = Assets::<DummyAsset>::default();
        let handle = assets.add(DummyAsset::default());
        let untyped_asset_id = handle.id().untyped();
        env.world.insert_resource(assets);
        let tween = Tween::new::<DummyAsset, DummyLens>(
            EaseFunction::QuadraticInOut,
            Duration::from_secs(1),
            DummyLens { start: 0., end: 1. },
        );
        let entity = env.world.commands().spawn(TweenAnim::new(tween)).id();

        // Ensure all commands are applied before starting the test
        env.world.flush();

        let delta_time = Duration::from_millis(200);
        let resource_id = env.world.resource_id::<Assets<DummyAsset>>().unwrap();

        // Asset resolver not registered; fails
        env.world
            .resource_scope(|world, resolver: Mut<TweenResolver>| {
                world.resource_scope(
                    |world, mut cycle_events: Mut<Messages<CycleCompletedEvent>>| {
                        world.resource_scope(
                            |world, mut anim_events: Mut<Messages<AnimCompletedEvent>>| {
                                assert!(resolver
                                    .resolve_asset(
                                        world,
                                        &TypeId::of::<DummyAsset>(),
                                        resource_id,
                                        untyped_asset_id,
                                        entity,
                                        delta_time,
                                        cycle_events.reborrow(),
                                        anim_events.reborrow(),
                                    )
                                    .is_err());
                            },
                        );
                    },
                );
            });

        // Register the asset resolver
        env.world
            .resource_scope(|world, mut resolver: Mut<TweenResolver>| {
                resolver.register_asset_resolver_for::<DummyAsset>(world.components());
            });

        // Asset resolver registered; succeeds
        env.world
            .resource_scope(|world, resolver: Mut<TweenResolver>| {
                world.resource_scope(
                    |world, mut cycle_events: Mut<Messages<CycleCompletedEvent>>| {
                        world.resource_scope(
                            |world, mut anim_events: Mut<Messages<AnimCompletedEvent>>| {
                                assert!(resolver
                                    .resolve_asset(
                                        world,
                                        &TypeId::of::<DummyAsset>(),
                                        resource_id,
                                        untyped_asset_id,
                                        entity,
                                        delta_time,
                                        cycle_events.reborrow(),
                                        anim_events.reborrow(),
                                    )
                                    .unwrap());
                            },
                        );
                    },
                );
            });
    }
}
