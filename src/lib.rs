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
//! two values, to animate any field of any component and asset, including both
//! built-in Bevy ones and custom user-defined ones. Each field of a component
//! or asset can be animated via a collection of predefined easing functions, or
//! providing a custom animation curve. The library supports any number of
//! animations queued in parallel, even on the same component or asset type, and
//! allows runtime control over playback and animation speed.
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
//! commands
//!     // Spawn an entity to animate the position of.
//!     .spawn(Transform::default())
//!     // Create a tweenable animation targetting the current entity.
//!     // This spawns a TweenAnim component.
//!     .tween(tween);
//! # }
//! ```
//!
//! This example shows the general pattern to add animations for any component
//! or asset. Since moving the position of an object is a very common task,
//! 🍃 Bevy Tweening provides a shortcut for it. The above example can be
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
//! The [`tween()`] extension is convenient for fire-and-forget animations,
//! because it implicitly targets the current entity. However, sometimes you
//! want to remember the [`Entity`] owning the [`TweenAnim`], or insert a marker
//! component on it, so that you can retrieve it later to control the animation
//! at runtime. In that case, you can use instead the [`tween_component()`]
//! extension instead:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_tweening::{lens::*, *};
//! # use std::time::Duration;
//! # fn system(mut commands: Commands) {
//! // Marker component to identify the TweenAnim controller.
//! #[derive(Component)]
//! struct AnimController;
//!
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
//! // The Entity with the Transform being animated.
//! // That entity contains a single Transform component.
//! let target_entity = commands.spawn(Transform::default()).id();
//!
//! // The Entity with the TweenAnim controlling the animation.
//! // That entity contains the (AnimController, TweenAnim) components.
//! let anim_entity = commands
//!     .spawn(AnimController)
//!     .tween_component(target_entity, tween)
//!     .id();
//! # }
//! ```
//!
//! See the [`EntityWorldMutTweeningExtensions`] extension trait for the various
//! helpers provided for common animations like [`move_to()`].
//!
//! # Ready to animate
//!
//! Unlike previous versions of 🍃 Bevy Tweening, **you don't need any
//! particular system setup** aside from adding the [`TweeningPlugin`] to your
//! [`App`]. In particular, per-component-type and per-asset-type systems are
//! gone. Instead, the plugin adds a _single_ system executing during the
//! [`Update`] schedule. Each [`TweenAnim`] acts as a controller for one
//! animation, and mutates its target which can be any component or asset, even
//! a custom one.
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
//! to do in older versions), simply enqueue each animation independently.
//! This require careful selection of timings if you want to synchronize
//! animations.
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
//! infinite animation in a sequence, and append more tweenables after it, those
//! tweenables will never play because playback will be stuck forever repeating
//! the first animation. You're responsible for creating sequences that make
//! sense. In general, only use infinite tweenable animations alone or as the
//! last element of a sequence (for example, move to position and then rotate
//! forever on self).
//!
//! # `TweenAnim`
//!
//! Bevy components and assets are animated with the [`TweenAnim`] component.
//! This component acts as a controller for the animation. It determines the
//! target component or asset to animate via an [`AnimTarget`], and accesses the
//! field(s) of that target using a [`Lens`].
//!
//! - Components are animated via the [`ComponentAnimTarget`], which identifies
//!   a component instance on an entity via the [`Entity`] itself and the
//!   [`ComponentId`] of the registered component type.
//! - Assets are animated in a similar way to component, via the
//!   [`AssetAnimTarget`] which identifies an asset via the type of its
//!   [`Assets`] collection (and so indirectly the type of asset itself) and the
//!   [`AssetId`] referencing that asset inside the collection.
//!
//! Because assets are typically shared, and the animation applies to the asset
//! itself, all users of the asset see the animation. For example, animating the
//! color of a [`ColorMaterial`] will change the color of all the
//! 2D meshes using that material. If you want to animate the color of a single
//! mesh, you need to duplicate the asset and assign a unique copy to that mesh,
//! then animate that copy alone.
//!
//! Although you generally should prefer using the various extensions on
//! commands, like the [`tween()`] function on entity commands, under the hood
//! the manual process of queuing a new animation involves spawning a
//! [`TweenAnim`] component.
//!
//! ```no_run
//! # use bevy::{prelude::*, ecs::component::Components};
//! # use bevy_tweening::*;
//! # fn make_tween() -> Tween { unimplemented!() }
//! fn my_system(mut commands: Commands, components: &Components) -> Result<(), TweeningError> {
//! # let entity = Entity::PLACEHOLDER;
//!     let entity = commands.spawn(Transform::default()).id();
//!     let target = ComponentAnimTarget::new::<Transform>(components, entity)?;
//!     // Place the TweenAnim on the same entity as the target component:
//!     let tween: Tween = make_tween();
//!     commands
//!         .entity(entity)
//!         .insert(TweenAnim::new(target, tween));
//!     // ---OR---
//!     // Place it on its own entity
//!     let tween: Tween = make_tween();
//!     commands.spawn(TweenAnim::new(target, tween));
//!     Ok(())
//! }
//! ```
//!
//! For assets, you also must ensure a resolver is registered for that asset
//! type, through [`TweenResolver::register_scoped_for()`]. Note that the
//! [`.tween_asset()`] extension takes care of this for you.
//!
//! ```no_run
//! # use bevy::{prelude::*, ecs::component::Components};
//! # use bevy_tweening::*;
//! # fn make_tween() -> Tween { unimplemented!() }
//! fn my_system(mut commands: Commands, components: &Components) -> Result<(), TweeningError> {
//! # let entity = Entity::PLACEHOLDER;
//!     let entity = commands.spawn(Transform::default()).id();
//!     let target = ComponentAnimTarget::new::<Transform>(components, entity)?;
//!     // Place the TweenAnim on the same entity as the target component:
//!     let tween: Tween = make_tween();
//!     commands
//!         .entity(entity)
//!         .insert(TweenAnim::new(target, tween));
//!     // ---OR---
//!     // Place it on its own entity
//!     let tween: Tween = make_tween();
//!     commands.spawn(TweenAnim::new(target, tween));
//!     Ok(())
//! }
//! ```
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
//! The [`AnimTarget`] references the target (component or asset) being
//! animated. However, only a part of that component or asset is generally
//! animated. To that end, the [`TweenAnim`] accesses the field(s) to animate
//! via a _lens_, a type that implements the [`Lens`] trait and allows mapping a
//! target component or asset to the actual value(s) animated.
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
//! of any component or asset, whether shipped with Bevy or defined by the user.
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
//! [`Transform::translation`]: https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.Transform.html#structfield.translation
//! [`Entity`]: https://docs.rs/bevy/0.16.0/bevy/ecs/entity/struct.Entity.html
//! [`ColorMaterial`]: https://docs.rs/bevy/0.16.0/bevy/sprite/struct.ColorMaterial.html
//! [`Transform`]: https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.Transform.html
//! [`TransformPositionLens`]: crate::lens::TransformPositionLens
//! [`tween()`]: crate::EntityWorldMutTweeningExtensions::tween
//! [`tween_component()`]: crate::EntityWorldMutTweeningExtensions::tween_component
//! [`tween_asset()`]: crate::EntityWorldMutTweeningExtensions::tween_asset
//! [`move_to()`]: crate::EntityWorldMutTweeningExtensions::move_to

use std::{any::TypeId, time::Duration};

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
pub use plugin::{AnimationSystem, TweeningPlugin};
use thiserror::Error;
pub use tweenable::{
    BoxedTweenable, CycleCompletedEvent, Delay, IntoBoxedTweenable, Sequence, TotalDuration, Tween,
    TweenState, Tweenable,
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

/// Extensions to queue tween-based animations.
///
/// This trait provide extension functions to [`EntityWorldMut`] and
/// [`EntityCommands`], allowing convenient syntaxes like inserting a new
/// component and immediately attaching a tweenable animation to it in a single
/// call.
///
/// ```
/// # use bevy::{prelude::*, ecs::world::CommandQueue};
/// # use bevy_tweening::{*, lens::TransformPositionLens};
/// # use std::time::Duration;
/// # let mut queue = CommandQueue::default();
/// # let mut world = World::default();
/// # let mut commands = Commands::new(&mut queue, &mut world);
/// let tween = Tween::new(
///     EaseFunction::QuadraticIn,
///     Duration::from_secs(1),
///     TransformPositionLens {
///         start: Vec3::ZERO,
///         end: Vec3::new(3.5, 0., 0.),
///     },
/// );
/// commands.spawn(Transform::default()).tween(tween);
/// ```
///
/// or even more concisely:
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
/// The extension offers 3 main functions, grouped into 2 core use cases:
/// - If you want to animate a component on the _current_ entity, use
///   [`tween()`]. This spawns a new separate empty entity with a new
///   [`TweenAnim`], which targets a component on the current entity. There's no
///   equivalent for assets, since an asset cannot be owned by an entity.
/// - If you want the _current entity to own the [`TweenAnim`]_:
///   - [`tween_component()`] animates a given target component.
///   - [`tween_asset()`] animates a given target asset.
///
/// This convenience comes at the price of reduced control and error checking.
/// Additional information like the [`Entity`] of a newly created [`TweenAnim`]
/// cannot be retrieved. And any error (_e.g._ trying to insert an animation
/// with a tweenable of a component type while the entity doesn't have that
/// component) cannot be forwarded back to the caller, so will produce a panic
/// instead. This is best used for cases where you know those conditions at
/// build time. To avoid a panic, prefer manually queuing a new tweenable
/// animation by spawning a [`TweenAnim`] component yourself.
///
/// [`tween()`]: Self::tween
/// [`tween_component()`]: Self::tween_component
/// [`tween_asset()`]: Self::tween_asset
pub trait EntityWorldMutTweeningExtensions<'a> {
    /// Queue the given [`Tweenable`] to animate the current entity.
    ///
    /// This inserts a new [`TweenAnim`] on a newly spawned entity, which
    /// animates the current entity. The proper component to animate on the
    /// current entity is based on the type of the lens stored inside the
    /// tweenable (see [`Tweenable::type_id()`]). That component must exist
    /// on the current entity.
    ///
    /// Because this function spawns a new entity for each animation, you can
    /// use it multiple times on the same source entity, including with the same
    /// component.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::{*, lens::TransformPositionLens};
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// let tween = Tween::new(
    ///     EaseFunction::QuadraticIn,
    ///     Duration::from_secs(1),
    ///     TransformPositionLens {
    ///         start: Vec3::ZERO,
    ///         end: Vec3::new(3.5, 0., 0.),
    ///     },
    /// );
    /// // Spawn Transform on a first entity, and TweenAnim on a second one,
    /// // animating the Transform one.
    /// commands.spawn(Transform::default()).tween(tween);
    /// ```
    fn tween(&mut self, tweenable: impl IntoBoxedTweenable) -> &mut Self;

    /// Queue the given [`Tweenable`] to animate a target component.
    ///
    /// This inserts a new [`TweenAnim`] on the current entity, which
    /// animates a given target component identified by the entity which owns
    /// it. The type of the component to animate on that target entity is based
    /// on the type of the lens stored inside the tweenable (see
    /// [`Tweenable::type_id()`]).
    ///
    /// Because this function spawns a new [`TweenAnim`] on the current entity,
    /// it cannot be called more than once.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::{*, lens::TransformPositionLens};
    /// # use std::time::Duration;
    /// # let mut queue = CommandQueue::default();
    /// # let mut world = World::default();
    /// # let mut commands = Commands::new(&mut queue, &mut world);
    /// #[derive(Component)]
    /// struct AnimMarker;
    ///
    /// let tween = Tween::new(
    ///     EaseFunction::QuadraticIn,
    ///     Duration::from_secs(1),
    ///     TransformPositionLens {
    ///         start: Vec3::ZERO,
    ///         end: Vec3::new(3.5, 0., 0.),
    ///     },
    /// );
    /// let target = commands.spawn(Transform::default()).id();
    /// // Spawn (AnimMarker, TweenAnim) on a new entity
    /// commands.spawn(AnimMarker).tween_component(target, tween);
    /// ```
    ///
    /// # Panics
    ///
    /// This call panics, or queue a command which later panics, if a
    /// [`TweenAnim`] component already exists on the current entity, whether
    /// from another prior call to [`tween_component()`] or [`tween_asset()`],
    /// or from manually inserting the component.
    ///
    /// [`tween_component()`]: Self::tween_component
    /// [`tween_asset()`]: Self::tween_asset
    fn tween_component(&mut self, entity: Entity, tweenable: impl IntoBoxedTweenable) -> &mut Self;

    /// Queue the given [`Tweenable`] to animate a target asset.
    ///
    /// This inserts a new [`TweenAnim`] on the current entity, which
    /// animates a given target asset. The asset type to animate is validated
    /// against the one on the type of the lens stored inside the
    /// tweenable (see [`Tweenable::type_id()`]).
    ///
    /// # Panics
    ///
    /// This call panics, or queue a command which later panics, if the type of
    /// the asset `A` is different from the type the `tweenable` is referencing
    /// through its [`Lens`].
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::world::CommandQueue};
    /// # use bevy_tweening::{*, lens::TransformPositionLens};
    /// # use std::time::Duration;
    /// # fn make_tween() -> Tween { unimplemented!() }
    /// #[derive(Asset, TypePath)]
    /// struct MyAsset;
    ///
    /// #[derive(Component)]
    /// struct MyAssetRef(pub Handle<MyAsset>);
    ///
    /// fn my_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    ///     let handle: Handle<MyAsset> = asset_server.load("my_asset");
    ///     let tween = make_tween();
    ///     // Spawn (MyAssetRef, TweenAnim) on a new entity
    ///     commands
    ///         .spawn(MyAssetRef(handle.clone()))
    ///         .tween_asset(&handle, tween);
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This call panics, or queue a command which later panics, if a
    /// [`TweenAnim`] component already exists on the current entity, whether
    /// from another prior call to [`tween_component()`] or [`tween_asset()`],
    /// or from manually inserting the component.
    ///
    /// [`tween_component()`]: Self::tween_component
    /// [`tween_asset()`]: Self::tween_asset
    fn tween_asset<A: Asset>(
        &mut self,
        id: impl Into<AssetId<A>>,
        tweenable: impl IntoBoxedTweenable,
    ) -> &mut Self;

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
    /// access to the [`Entity`] created. To retrieve the ID and control
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
        &mut self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut Self;

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
    /// access to the [`Entity`] created. To retrieve the ID and control
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
        &mut self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut Self;

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
    /// access to the [`Entity`] created. To retrieve the ID and control
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
        &mut self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut Self;

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
    /// access to the [`Entity`] created. To retrieve the ID and control
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
        &mut self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut Self;
}

impl<'a> EntityWorldMutTweeningExtensions<'a> for EntityCommands<'a> {
    #[inline]
    fn tween(&mut self, tweenable: impl IntoBoxedTweenable) -> &mut EntityCommands<'a> {
        let tweenable = tweenable.into_boxed();
        self.queue(move |mut entity: EntityWorldMut| {
            entity.tween(tweenable);
        })
    }

    #[inline]
    fn tween_component(
        &mut self,
        entity: Entity,
        tweenable: impl IntoBoxedTweenable,
    ) -> &mut EntityCommands<'a> {
        let tweenable = tweenable.into_boxed();
        self.queue(move |mut this: EntityWorldMut| {
            this.tween_component(entity, tweenable);
        })
    }

    fn tween_asset<A: Asset>(
        &mut self,
        id: impl Into<AssetId<A>>,
        tweenable: impl IntoBoxedTweenable,
    ) -> &mut Self {
        let id = id.into();
        let tweenable = tweenable.into_boxed();
        self.queue(move |mut entity: EntityWorldMut| {
            entity.tween_asset(id, tweenable);
        })
    }

    #[inline]
    fn move_to(
        &mut self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut EntityCommands<'a> {
        let ease_method = ease_method.into();
        self.queue(move |mut entity: EntityWorldMut| {
            entity.move_to(end, duration, ease_method);
        })
    }

    #[inline]
    fn move_from(
        &mut self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut EntityCommands<'a> {
        let ease_method = ease_method.into();
        self.queue(move |mut entity: EntityWorldMut| {
            entity.move_from(start, duration, ease_method);
        })
    }

    #[inline]
    fn scale_to(
        &mut self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut EntityCommands<'a> {
        let ease_method = ease_method.into();
        self.queue(move |mut entity: EntityWorldMut| {
            entity.scale_to(end, duration, ease_method);
        })
    }

    #[inline]
    fn scale_from(
        &mut self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut EntityCommands<'a> {
        let ease_method = ease_method.into();
        self.queue(move |mut entity: EntityWorldMut| {
            entity.scale_from(start, duration, ease_method);
        })
    }
}

///
pub trait EntityWorldMutTargetExtensions<'a> {
    ///
    fn anim_target<C: Component<Mutability = Mutable>>(self) -> ComponentAnimTarget;
}

impl<'a> EntityWorldMutTargetExtensions<'a> for EntityWorldMut<'a> {
    #[inline]
    fn anim_target<C: Component<Mutability = Mutable>>(self) -> ComponentAnimTarget {
        ComponentAnimTarget::new::<C>(self.world().components(), self.id()).unwrap()
    }
}

impl<'a> EntityWorldMutTweeningExtensions<'a> for EntityWorldMut<'a> {
    #[inline]
    fn tween(&mut self, tweenable: impl IntoBoxedTweenable) -> &mut Self {
        let entity = self.id();
        self.tween_component(entity, tweenable)
    }

    fn tween_component(&mut self, entity: Entity, tweenable: impl IntoBoxedTweenable) -> &mut Self {
        let tweenable = tweenable.into_boxed();
        let type_id = tweenable.type_id().unwrap();
        let component_id = self.world().components().get_id(type_id).unwrap();
        let target = ComponentAnimTarget {
            component_id,
            entity,
        };
        self.insert(TweenAnim::new(target, tweenable));
        self
    }

    fn tween_asset<A: Asset>(
        &mut self,
        id: impl Into<AssetId<A>>,
        tweenable: impl IntoBoxedTweenable,
    ) -> &mut Self {
        let tweenable = tweenable.into_boxed();
        let asset_id = id.into().untyped();
        let type_id = tweenable.type_id().unwrap();
        assert_eq!(type_id, TypeId::of::<A>());

        self.world_scope(|world| {
            world.resource_scope(|world: &mut World, mut resolver: Mut<TweenResolver>| {
                resolver.register_scoped_for::<A>(world.components());
            });
        });

        let resource_id = self.world().resource_id::<Assets<A>>().unwrap();
        let target = AssetAnimTarget {
            resource_id,
            asset_id,
        };
        self.insert(TweenAnim::new(target, tweenable));
        self
    }

    #[inline]
    fn move_to(
        &mut self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut Self {
        let start = self.get::<Transform>().unwrap().translation;
        let lens = lens::TransformPositionLens { start, end };
        let tween = Tween::new(ease_method, duration, lens);
        self.tween(tween)
    }

    #[inline]
    fn move_from(
        &mut self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut Self {
        let end = self.get::<Transform>().unwrap().translation;
        let lens = lens::TransformPositionLens { start, end };
        let tween = Tween::new(ease_method, duration, lens);
        self.tween(tween)
    }

    #[inline]
    fn scale_to(
        &mut self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut Self {
        let start = self.get::<Transform>().unwrap().scale;
        let lens = lens::TransformScaleLens { start, end };
        let tween = Tween::new(ease_method, duration, lens);
        self.tween(tween)
    }

    #[inline]
    fn scale_from(
        &mut self,
        start: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut Self {
        let end = self.get::<Transform>().unwrap().scale;
        let lens = lens::TransformScaleLens { start, end };
        let tween = Tween::new(ease_method, duration, lens);
        self.tween(tween)
    }
}

/// Event raised when a [`TweenAnim`] completed.
#[derive(Copy, Clone, Event)]
pub struct AnimCompletedEvent {
    /// The entity owning the [`TweenAnim`] which completed.
    ///
    /// Note that commonly the [`TweenAnim`] is despawned on completion, so
    /// can't be queried anymore with this entity. You can prevent a completed
    /// animation from being automatically destroyed by
    /// setting [`TweenAnim::destroy_on_completion`] to `false`.
    pub anim_entity: Entity,
    /// The animation target.
    ///
    /// This is provided both as a convenience for [`TweenAnim`]s not destroyed
    /// on completion, and because for those animations which are destroyed
    /// on completion the information is not available anymore when this
    /// event is received.
    pub target: AnimTarget,
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
    /// The asset container for the given asset type is not registered.
    #[error("Asset container Assets<A> for asset type A = {0:?} is not registered in the World.")]
    AssetNotRegistered(TypeId),
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
    /// Expected [`Tweenable::type_id()`] to return a value, but it returned
    /// `None`.
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

/// Component animation target.
///
/// References a component used as the target of a tweenable animation. The
/// component is identified by the ID of the component type as registered in the
/// [`World`] where the animation is queued, and the [`Entity`] holding the
/// component instance of that type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentAnimTarget {
    /// Component ID of the registered component being animated.
    pub component_id: ComponentId,
    /// Entity holding the component instance being animated.
    pub entity: Entity,
}

impl ComponentAnimTarget {
    /// Create a new component target.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::any::TypeId;
    /// # use bevy::{prelude::*, ecs::component::Components};
    /// # use bevy_tweening::*;
    /// fn my_system(mut commands: Commands, components: &Components) {
    ///     let entity = commands.spawn(Transform::default()).id();
    ///     let target = ComponentAnimTarget::new::<Transform>(components, entity);
    ///     // [...]
    /// }
    /// ```
    pub fn new<C: Component<Mutability = Mutable>>(
        components: &Components,
        entity: Entity,
    ) -> Result<Self, TweeningError> {
        let component_id = components
            .component_id::<C>()
            .ok_or(TweeningError::ComponentNotRegistered(TypeId::of::<C>()))?;
        Ok(Self {
            component_id,
            entity,
        })
    }

    /// Create a new component target from a component type ID.
    ///
    /// This is useful when the component type is not known at compile time;
    /// otherwise you should prefer [`new()`].
    ///
    /// # Example
    ///
    /// ```
    /// # use std::any::TypeId;
    /// # use bevy::{prelude::*, ecs::component::Components};
    /// # use bevy_tweening::*;
    /// fn my_system(mut commands: Commands, components: &Components) {
    ///     let entity = commands.spawn(Transform::default()).id();
    ///     let type_id = TypeId::of::<Transform>();
    ///     let target = ComponentAnimTarget::new_untyped(components, type_id, entity);
    ///     // [...]
    /// }
    /// ```
    ///
    /// [`new()`]: Self::new
    pub fn new_untyped(
        components: &Components,
        type_id: TypeId,
        entity: Entity,
    ) -> Result<Self, TweeningError> {
        let component_id = components
            .get_id(type_id)
            .ok_or(TweeningError::ComponentNotRegistered(type_id))?;
        Ok(Self {
            component_id,
            entity,
        })
    }
}

/// Asset animation target.
///
/// References an asset used as the target of a tweenable animation. The asset
/// is identified by the ID of the [`Assets`] resource type registered in the
/// [`World`] where the animation is queued, and the unique asset ID identifying
/// the asset instance inside that [`Assets`] resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetAnimTarget {
    /// Resource ID of the registered [`Assets`] asset container.
    pub resource_id: ComponentId,
    /// Asset ID of the target asset being animated.
    pub asset_id: UntypedAssetId,
}

impl AssetAnimTarget {
    /// Create a new asset target.
    ///
    /// The asset type `A` must be such that [`Assets<A>`] is registered in the
    /// input [`Components`].
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy::{prelude::*, ecs::component::Components};
    /// # use bevy_tweening::*;
    /// fn my_system(components: &Components, asset_server: Res<AssetServer>) {
    ///     let handle: Handle<Image> = asset_server.load("image.png");
    ///     let target = AssetAnimTarget::new::<Image>(components, &handle);
    ///     // [...]
    /// }
    /// ```
    pub fn new<A: Asset>(
        components: &Components,
        asset_id: impl Into<AssetId<A>>,
    ) -> Result<Self, TweeningError> {
        let resource_id = components
            .resource_id::<Assets<A>>()
            .ok_or(TweeningError::AssetNotRegistered(TypeId::of::<A>()))?;
        Ok(Self {
            resource_id,
            asset_id: asset_id.into().untyped(),
        })
    }

    /// Create a new asset target from an `Assets<A>` type ID.
    ///
    /// The `assets_type_id` must reference an [`Assets<A>`] type registered in
    /// the input [`Components`]. This is useful when the component type is not
    /// known at compile time; otherwise you should prefer [`new()`].
    ///
    /// # Example
    ///
    /// ```
    /// # use std::any::TypeId;
    /// # use bevy::{prelude::*, ecs::component::Components};
    /// # use bevy_tweening::*;
    /// fn my_system(components: &Components, asset_server: Res<AssetServer>) {
    ///     let handle: Handle<Image> = asset_server.load("image.png");
    ///     let handle = handle.untyped();
    ///     let assets_type_id = TypeId::of::<Assets<Image>>();
    ///     let target = AssetAnimTarget::new_untyped(components, assets_type_id, &handle);
    ///     // [...]
    /// }
    /// ```
    ///
    /// [`new()`]: Self::new
    pub fn new_untyped(
        components: &Components,
        assets_type_id: TypeId,
        asset_id: impl Into<UntypedAssetId>,
    ) -> Result<Self, TweeningError> {
        let asset_id = asset_id.into();
        // Note: asset_id.type_id() is A, whereas assets_type_id is Assets<A>
        let resource_id = components
            .get_resource_id(assets_type_id)
            .ok_or(TweeningError::AssetNotRegistered(assets_type_id))?;
        Ok(Self {
            resource_id,
            asset_id,
        })
    }
}

/// Animation target.
///
/// References either a component or an asset used as the target of a tweenable
/// animation. See [`ComponentAnimTarget`] and [`AssetAnimTarget`] for details.
/// This is a lightweight reference (copyable) implicitly tied to a given
/// [`World`].
///
/// To create an animation target from a given component or asset, see the
/// [`WorldTargetExtensions`] extension trait for [`World`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnimTarget {
    /// Component animation target.
    Component(ComponentAnimTarget),
    /// Asset animation target.
    Asset(AssetAnimTarget),
}

impl AnimTarget {
    /// Check if the target is an [`AnimTarget::Component`].
    #[inline]
    pub fn is_component(&self) -> bool {
        matches!(*self, AnimTarget::Component(..))
    }

    /// Convert this target to a component target if possible.
    ///
    /// # Returns
    ///
    /// Returns self as a [`ComponentAnimTarget`] if the value matches that enum
    /// variant, or `None` if it doesn't.
    #[inline]
    pub fn as_component(&self) -> Option<&ComponentAnimTarget> {
        if let Self::Component(component) = self {
            Some(component)
        } else {
            None
        }
    }

    /// Convert this target to a component target if possible.
    ///
    /// # Returns
    ///
    /// Returns self as a [`ComponentAnimTarget`] if the value matches that enum
    /// variant, or `None` if it doesn't.
    #[inline]
    pub fn as_component_mut(&mut self) -> Option<&mut ComponentAnimTarget> {
        if let Self::Component(component) = self {
            Some(component)
        } else {
            None
        }
    }

    /// Check if the target is an [`AnimTarget::Asset`].
    #[inline]
    pub fn is_asset(&self) -> bool {
        matches!(*self, AnimTarget::Asset(..))
    }

    /// Convert this target to an asset target if possible.
    ///
    /// # Returns
    ///
    /// Returns self as an [`AssetAnimTarget`] if the value matches that enum
    /// variant, or `None` if it doesn't.
    #[inline]
    pub fn as_asset(&self) -> Option<&AssetAnimTarget> {
        if let Self::Asset(asset) = self {
            Some(asset)
        } else {
            None
        }
    }

    /// Convert this target to an asset target if possible.
    ///
    /// # Returns
    ///
    /// Returns self as an [`AssetAnimTarget`] if the value matches that enum
    /// variant, or `None` if it doesn't.
    #[inline]
    pub fn as_asset_mut(&mut self) -> Option<&mut AssetAnimTarget> {
        if let Self::Asset(asset) = self {
            Some(asset)
        } else {
            None
        }
    }
}

impl From<ComponentAnimTarget> for AnimTarget {
    fn from(value: ComponentAnimTarget) -> Self {
        AnimTarget::Component(value)
    }
}

impl From<AssetAnimTarget> for AnimTarget {
    fn from(value: AssetAnimTarget) -> Self {
        AnimTarget::Asset(value)
    }
}

/// Extension trait for [`World`].
///
/// This trait extends [`World`] with some helper functions.
pub trait WorldTargetExtensions {
    /// Get a [`ComponentAnimTarget`] for the given component type and entity
    /// pair.
    ///
    /// The target references the component instance held by the entity. The
    /// entity must exist in the [`World`], and the component type must be
    /// registered.
    ///
    /// # Returns
    ///
    /// Returns the animation target referencing the component instance, or
    /// `None` if either the entity doesn't exist or the component type is not
    /// registered.
    fn get_anim_component_target<C: Component<Mutability = Mutable>>(
        &self,
        entity: Entity,
    ) -> Option<ComponentAnimTarget>;

    /// Get an [`AssetAnimTarget`] for the given asset type and ID pair.
    ///
    /// The target references the asset instance with the given ID. The
    /// ID must be valid, that is reference an existing asset in the `Assets<A>`
    /// collection, and the asset type `A` must be registered.
    ///
    /// # Returns
    ///
    /// Returns the animation target referencing the asset instance, or
    /// `None` if either the ID doesn't reference an existing asset or the asset
    /// type is not registered.
    fn get_anim_asset_target<A: Asset>(&self, id: impl Into<AssetId<A>>)
        -> Option<AssetAnimTarget>;
}

impl WorldTargetExtensions for World {
    fn get_anim_component_target<C: Component<Mutability = Mutable>>(
        &self,
        entity: Entity,
    ) -> Option<ComponentAnimTarget> {
        let component_id = self.component_id::<C>()?;
        if !self.entities().contains(entity) {
            return None;
        }
        Some(ComponentAnimTarget {
            component_id,
            entity,
        })
    }

    fn get_anim_asset_target<A: Asset>(
        &self,
        id: impl Into<AssetId<A>>,
    ) -> Option<AssetAnimTarget> {
        let id = id.into();
        if !self.resource::<Assets<A>>().contains(id) {
            return None;
        }
        let resource_id = self.resource_id::<Assets<A>>()?;
        Some(AssetAnimTarget {
            resource_id,
            asset_id: id.untyped(),
        })
    }
}

/// Animation instance.
///
/// The [`TweenAnim`] represents a single animation instance for a single
/// target, component or asset. Each instance is independent, even if it mutates
/// the same target as another instance. Spawning this component adds an active
/// animation, and destroying it stops that animation. The component can also be
/// used to control the animation playback at runtime, like the playback speed.
///
/// You can create this component yourself via [`new()`] and spawn it as a
/// regular component, but doing so involves creating an [`AnimTarget`], which
/// is now always convenient. Instead, it's often easier to use one of the
/// extension functions provided by the [`EntityWorldMutTweeningExtensions`]
/// trait, like [`tween()`]. Those extensions automatically create an approriate
/// [`AnimTarget`] for you.
///
/// _If you're looking for the basic tweenable animation description, see
/// [`Tween`] instead._
///
/// [`new()`]: Self::new
/// [`tween()`]: EntityWorldMutTweeningExtensions::tween
#[derive(Component)]
pub struct TweenAnim {
    /// Target [`Entity`] containing the component to animate, or target asset.
    target: AnimTarget,
    /// Animation description.
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
    /// Panics if the tweenable is "typeless", that is [`Tweenable::type_id()`]
    /// returns `None`. Animations must target a concrete component or asset
    /// type. This means in particular that you can't use a single [`Delay`]
    /// alone. You can however use a [`Delay`] or other typeless tweenables
    /// as part of a [`Sequence`], provided there's at least one other typed
    /// tweenable in the sequence to make it typed too.
    #[inline]
    pub fn new(target: impl Into<AnimTarget>, tweenable: impl IntoBoxedTweenable) -> Self {
        let tweenable = tweenable.into_boxed();
        assert!(
            tweenable.type_id().is_some(),
            "The top-level Tweenable of a TweenAnim must be typed (type_id() returns Some)."
        );
        Self {
            target: target.into(),
            tweenable,
            playback_state: PlaybackState::Playing,
            speed: 1.,
            destroy_on_completion: true,
            tween_state: TweenState::Active,
        }
    }

    /// Create a new tweenable animation for a specific component target.
    ///
    /// This is a convenience helper which builds a [`ComponentAnimTarget`] then
    /// calls [`new()`] with it. It's often useful in cases where you want to
    /// manually insert the [`TweenAnim`] component to save its [`Entity`] so
    /// you can reference it later, typically to control playback. Otherwise,
    /// prefer using [`.tween()`] or similar to automatically create the
    /// component and insert it.
    ///
    /// Note that the target component type is automically derived from the
    /// input `tweenable`.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::any::TypeId;
    /// # use bevy::{prelude::*, ecs::component::Components};
    /// # use bevy_tweening::*;
    /// # fn make_tween() -> Tween { unimplemented!() }
    /// fn my_system(mut commands: Commands, components: &Components) -> Result<(), TweeningError> {
    ///     // Get the entity holding the target component to animate. It doesn't need
    ///     // to be spawned here, it only needs to exist.
    ///     let target_entity = commands.spawn(Transform::default()).id();
    ///     // Create the animation
    ///     let tween = make_tween();
    ///     let anim = TweenAnim::for_component(components, target_entity, tween)?;
    ///     // Spawn the TweenAnim component on a new, separate Entity, and record that
    ///     // Entity for later reference.
    ///     let anim_entity = commands.spawn(anim).id();
    ///     // [...]
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`new()`]: Self::new
    /// [`.tween()`]: crate::EntityWorldMutTweeningExtensions::tween
    pub fn for_component(
        components: &Components,
        target: Entity,
        tweenable: impl Tweenable + 'static,
    ) -> Result<Self, TweeningError> {
        let type_id = tweenable.type_id().ok_or(TweeningError::UntypedTweenable)?;
        let target = ComponentAnimTarget::new_untyped(components, type_id, target)?;
        Ok(Self::new(target, tweenable))
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
        let mut q_anims = world.query::<&TweenAnim>();
        let mut targets = Vec::with_capacity(anims.len());
        for entity in anims {
            if let Ok(target) = q_anims.get(world, *entity).map(|anim| anim.target) {
                targets.push((*entity, target));
            }
        }
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
    pub fn step_all(world: &mut World, delta_time: Duration) -> Result<(), TweeningError> {
        let mut q_anims = world.query::<(Entity, &TweenAnim)>();
        let targets = q_anims
            .iter(world)
            .map(|(entity, anim)| (entity, anim.target))
            .collect::<Vec<_>>();
        Self::step_impl(world, delta_time, &targets[..]);
        Ok(())
    }

    fn step_impl(world: &mut World, delta_time: Duration, anims: &[(Entity, AnimTarget)]) {
        let mut to_remove = Vec::with_capacity(anims.len());
        world.resource_scope(|world, asset_resolver: Mut<TweenResolver>| {
            world.resource_scope(
                |world, mut cycle_events: Mut<Events<CycleCompletedEvent>>| {
                    world.resource_scope(
                        |world, mut anim_events: Mut<Events<AnimCompletedEvent>>| {
                            let anim_comp_id = world.component_id::<TweenAnim>().unwrap();
                            for (anim_entity, anim_target) in anims {
                                let ret = match anim_target {
                                    AnimTarget::Component(comp_target) => {
                                        let (mut entities, commands) =
                                            world.entities_and_commands();
                                        if *anim_entity == comp_target.entity {
                                            // The TweenAnim animates another component on the same
                                            // entity
                                            let Ok([mut ent]) = entities.get_mut([*anim_entity])
                                            else {
                                                continue;
                                            };
                                            let Ok([anim, target]) = ent.get_mut_by_id([
                                                anim_comp_id,
                                                comp_target.component_id,
                                            ]) else {
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
                                                target,
                                                cycle_events.reborrow(),
                                                anim_events.reborrow(),
                                            )
                                        } else {
                                            // The TweenAnim animates a component on a different
                                            // entity
                                            let Ok([mut anim, mut target]) = entities
                                                .get_mut([*anim_entity, comp_target.entity])
                                            else {
                                                continue;
                                            };
                                            let Some(mut anim) = anim.get_mut::<TweenAnim>() else {
                                                continue;
                                            };
                                            let Ok(target) =
                                                target.get_mut_by_id(comp_target.component_id)
                                            else {
                                                continue;
                                            };
                                            anim.step_self(
                                                commands,
                                                *anim_entity,
                                                delta_time,
                                                target,
                                                cycle_events.reborrow(),
                                                anim_events.reborrow(),
                                            )
                                        }
                                    }
                                    AnimTarget::Asset(asset_target) => asset_resolver
                                        .resolve_scope(
                                            world,
                                            &asset_target,
                                            *anim_entity,
                                            delta_time,
                                            cycle_events.reborrow(),
                                            anim_events.reborrow(),
                                        ),
                                };

                                let retain = ret.map(|ret| ret.retain).unwrap_or(false);
                                if !retain {
                                    to_remove.push(*anim_entity);
                                }
                            }
                        },
                    );
                },
            );
        });

        for entity in to_remove.drain(..) {
            world.entity_mut(entity).remove::<TweenAnim>();
        }

        world.flush();
    }

    fn step_self(
        &mut self,
        mut commands: Commands,
        anim_entity: Entity,
        delta_time: Duration,
        mut mut_untyped: MutUntyped,
        mut cycle_events: Mut<Events<CycleCompletedEvent>>,
        mut anim_events: Mut<Events<AnimCompletedEvent>>,
    ) -> Result<StepResult, TweeningError> {
        let mut completed_events = Vec::with_capacity(8);
        let mut sent_commands = false;

        // Sanity checks on fields which can be freely modified by the user
        self.speed = self.speed.max(0.);

        // Retain completed animations only if requested
        if self.tween_state == TweenState::Completed {
            let ret = StepResult {
                retain: !self.destroy_on_completion,
                sent_commands: false,
            };
            return Ok(ret);
        }

        // Skip paused animations (but retain them)
        if self.playback_state == PlaybackState::Paused || self.speed <= 0. {
            let ret = StepResult {
                retain: true,
                sent_commands: false,
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
                id: anim_entity,
                target: self.target,
            });
        };
        let state = self.tweenable.step(
            anim_entity,
            delta_time,
            mut_untyped.reborrow(),
            &mut notify_completed,
        );
        self.tween_state = state;

        // Send tween completed events once we reclaimed mut access to world and can get
        // a Commands.
        if !completed_events.is_empty() {
            sent_commands = true;

            for event in completed_events.drain(..) {
                // Send buffered event
                cycle_events.send(event);

                // Trigger all entity-scoped observers
                commands.trigger_targets(event, anim_entity);
            }
        }

        // Raise animation completed event
        if state == TweenState::Completed {
            let event: AnimCompletedEvent = AnimCompletedEvent {
                anim_entity,
                target: self.target,
            };

            // Send buffered event
            anim_events.send(event);

            // Trigger all entity-scoped observers
            sent_commands = true;
            commands.trigger_targets(event, anim_entity);
        }

        let ret = StepResult {
            retain: state == TweenState::Active || !self.destroy_on_completion,
            sent_commands,
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

    /// Get the target this animation is mutating.
    ///
    /// To change the target, use [`TweenAnim::set_target()`].
    #[inline]
    pub fn target(&self) -> &AnimTarget {
        &self.target
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

    /// Retarget this animation.
    ///
    /// Attempt to change the target of this animation, even if it's already
    /// playing. This function performs a number of checks on the new target
    /// to ensure it's compatible with the previous target currently set. In
    /// particular, the new target needs to have:
    /// - the same kind (component or asset);
    /// - the same type.
    ///
    /// # Returns
    ///
    /// On success, returns the previous target which has been replaced.
    pub fn set_target(&mut self, target: AnimTarget) -> Result<AnimTarget, TweeningError> {
        match (self.target, target) {
            (AnimTarget::Component(old_component), AnimTarget::Component(new_component)) => {
                if old_component.component_id != new_component.component_id {
                    return Err(TweeningError::MismatchingComponentId(
                        old_component.component_id,
                        new_component.component_id,
                    ));
                }
            }
            (AnimTarget::Asset(old_asset), AnimTarget::Asset(new_asset)) => {
                if old_asset.resource_id != new_asset.resource_id {
                    return Err(TweeningError::MismatchingAssetResourceId(
                        old_asset.resource_id,
                        new_asset.resource_id,
                    ));
                }
            }
            _ => {
                return Err(TweeningError::MismatchingTargetKind(
                    self.target.is_component(),
                    target.is_component(),
                ))
            }
        }
        let old_target = self.target;
        self.target = target;
        Ok(old_target)
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

type Resolver = Box<
    dyn for<'w> Fn(MutUntyped<'w>, UntypedAssetId) -> Option<MutUntyped<'w>>
        + Send
        + Sync
        + 'static,
>;

type ScopedResolver = Box<
    dyn for<'w> Fn(
            &mut World,
            UntypedAssetId,
            Entity,
            Duration,
            Mut<Events<CycleCompletedEvent>>,
            Mut<Events<AnimCompletedEvent>>,
        ) -> Result<StepResult, TweeningError>
        + Send
        + Sync
        + 'static,
>;

///
#[derive(Default, Resource)]
pub struct TweenResolver {
    /// Asset resolver allowing to convert a pair of { untyped pointer to
    /// `Assets<A>`, untyped `AssetId` } into an untyped pointer to the asset A
    /// itself. This is necessary because there's no UntypedAssets interface in
    /// Bevy. The TypeId key must be the type of the `Assets<A>` type itself.
    /// The resolver is allowed to fail (return `None`), for example when the
    /// asset ID doesn't reference a valid asset.
    asset_resolver: HashMap<ComponentId, Resolver>,
    scoped_resolver: HashMap<ComponentId, ScopedResolver>,
}

impl TweenResolver {
    ///
    pub fn resolve_asset(
        &self,
        component_id: ComponentId,
    ) -> Option<
        &(dyn for<'w> Fn(MutUntyped<'w>, UntypedAssetId) -> Option<MutUntyped<'w>>
              + Send
              + Sync
              + 'static),
    > {
        self.asset_resolver
            .get(&component_id)
            .map(|resolver| resolver.as_ref())
    }

    ///
    pub fn register_scoped_for<A: Asset>(&mut self, components: &Components) {
        let resource_id = components.resource_id::<Assets<A>>().unwrap();
        let resolver = |world: &mut World,
                        asset_id: UntypedAssetId,
                        entity: Entity,
                        delta_time: Duration,
                        mut cycle_events: Mut<Events<CycleCompletedEvent>>,
                        mut anim_events: Mut<Events<AnimCompletedEvent>>|
         -> Result<StepResult, TweeningError> {
            let asset_id = asset_id.typed::<A>();
            // First, remove the Assets<A> from the world so we can access it mutably in
            // parallel of the TweenAnim
            world.resource_scope(|world, assets: Mut<Assets<A>>| {
                // Next, fetch the asset A itself from its Assets<A> based on its asset ID
                let Some(asset) = assets.filter_map_unchanged(|assets| assets.get_mut(asset_id))
                else {
                    return Err(TweeningError::InvalidAssetId(asset_id.into()));
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
                anim.step_self(
                    commands,
                    entity,
                    delta_time,
                    asset.into(),
                    cycle_events.reborrow(),
                    anim_events.reborrow(),
                )
            })
        };
        self.scoped_resolver
            .entry(resource_id)
            .or_insert(Box::new(resolver));
    }

    #[inline]
    pub(crate) fn resolve_scope(
        &self,
        world: &mut World,
        target: &AssetAnimTarget,
        entity: Entity,
        delta_time: Duration,
        cycle_events: Mut<Events<CycleCompletedEvent>>,
        anim_events: Mut<Events<AnimCompletedEvent>>,
    ) -> Result<StepResult, TweeningError> {
        let Some(resolver) = self.scoped_resolver.get(&target.resource_id) else {
            return Err(TweeningError::AssetResolverNotRegistered(
                target.resource_id,
            ));
        };
        resolver(
            world,
            target.asset_id,
            entity,
            delta_time,
            cycle_events,
            anim_events,
        )
    }
}

pub(crate) struct StepResult {
    pub retain: bool,
    #[allow(unused)]
    pub sent_commands: bool,
}

// /// Animator for tween-based animations.
// ///
// /// This resource stores all the active tweening animations for the entire
// /// application. It's essentially a hash map from a [`Entity`] uniquely
// /// identifying an active animation, to the [`TweenAnim`] runtime data of
// that /// animation. Use this resource to lookup animations by ID and modify
// their /// runtime data, for example their playback speed.
// ///
// /// # Active animations
// ///
// /// Animations queued into the [`TweenAnimator`] by default are pruned
// /// automatically on completion, and only active animations are retained. If
// you /// want the animator to retain completed animation instances, so you can
// /// continue to access them, you can set [`TweenAnim::destroy_on_completion`]
// to /// `false` to prevent this automated destruction. Note however that doing
// so /// will retain the animation instance forever, until set to `true` again.
// So /// you should avoid retaining all animations forever to prevent wasting
// memory /// and degrading performances. In general, the default pruning
// behavior is best /// suited for one-shot animations re-created each time
// they're needed. /// Conversely, disabling auto-destruction on completion is
// best suited to /// reusing the same animation instance again and again.
// ///
// /// # Lookup without `Entity`
// ///
// /// If you don't know the [`Entity`] of an animation, you can also lookup the
// /// set of animations for a given target, either component on an entity or an
// /// asset.
// ///
// /// ```
// /// # use bevy::{prelude::*, ecs::component::Components};
// /// # use bevy_tweening::*;
// /// fn my_system(components: &Components, animator: Res<TweenAnimator>) ->
// Result<()> { /// # let entity = Entity::PLACEHOLDER;
// ///     // Create an AnimTarget::Component() from an Entity and a component
// type ///     // let entity = ...
// ///     let target = ComponentAnimTarget::new::<Transform>(components,
// entity)?; ///     let target: AnimTarget = target.into();
// ///
// ///     // Lookup all active animations and filter by target
// ///     let animations: Vec<&TweenAnim> = animator
// ///         .iter()
// ///         .filter_map(|(_id, anim)| {
// ///             if *anim.target() == target {
// ///                 Some(anim)
// ///             } else {
// ///                 None
// ///             }
// ///         })
// ///         .collect::<_>();
// ///
// ///     Ok(())
// /// }
// /// ```

// impl TweenAnimator {
//     /// Add a new component animation to the animator queue.
//     ///
//     /// In general you don't need to call this directly. Instead, use the
//     /// extensions provided by [`EntityWorldMutTweeningExtensions`] to
// directly     /// create and queue tweenable animations on a given
// [`EntityCommands`],     /// like this:
//     ///
//     /// ```
//     /// # use bevy::prelude::*;
//     /// # use bevy_tweening::{lens::*, *};
//     /// # use std::time::Duration;
//     /// # let mut world = World::default();
//     /// # world.register_component::<Transform>();
//     /// # world.register_resource::<TweenAnimator>();
//     /// # world.init_resource::<TweenAnimator>();
//     /// # let entity = world.spawn_empty().id();
//     /// let tween = Tween::new(
//     ///     EaseFunction::QuadraticInOut,
//     ///     Duration::from_secs(1),
//     ///     TransformPositionLens {
//     ///         start: Vec3::ZERO,
//     ///         end: Vec3::new(3.5, 0., 0.),
//     ///     },
//     /// );
//     /// world.entity_mut(entity).tween(tween);
//     /// ```
//     ///
//     /// This function is still useful if you want to store the [`Entity`] of
//     /// the new animation, to later access it to dynamically modify the
// playback     /// (e.g. speed).
//     ///
//     /// ```
//     /// # use bevy::{prelude::*, ecs::component::Components};
//     /// # use bevy_tweening::{lens::*, *};
//     /// # use std::time::Duration;
//     /// # fn make_tween() -> Tween { unimplemented!() }
//     /// // Helper component to store a Entity
//     /// #[derive(Component)]
//     /// struct MyTweenId(pub Entity);
//     ///
//     /// // System which spawns the component and its animation
//     /// fn my_spawn_system(
//     ///     mut commands: Commands,
//     ///     components: &Components,
//     ///     mut animator: ResMut<TweenAnimator>,
//     /// ) -> Result<()> {
//     ///     # let tween = make_tween();
//     ///     let entity = commands.spawn(Transform::default()).id();
//     ///     // The component type is deducted from `tween` here
//     ///     let tween_id = animator.add_component(components, entity,
// tween)?;     ///     // Save the new Entity for later use
//     ///     commands.entity(entity).insert(MyTweenId(tween_id));
//     ///     Ok(())
//     /// }
//     ///
//     /// // System which modifies the animation playback
//     /// fn my_use_system(mut animator: ResMut<TweenAnimator>, query:
// Query<&MyTweenId>) -> Option<()> {     ///     let tween_id =
// query.single().ok()?.0;     ///     animator.get_mut(tween_id)?.speed = 1.2;
// // 120% playback speed     ///     Some(())
//     /// }
//     /// ```
//     ///
//     /// Note in the above that at the time when the animation is queued into
// the     /// animator, the component doesn't yet exist in the world, because
// the     /// spawning is deffered via [`Commands`]. This is not an error; the
//     /// [`TweenAnimator`] doesn't perform any check when this function is
//     /// called, on purpose to allow patterns like inserting the returned
//     /// [`Entity`] into a component or resource from within the same system
//     /// which spawned the target, without having to apply deferred commands
//     /// first. When the animation steps however, the target is validated, and
// if     /// invalid (either because the entity doesn't exist or it doesn't
// have the     /// target component) then the animation instance is destroyed.
//     #[inline]
//     pub fn add_component<T>(
//         &mut self,
//         components: &Components,
//         entity: Entity,
//         tweenable: T,
//     ) -> Result<Entity, TweeningError>
//     where
//         T: Tweenable + 'static,
//     {
//         let Some(type_id) = tweenable.type_id() else {
//             return Err(TweeningError::UntypedTweenable);
//         };
//         let target = ComponentAnimTarget::new_untyped(components, type_id,
// entity)?.into();         Ok(self.add_component_target(target, tweenable))
//     }

//     /// Add a new component animation via an existing
// [`ComponentAnimTarget`].     ///
//     /// See [`add_component()`] for details. This variant is useful when you
// can     /// build in advance a [`ComponentAnimTarget`], but at the same time
// don't     /// readily have access to the [`Components`] of the world.
//     ///
//     /// Note that there's no equivalent for assets, because asset animations
//     /// need to register some internal type-dependent resolver due to assets
//     /// being only accessible through the typed [`Assets<A>`] API.
//     ///
//     /// [`add_component()`]: Self::add_component
//     #[inline]
//     pub fn add_component_target<T>(&mut self, target: ComponentAnimTarget,
// tweenable: T) -> Entity     where
//         T: Tweenable + 'static,
//     {
//         self.anims
//             .insert(TweenAnim::new(target.into(), Box::new(tweenable)))
//     }

//     /// Add a new asset animation to the animator queue.
//     ///
//     /// ```
//     /// # use bevy::{prelude::*, ecs::component::Components};
//     /// # use bevy_tweening::{lens::*, *};
//     /// # use std::time::Duration;
//     /// # fn make_tween() -> Tween { unimplemented!() }
//     /// #[derive(Asset, TypePath)]
//     /// struct MyAsset;
//     ///
//     /// // Helper component to store a Entity
//     /// #[derive(Resource)]
//     /// struct MyTweenId(pub Entity);
//     ///
//     /// // System which spawns the asset animation
//     /// fn my_spawn_system(
//     ///     mut assets: ResMut<Assets<MyAsset>>,
//     ///     components: &Components,
//     ///     mut animator: ResMut<TweenAnimator>,
//     ///     mut my_tween_id: ResMut<MyTweenId>,
//     /// ) -> Result<()> {
//     ///     # let tween = make_tween();
//     ///     let handle = assets.add(MyAsset);
//     ///     // The asset type is deducted from `tween` here
//     ///     let tween_id = animator.add_asset(components, handle.id(),
// tween)?;     ///     // Save the new Entity for later use
//     ///     my_tween_id.0 = tween_id;
//     ///     Ok(())
//     /// }
//     ///
//     /// // System which modifies the animation playback
//     /// fn my_use_system(
//     ///     mut animator: ResMut<TweenAnimator>,
//     ///     my_tween_id: Res<MyTweenId>,
//     /// ) -> Option<()> {
//     ///     let tween_id = my_tween_id.0;
//     ///     animator.get_mut(tween_id)?.speed = 1.2; // 120% playback speed
//     ///     Some(())
//     /// }
//     /// ```
//     ///
//     /// Note that unlike [`add_component()`], this function depends on the
// asset     /// type. This is required to allow registering some internal
// resolver to     /// extract the typed animation from its typed `Assets<A>`,
// as this can't be     /// done via untyped references (unlike for components).
//     ///
//     /// [`get_asset_target()`]: WorldTargetExtensions::get_asset_target
//     /// [`add_component()`]: Self::add_component
//     #[inline]
//     pub fn add_asset<T, A: Asset>(
//         &mut self,
//         components: &Components,
//         asset_id: impl Into<AssetId<A>>,
//         tweenable: T,
//     ) -> Result<Entity, TweeningError>
//     where
//         T: Tweenable + 'static,
//     {
//         let Some(type_id) = tweenable.type_id() else {
//             return Err(TweeningError::UntypedTweenable);
//         };
//         if type_id != TypeId::of::<A>() {
//             return Err(TweeningError::InvalidAssetIdType {
//                 expected: TypeId::of::<A>(),
//                 actual: type_id,
//             });
//         }
//         let asset_id = asset_id.into();
//         let target = AssetAnimTarget::new(components, asset_id)?;
//         self.asset_resolver.insert(
//             target.resource_id,
//             Box::new(
//                 |assets: MutUntyped, asset_id: UntypedAssetId| -> Option<MutUntyped> {
//                     // SAFETY: The correct type is captured from the outer
// function                     #[allow(unsafe_code)]
//                     let assets = unsafe { assets.with_type::<Assets<A>>() };
//                     let asset_id = asset_id.try_typed::<A>().ok()?;
//                     assets
//                         .filter_map_unchanged(|assets|
// assets.get_mut(asset_id))                         .map(Into::into)
//                 },
//             ),
//         );
//         Ok(self
//             .anims
//             .insert(TweenAnim::new(target.into(), Box::new(tweenable))))
//     }

//     /// Check if the animation with the given ID is queued.
//     ///
//     /// If this returns `false` then any animation which might have existed
// with     /// this ID was destroyed, and this ID will forever be invalid and
//     /// unused.
//     ///
//     /// # Returns
//     ///
//     /// Returns `true` if the animation is queued, either because it's
// playing,     /// or because it's completed but
// [`TweenAnim::destroy_on_completion`] was     /// set to `false`. In that case
// the [`TweenAnim`] can be accessed by the     /// likes of [`get()`].
//     ///
//     /// [`get()`]: Self::get
//     #[inline]
//     pub fn contains(&self, id: Entity) -> bool {
//         self.anims.contains_key(id)
//     }

//     /// Get a queued tweenable animation from its ID.
//     ///
//     /// This fails and returns `None` if the animation has completed and was
//     /// removed from the animator's internal queue, or if the ID is invalid
//     /// (notably, `Entity::PLACEHOLDER`).
//     #[inline]
//     pub fn get(&self, id: Entity) -> Option<&TweenAnim> {
//         self.anims.get(id)
//     }

//     /// Get a queued tweenable animation from its ID.
//     ///
//     /// This fails and returns `None` if the animation has completed and was
//     /// removed from the animator's internal queue, or if the ID is invalid
//     /// (notably, `Entity::PLACEHOLDER`).
//     #[inline]
//     pub fn get_mut(&mut self, id: Entity) -> Option<&mut TweenAnim> {
//         self.anims.get_mut(id)
//     }

//     /// Get a queued tweenable animation from its ID.
//     ///
//     /// This fails and returns [`TweeningError::InvalidTweenId`] if the
//     /// animation has completed and was removed from the animator's internal
//     /// queue, or if the ID is invalid (notably, `Entity::PLACEHOLDER`).
//     #[inline]
//     pub fn try_get(&self, id: Entity) -> Result<&TweenAnim, TweeningError> {
//         self.anims.get(id).ok_or(TweeningError::InvalidTweenId(id))
//     }

//     /// Get a queued tweenable animation from its ID.
//     ///
//     /// This fails and returns [`TweeningError::InvalidTweenId`] if the
//     /// animation has completed and was removed from the animator's internal
//     /// queue, or if the ID is invalid (notably, `Entity::PLACEHOLDER`).
//     #[inline]
//     pub fn try_get_mut(&mut self, id: Entity) -> Result<&mut TweenAnim,
// TweeningError> {         self.anims
//             .get_mut(id)
//             .ok_or(TweeningError::InvalidTweenId(id))
//     }

//     /// Get an iterator over the queued tweenable animations.
//     ///
//     /// # Returns
//     ///
//     /// An iterator over pairs of (ID, animation) for all animations still in
//     /// the internal queue.
//     #[inline]
//     pub fn iter(&self) -> impl Iterator<Item = (Entity, &TweenAnim)> {
//         self.anims.iter()
//     }

//     /// Get a mutable iterator over the queued tweenable animations.
//     ///
//     /// # Returns
//     ///
//     /// An iterator over pairs of (ID, animation) for all animations still in
//     /// the internal queue.
//     #[inline]
//     pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut
// TweenAnim)> {         self.anims.iter_mut()
//     }

//     /// Remove a queued tweenable animation and return it.
//     ///
//     /// This immediately removes the animation, if it exists, without
// modifying     /// further its target. After the animation is removed, the
// `id` is     /// invalid and will never be reused. The animation cannot be
// re-queued;     /// instead, a new animation sould be created with
// [`add_component()`] or     /// [`add_asset()`], which will generate a
// different ID.     ///
//     /// # Returns
//     ///
//     /// If the ID was valid and an animation with this ID was removed,
// returns     /// that animation. Otherwise returns `None`.
//     ///
//     /// [`add_component()`]: Self::add_component
//     /// [`add_asset()`]: Self::add_asset
//     #[inline]
//     pub fn remove(&mut self, id: Entity) -> Option<TweenAnim> {
//         self.anims.remove(id)
//     }

//     /// Retarget a queued animation.
//     ///
//     /// Attempt to change the target of a tweening animation already
// enqueued,     /// and possibly already playing. This function performs a
// number of checks     /// on the new target to ensure it's compatible with the
// previous     /// target. In particular, the new target needs to have:
//     /// - the same kind (component or asset);
//     /// - the same type.
//     ///
//     /// # Returns
//     ///
//     /// On success, returns the previous target which has been replaced.
//     ///
//     /// [`add_component()`]: Self::add_component
//     /// [`add_asset()`]: Self::add_asset
//     /// [`set_target()`]: Self::set_target
//     pub fn set_target(
//         &mut self,
//         id: Entity,
//         target: AnimTarget,
//     ) -> Result<AnimTarget, TweeningError> {
//         let anim = self
//             .anims
//             .get_mut(id)
//             .ok_or(TweeningError::InvalidTweenId(id))?;
//         match (anim.target, target) {
//             (AnimTarget::Component(old_component),
// AnimTarget::Component(new_component)) => {                 if
// old_component.component_id != new_component.component_id {
// return Err(TweeningError::MismatchingComponentId(
// old_component.component_id,
// new_component.component_id,                     ));
//                 }
//             }
//             (AnimTarget::Asset(old_asset), AnimTarget::Asset(new_asset)) => {
//                 if old_asset.resource_id != new_asset.resource_id {
//                     return Err(TweeningError::MismatchingAssetResourceId(
//                         old_asset.resource_id,
//                         new_asset.resource_id,
//                     ));
//                 }
//             }
//             _ => {
//                 return Err(TweeningError::MismatchingTargetKind(
//                     anim.target.is_component(),
//                     target.is_component(),
//                 ))
//             }
//         }
//         let old_target = anim.target;
//         anim.target = target;
//         Ok(old_target)
//     }

//     /// Swap a queued animation.
//     ///
//     /// Attempt to change the tweenable of an animation already enqueued, and
//     /// possibly already playing.
//     ///
//     /// If the tweenable is successfully swapped, this resets the
//     /// [`TweenAnim::tween_state()`] to [`TweenState::Active`], even if the
//     /// tweenable would otherwise be completed _e.g._ because its elapsed
// time     /// is past its total duration. Conversely, this doesn't update the
// target,     /// as this function doesn't have mutable access to it.
//     ///
//     /// To ensure the old and new animations have the same elapsed time (for
//     /// example if they need to be synchronized), call [`set_elapsed()`]
// first     ///   on the input `tweenable`, with the duration value of the old
//     ///   tweenable as returned by [`elapsed()`].
//     ///
//     /// ```
//     /// # use std::time::Duration;
//     /// # use bevy::prelude::*;
//     /// # use bevy_tweening::*;
//     /// # fn sys(mut animator: ResMut<TweenAnimator>) {
//     /// # let id = Entity::PLACEHOLDER;
//     /// # let mut tweenable = Delay::new(Duration::from_secs(1));
//     /// let elapsed = animator.get(id).unwrap().tweenable().elapsed();
//     /// tweenable.set_elapsed(elapsed);
//     /// animator.set_tweenable(id, tweenable);
//     /// # }
//     /// ```
//     ///
//     /// To recompute the actual tweenable animation state and force a target
//     /// update, use [`step_one()`] with a [`Duration::ZERO`].
//     ///
//     /// # Returns
//     ///
//     /// On success, returns the previous tweenable which has been swapped
// out.     ///
//     /// [`set_elapsed()`]: crate::Tweenable::set_elapsed
//     /// [`elapsed()`]: crate::Tweenable::elapsed
//     /// [`step_one()`]: Self::step_one
//     pub fn set_tweenable<T>(
//         &mut self,
//         id: Entity,
//         tweenable: T,
//     ) -> Result<BoxedTweenable, TweeningError>
//     where
//         T: Tweenable + 'static,
//     {
//         let anim = self
//             .anims
//             .get_mut(id)
//             .ok_or(TweeningError::InvalidTweenId(id))?;
//         let mut old_tweenable: BoxedTweenable = Box::new(tweenable);
//         std::mem::swap(&mut anim.tweenable, &mut old_tweenable);
//         // Reset tweening state, the new tweenable is at t=0
//         anim.tween_state = TweenState::Active;
//         Ok(old_tweenable)
//     }

//     /// Step the given queued animation.
//     ///
//     /// In general all animations are stepped automatically via
// [`step_all()`],     /// which is called from a system added by the
// [`TweeningPlugin`]. This     /// function can be used to step a single
// animation, either in addition or     /// in place of that system or any call
// to [`step_all()`]. If the `id` is     /// invalid, this function does
// nothing.     ///
//     /// One useful use of this function is to force a mutation on the target,
//     /// for example because some specific animation change was made which
//     /// couldn't directly mutate it. By passing [`Duration::ZERO`], this
//     /// function effectively forces the target state to the current animation
//     /// "position" without playing back the animation itself (the elapsed
// time     /// is not modified).
//     ///
//     /// # Returns
//     ///
//     /// If the animation completed and was destroyed, returns a copy of that
//     /// animation. Otherwise if the animation is still queued, returns
// `None`.     ///
//     /// [`step_all()`]: Self::step_all
//     #[inline]
//     pub fn step_one(
//         &mut self,
//         world: &mut World,
//         id: Entity,
//         delta_time: Duration,
//     ) -> Option<TweenAnim> {
//         world.resource_scope(|world, events:
// Mut<Events<CycleCompletedEvent>>| {             world.resource_scope(|world,
// anim_events: Mut<Events<AnimCompletedEvent>>| {                 let anim =
// self.anims.get_mut(id)?;

//                 let ret = Self::step_impl(
//                     id,
//                     anim,
//                     &self.asset_resolver,
//                     world,
//                     delta_time,
//                     events,
//                     anim_events,
//                 );

//                 if let Ok(ret) = ret {
//                     if ret.sent_commands {
//                         world.flush();
//                     }

//                     if !ret.retain {
//                         return Some(self.anims.remove(id).unwrap());
//                     }
//                 }
//                 None
//             })
//         })
//     }

//     fn step_impl(
//         tween_id: Entity,
//         anim: &mut TweenAnim,
//         asset_resolver: &HashMap<ComponentId, Resolver>,
//         world: &mut World,
//         delta_time: Duration,
//         mut events: Mut<Events<CycleCompletedEvent>>,
//         mut anim_events: Mut<Events<AnimCompletedEvent>>,
//     ) -> Result<StepResult, TweeningError> {
//         let mut queued_systems = Vec::with_capacity(8);
//         let mut completed_events = Vec::with_capacity(8);
//         let mut sent_commands = false;

//         // Sanity checks on fields which can be freely modified by the user
//         anim.speed = anim.speed.max(0.);

//         // Retain completed animations only if requested
//         if anim.tween_state == TweenState::Completed {
//             let ret = StepResult {
//                 retain: !anim.destroy_on_completion,
//                 sent_commands: false,
//             };
//             return Ok(ret);
//         }

//         // Skip paused animations (but retain them)
//         if anim.playback_state == PlaybackState::Paused || anim.speed <= 0. {
//             let ret = StepResult {
//                 retain: true,
//                 sent_commands: false,
//             };
//             return Ok(ret);
//         }

//         // Resolve the (untyped) target as a MutUntyped<T>
//         let Some(mut mut_untyped) = (match &anim.target {
//             AnimTarget::Component(ComponentAnimTarget {
//                 entity,
//                 component_id,
//             }) => world.get_mut_by_id(*entity, *component_id),
//             AnimTarget::Asset(AssetAnimTarget {
//                 resource_id,
//                 asset_id,
//             }) => {
//                 if let Some(assets) =
// world.get_resource_mut_by_id(*resource_id) {                     if let
// Some(resolver) = asset_resolver.get(resource_id) {
// resolver(assets, *asset_id)                     } else {
//                         None
//                     }
//                 } else {
//                     None
//                 }
//             }
//         }) else {
//             let ret = StepResult {
//                 retain: false,
//                 sent_commands: false,
//             };
//             return Ok(ret);
//         };

//         // Scale delta time by this animation's speed. Reject negative
// speeds; use         // backward playback to play in reverse direction.
//         // Note: must use f64 for precision; f32 produces visible roundings.
//         let delta_time = delta_time.mul_f64(anim.speed);

//         // Step the tweenable animation
//         let entity = anim.target.as_component().map(|comp| comp.entity);
//         let mut notify_completed = || {
//             completed_events.push((
//                 CycleCompletedEvent {
//                     id: tween_id,
//                     target: anim.target,
//                 },
//                 entity,
//             ));
//         };
//         let mut queue_system = |system_id: SystemId| {
//             queued_systems.push(system_id);
//         };
//         let state = anim.tweenable.step(
//             tween_id,
//             delta_time,
//             mut_untyped.reborrow(),
//             &mut notify_completed,
//             &mut queue_system,
//         );
//         anim.tween_state = state;

//         // Send tween completed events once we reclaimed mut access to world
// and can get         // a Commands.
//         if !completed_events.is_empty() {
//             let mut commands = world.commands();
//             sent_commands = true;

//             for (event, entity) in completed_events.drain(..) {
//                 // Send buffered event
//                 events.send(event);

//                 // Trigger all entity-scoped observers
//                 if let Some(entity) = entity {
//                     commands.trigger_targets(event, entity);
//                 }
//             }
//         }

//         // Execute one-shot systems
//         for sys_id in queued_systems.drain(..) {
//             let _ = world.run_system(sys_id);
//         }

//         // Raise animation completed event
//         if state == TweenState::Completed {
//             let event = AnimCompletedEvent {
//                 id: tween_id,
//                 target: anim.target,
//             };

//             // Send buffered event
//             anim_events.send(event);

//             // Trigger all entity-scoped observers
//             if let Some(entity) = entity {
//                 let mut commands = world.commands();
//                 sent_commands = true;
//                 commands.trigger_targets(event, entity);
//             }
//         }

//         let ret = StepResult {
//             retain: state == TweenState::Active ||
// !anim.destroy_on_completion,             sent_commands,
//         };
//         Ok(ret)
//     }

//     /// Step all queued animations.
//     ///
//     /// Loop over the internal queue of tweenable animations, apply them to
//     /// their respective target, and prune all the completed ones (the ones
//     /// returning [`TweenState::Completed`]). In the later case, send
//     /// [`AnimCompletedEvent`]s.
//     ///
//     /// If you use the [`TweeningPlugin`], this is automatically called by
// the     /// animation system the plugin registers. See the
//     /// [`AnimationSystem::AnimationUpdate`] system set.
//     ///
//     /// Note that the order in which the animations are iterated over and
//     /// played, and therefore also the order in which any event is raised or
// any     /// one-shot system is executed, is an unspecified implementation
// detail.     /// There is no guarantee on that order nor its stability frame
// to     /// frame.
//     pub fn step_all(&mut self, world: &mut World, delta_time: Duration) {
//         world.resource_scope(|world, mut events:
// Mut<Events<CycleCompletedEvent>>| {             world.resource_scope(|world,
// mut anim_events: Mut<Events<AnimCompletedEvent>>| {                 let mut
// sent_commands = false;

//                 // Loop over active animations, tick them, and retain those
// which are still                 // active after that
//                 self.anims.retain(|tween_id, anim| {
//                     let ret = Self::step_impl(
//                         tween_id,
//                         anim,
//                         &self.asset_resolver,
//                         world,
//                         delta_time,
//                         events.reborrow(),
//                         anim_events.reborrow(),
//                     );
//                     if let Ok(ret) = ret {
//                         sent_commands = sent_commands || ret.sent_commands;
//                         return ret.retain;
//                     }
//                     false
//                 });

//                 // Flush commands
//                 if sent_commands {
//                     world.flush();
//                 }
//             });
//         });
//     }
// }

/// Extension trait to interpolate between two colors.
///
/// This adds a [`Color::lerp()`] function which linearly interpolates the
/// `LinearRgba` values component-wise.
///
/// Note that this is a convenience helper with naive color interpolation. In
/// general, to get more accurrate colors, you should create your own [`Lens`]
/// and apply a better interpolation, for example based on luminosity. There's
/// no "canonical" color interpolation, and the best answer varies depending on
/// the context.
#[allow(dead_code)]
#[cfg(any(feature = "bevy_sprite", feature = "bevy_ui", feature = "bevy_text"))]
trait ColorLerper {
    fn lerp(&self, target: &Self, ratio: f32) -> Self;
}

#[allow(dead_code)]
#[cfg(any(feature = "bevy_sprite", feature = "bevy_ui", feature = "bevy_text"))]
impl ColorLerper for Color {
    fn lerp(&self, target: &Color, ratio: f32) -> Color {
        let src = self.to_linear();
        let dst = target.to_linear();
        let r = src.red.lerp(dst.red, ratio);
        let g = src.green.lerp(dst.green, ratio);
        let b = src.blue.lerp(dst.blue, ratio);
        let a = src.alpha.lerp(dst.alpha, ratio);
        Color::linear_rgba(r, g, b, a)
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::ecs::{change_detection::MaybeLocation, component::Tick};

    use super::*;
    use crate::test_utils::*;

    struct DummyLens {
        start: f32,
        end: f32,
    }

    #[derive(Debug, Default, Clone, Copy, Component)]
    struct DummyComponent {
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

    // #[test]
    // fn animator_default() {
    //     let mut animator = TweenAnimator::default();
    //     assert!(animator.get(Entity::PLACEHOLDER).is_none());
    //     assert!(animator.iter().next().is_none());
    //     assert!(animator.iter_mut().next().is_none());
    //     assert!(animator.remove(Entity::PLACEHOLDER).is_none());
    // }

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
        .with_completed_event(true);
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
        .with_completed_event(true);
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
            _trigger: Trigger<CycleCompletedEvent>,
            mut count: ResMut<Count<CycleCompletedEvent, GlobalMarker>>,
        ) {
            count.count += 1;
        }
        env.world.add_observer(observe_global);

        fn observe_entity(
            _trigger: Trigger<CycleCompletedEvent>,
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
}
