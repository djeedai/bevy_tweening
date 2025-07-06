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
//! two values, for any component and asset, including both built-in Bevy ones
//! and custom user-defined ones. Each field of a component or asset can be
//! animated via a collection of predefined easing functions, or providing a
//! custom animation curve. The library supports any number of animations queued
//! in parallel, even on the same component or asset type, and allows runtime
//! control over playback and animation speed.
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
//!     // Animation time.
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
//!     .spawn((Transform::default(),))
//!     // Queue the tweenable animation
//!     .tween(tween);
//! # }
//! ```
//!
//! This example shows the general pattern to add animations for any component
//! or asset. Since moving the position of an object is a very common
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
//!     // Create-and-queue a new Transform::translation animation
//!     .move_to(
//!         Vec3::new(1., 2., -4.),
//!         Duration::from_secs(1),
//!         EaseFunction::QuadraticInOut,
//!     );
//! # }
//! ```
//!
//! # Ready to animate
//!
//! Unlike previous versions of 🍃 Bevy Tweening, you don't need any particular
//! setup aside from adding the [`TweeningPlugin`] to your [`App`].
//! In particular, per-component-type systems are gone. Instead, the
//! [`TweenAnimator`] updates all tweenable animations for all components and
//! assets at once, even for custom component and asset types.
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
//! To execute multiple animations in parallel, simply enqueue each animation
//! independently. This require careful selection of timings.
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
//! infinite animation in a sequence, and append more tweenable after it, those
//! tweenable will never play because playback will be stuck forever repeating
//! the first animation. You're responsible for creating sequences that make
//! sense. In general, only use infinite tweenable animations alone or as the
//! last element of a sequence.
//!
//! # `TweenAnimator` and lenses
//!
//! Bevy components and assets are animated with the [`TweenAnimator`] resource.
//! The animator determine the component or asset to animate via an
//! [`AnimTarget`], and accesses its field(s) using a [`Lens`].
//!
//! - Components are animated via the [`ComponentTarget`], which identifies a
//!   component instance on an entity via the [`Entity`] itself and the
//!   [`ComponentId`] of the registered component type.
//! - Assets are animated in a similar way to component, via the [`AssetTarget`]
//!   which identifies an asset via the type of its [`Assets`] collection and
//!   the [`AssetId`] referencing that asset inside the collection.
//!
//! Because assets are typically shared, and the animation applies to the asset
//! itself, all users of the asset see the animation. For example, animating the
//! color of a [`ColorMaterial`] will change the color of all the
//! 2D meshes using that material. If you want to animate the color of a single
//! mesh, you have to duplicate the asset and assign a unique copy to that mesh,
//! then animate that copy alone.
//!
//! ## Lenses
//!
//! The [`AnimTarget`] references the container (component or asset) being
//! animated. However only a part of that component or asset is generally
//! animated. To that end, the [`TweenAnimator`] accesses the field(s) to
//! animate via a _lens_, a type that implements the [`Lens`] trait and allows
//! mapping a container to the actual value(s) animated.
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
//! provided for convenience and mainly as examples. In general 🍃 Bevy Tweening
//! expects you to write your own lenses by implementing the trait, which as you
//! can see above is very simple. This allows animating virtually any field of
//! any component or asset, whether shipped with Bevy or defined by the user.
//!
//! [`Transform::translation`]: https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.Transform.html#structfield.translation
//! [`Entity`]: https://docs.rs/bevy/0.16.0/bevy/ecs/entity/struct.Entity.html
//! [`Query`]: https://docs.rs/bevy/0.16.0/bevy/ecs/system/struct.Query.html
//! [`ColorMaterial`]: https://docs.rs/bevy/0.16.0/bevy/sprite/struct.ColorMaterial.html
//! [`Sprite`]: https://docs.rs/bevy/0.16.0/bevy/sprite/struct.Sprite.html
//! [`Node`]: https://docs.rs/bevy/0.16.0/bevy/ui/struct.Node.html#structfield.position
//! [`TextColor`]: https://docs.rs/bevy/0.16.0/bevy/text/struct.TextColor.html
//! [`Transform`]: https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.Transform.html
//! [`TransformPositionLens`]: crate::lens::TransformPositionLens

use std::time::Duration;

use bevy::{
    asset::UntypedAssetId,
    ecs::{change_detection::MutUntyped, component::ComponentId},
    platform::collections::HashMap,
    prelude::*,
};
pub use lens::Lens;
pub use plugin::{AnimationSystem, TweeningPlugin};
use slotmap::{new_key_type, SlotMap};
pub use tweenable::{
    BoxedTweenable, Delay, Sequence, TotalDuration, Tween, TweenAssetExtensions,
    TweenCompletedEvent, TweenState, Tweenable,
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
#[derive(Clone, Copy)]
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
/// [`Tweenable::step()`], generally by the [`TweenAnimator`], is added or
/// subtracted to the current time position on the animation's timeline.
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

new_key_type! {
    /// Unique identifier for a tweenable animation currently registered with the [`TweenAnimator`].
    pub struct TweenId;
}

/// Extensions for [`EntityCommands`] to queue tween-based animations.
///
/// This trait provide extension functions to [`EntityCommands`], allowing
/// convenient syntaxes like inserting a new component and immediately attaching
/// a tweenable animation to it in a single call.
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
/// This convenience comes at the price of reduced control and error checking.
/// Additional information like the [`TweenId`] of a newly created [`TweenAnim`]
/// cannot be retrieved. And any error (_e.g._ trying to insert an animation
/// with a tweenable of a component type while the entity doesn't have that
/// component) cannot be forwarded back to the caller, so will produce a panic
/// instead. This is best used for cases where you know those conditions at
/// build time. To avoid a panic, prefer manually queuing a new tweenable
/// animation through the [`TweenAnimator`].
pub trait EntityCommandsTweeningExtensions<'a> {
    /// Queue the given [`Tweenable`].
    ///
    /// This calls [`TweenAnimator::add()`] on the current entity, deriving the
    /// proper component to animate based on the type of the lens stored in the
    /// tweenable (see [`Tweenable::type_id()`]). That component must exists on
    /// the entity.
    fn tween<T>(&mut self, tweenable: T) -> &mut Self
    where
        T: Tweenable + 'static;

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
    /// access to the [`TweenId`] created. To retrieve the ID and control
    /// the animation playback, directly add the tweenable via
    /// [`TweenAnimator::add()`].
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
}

/// Build an [`EntityCommand`] which queues the new tweenable into the
/// [`Animator`].
fn make_tween_command<T>(tweenable: T) -> impl EntityCommand
where
    T: Tweenable + 'static,
{
    move |mut entity: EntityWorldMut| {
        let e = entity.id();
        let type_id = tweenable
            .type_id()
            .expect("Can't add untyped tweenable as root animation.");
        let component_id = entity
            .world()
            .components()
            .get_id(type_id)
            .expect("Tweenable component not registered with the World.");
        let target = ComponentTarget {
            entity: e,
            component_id,
        };
        entity
            .resource_mut::<TweenAnimator>()
            .queue(target.into(), tweenable);
    }
}

fn make_transform_from_command(
    end: Vec3,
    duration: Duration,
    ease_method: EaseMethod,
) -> impl EntityCommand {
    move |mut entity: EntityWorldMut| {
        let component_id = entity
            .world()
            .component_id::<Transform>()
            .expect("Transform component not registered in the World");
        let start = entity.get::<Transform>().unwrap().translation;
        let lens = lens::TransformPositionLens { start, end };
        let tween = Tween::new(ease_method, duration, lens);
        let e = entity.id();
        let target = ComponentTarget {
            entity: e,
            component_id,
        };
        entity
            .resource_mut::<TweenAnimator>()
            .queue(target.into(), tween);
    }
}

impl<'a> EntityCommandsTweeningExtensions<'a> for EntityCommands<'a> {
    fn tween<T>(&mut self, tweenable: T) -> &mut EntityCommands<'a>
    where
        T: Tweenable + 'static,
    {
        self.queue(make_tween_command(tweenable))
    }

    fn move_to(
        &mut self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut EntityCommands<'a> {
        self.queue(make_transform_from_command(
            end,
            duration,
            ease_method.into(),
        ))
    }
}

impl<'a> EntityCommandsTweeningExtensions<'a> for EntityWorldMut<'a> {
    fn tween<T>(&mut self, tweenable: T) -> &mut Self
    where
        T: Tweenable + 'static,
    {
        let type_id = tweenable.type_id().unwrap();
        let component_id = self.world().components().get_id(type_id).unwrap();
        let target = ComponentTarget {
            component_id,
            entity: self.id(),
        };
        self.world_scope(|world: &mut World| {
            world.resource_mut::<TweenAnimator>().add(target, tweenable);
        });
        self
    }

    fn move_to(
        &mut self,
        end: Vec3,
        duration: Duration,
        ease_method: impl Into<EaseMethod>,
    ) -> &mut Self {
        let component_id = self.world().component_id::<Transform>().unwrap();
        let start = self.get::<Transform>().unwrap().translation;
        let lens = lens::TransformPositionLens { start, end };
        let tween = Tween::new(ease_method, duration, lens);
        let e = self.id();
        let target = ComponentTarget {
            entity: e,
            component_id,
        };
        self.world_scope(|world: &mut World| {
            world.resource_mut::<TweenAnimator>().add(target, tween);
        });
        self
    }
}

/// Event raised when a [`TweenAnim`] completed.
#[derive(Copy, Clone, Event)]
pub struct AnimCompletedEvent {
    /// The ID of the tween animation which completed. Note that commonly the
    /// [`TweenAnim`] is pruned out of the [`TweenAnimator`] on completion, so
    /// can't be queried anymore with this ID. However animation IDs are unique,
    /// so this can be used to identify the tweenable animation from an ID
    /// stored by the user.
    pub id: TweenId,
    /// The animation target. This is provided both as a convenience for
    /// [`TweenAnim`]s not pruned from the [`TweenAnimator`] on completion, and
    /// because for those animations which are pruned the information is not
    /// available in anymore in another way.
    pub target: AnimTarget,
}

/// Component animation target.
///
/// References a component used as the target of a tweenable animation. The
/// component is identified by the ID of the component type as registered in the
/// [`World`] where the animation is queued, and the [`Entity`] holding the
/// component instance of that type.
///
/// This is a lightweight reference (copyable) implicitly tied to a given
/// [`World`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentTarget {
    /// Component ID of the registered component being animated.
    pub component_id: ComponentId,
    /// Entity holding the component instance being animated.
    pub entity: Entity,
}

/// Asset animation target.
///
/// References an asset used as the target of a tweenable animation. The asset
/// is identified by the ID of the [`Assets`] resource type registered in the
/// [`World`] where the animation is queued, and the unique asset ID identifying
/// the asset instance inside that [`Assets`] resource.
///
/// This is a lightweight reference (copyable) implicitly tied to a given
/// [`World`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetTarget {
    /// Resource ID of the registered [`Assets`] asset container.
    pub resource_id: ComponentId,
    /// Asset ID of the target asset being animated.
    pub asset_id: UntypedAssetId,
}

/// Animation target.
///
/// References either a component or an asset used as the target of a tweenable
/// animation. See [`ComponentTarget`] and [`AssetTarget`] for details.
///
/// This is a lightweight reference (copyable) implicitly tied to a given
/// [`World`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnimTarget {
    /// Component animation target.
    Component(ComponentTarget),
    /// Asset animation target.
    Asset(AssetTarget),
}

impl AnimTarget {
    /// Check if the target is a component.
    #[inline]
    pub fn is_component(&self) -> bool {
        matches!(*self, AnimTarget::Component(..))
    }

    /// Convert this target to a component target if possible.
    #[inline]
    pub fn as_component(&self) -> Option<&ComponentTarget> {
        if let Self::Component(component) = self {
            Some(component)
        } else {
            None
        }
    }

    /// Convert this target to a component target if possible.
    #[inline]
    pub fn as_component_mut(&mut self) -> Option<&mut ComponentTarget> {
        if let Self::Component(component) = self {
            Some(component)
        } else {
            None
        }
    }

    /// Check if the target is an asset.
    #[inline]
    pub fn is_asset(&self) -> bool {
        matches!(*self, AnimTarget::Asset(..))
    }

    /// Convert this target to an asset target if possible.
    #[inline]
    pub fn as_asset(&self) -> Option<&AssetTarget> {
        if let Self::Asset(asset) = self {
            Some(asset)
        } else {
            None
        }
    }

    /// Convert this target to an asset target if possible.
    #[inline]
    pub fn as_asset_mut(&mut self) -> Option<&mut AssetTarget> {
        if let Self::Asset(asset) = self {
            Some(asset)
        } else {
            None
        }
    }
}

impl From<ComponentTarget> for AnimTarget {
    fn from(value: ComponentTarget) -> Self {
        AnimTarget::Component(value)
    }
}

impl From<AssetTarget> for AnimTarget {
    fn from(value: AssetTarget) -> Self {
        AnimTarget::Asset(value)
    }
}

/// A [`Tweenable`]-based animation.
pub struct TweenAnim {
    /// Target [`Entity`] containing the component to animate, or target asset.
    pub target: AnimTarget,
    /// Animation description.
    pub tweenable: BoxedTweenable,
    /// Control if the animation is played or not.
    pub state: PlaybackState,
    /// Relative playback speed. Defaults to `1.` (normal speed).
    pub speed: f32,
}

impl TweenAnim {
    /// Create a new tween animation.
    ///
    /// # Panics
    ///
    /// Panics if the tweenable is "typeless", that is [`Tweenable::type_id()`]
    /// returns `None`. Root animations enqueued in the [`TweenAnimator`] must
    /// target a concrete component or asset type. This means in particular
    /// that you can't insert a single [`Delay`]. You can however use a
    /// [`Delay`] or other typeless tweenables as part of a [`Sequence`],
    /// provided there's at least one other typed tweenable in the sequence.
    pub fn new(target: AnimTarget, tweenable: impl Tweenable + 'static) -> Self {
        Self {
            target,
            tweenable: Box::new(tweenable),
            state: PlaybackState::Playing,
            speed: 1.,
        }
    }

    /// Create a new tween animation for an asset.
    ///
    /// The `type_id` must be `TypeId::of::<Assets<A>>`, where `A: Asset` is the
    /// asset type to animate. The `asset_id` is the type-erased ID of the
    /// actual asset, as stored in the `Assets<A>` container.
    ///
    /// # Panics
    ///
    /// Panics if the tweenable is "typeless", that is [`Tweenable::type_id()`]
    /// returns `None`. Root animations enqueued in the [`TweenAnimator`] must
    /// target a concrete component or asset type. This means in particular
    /// that you can't insert a single [`Delay`]. You can however use a
    /// [`Delay`] or other typeless tweenables as part of a [`Sequence`],
    /// provided there's at least one other typed tweenable in the sequence.
    ///
    /// Panics if the tweenable type is different from the `type_id` parameter,
    /// which identified the type of the [`Assets`] container storing the asset
    /// to animate.
    // pub fn new_asset(
    //     type_id: TypeId,
    //     asset_id: UntypedAssetId,
    //     tweenable: impl Tweenable + 'static,
    // ) -> Self {
    //     let tween_type_id = tweenable.type_id().expect(
    //         "Typeless tweenable like Delay can't be inserted as root tweenable in an animation.",
    //     );
    //     assert_eq!(
    //         type_id, tween_type_id,
    //         "The tweenable type doesn't match the Assets<A> type of the target asset."
    //     );
    //     Self {
    //         //target: AnimTarget::Entity(target),
    //         target: AnimTarget::Asset { type_id, asset_id },
    //         tweenable: Box::new(tweenable),
    //         state: PlaybackState::Playing,
    //         speed: 1.,
    //     }
    // }

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
        self.state = PlaybackState::Paused;
        self.tweenable.rewind();
    }
}

/// Animator for tween-based animations.
///
/// This resource stores all the active tweening animations for the entire
/// application. It's essentially a hash map from a [`TweenId`] uniquely
/// identifying an active animation, to the [`TweenAnim`] runtime data of that
/// animation. Use this resource to lookup animations by ID and modify their
/// runtime data, for example their playback speed.
#[derive(Resource)]
pub struct TweenAnimator {
    /// Queue of animations currently playing.
    anims: SlotMap<TweenId, TweenAnim>,
    /// Asset resolver allowing to convert a pair of { untyped pointer to
    /// `Assets<A>`, untyped `AssetId` } into an untyped pointer to the asset A
    /// itself. This is necessary because there's no UntypedAssets interface in
    /// Bevy. The TypeId key must be the type of the `Assets<A>` type itself.
    asset_resolver: HashMap<
        ComponentId,
        Box<
            dyn for<'w> Fn(MutUntyped<'w>, UntypedAssetId) -> MutUntyped<'w>
                + Send
                + Sync
                + 'static,
        >,
    >,
}

impl Default for TweenAnimator {
    fn default() -> Self {
        Self {
            anims: Default::default(),
            asset_resolver: Default::default(),
        }
    }
}

impl TweenAnimator {
    /// Add a new component animation to the animator queue.
    ///
    /// In general you don't need to call this directly. Instead, use the
    /// extensions provided by [`EntityCommandsTweeningExtensions`] to directly
    /// create and queue tweenable animations on a given [`EntityCommands`],
    /// like this:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_tweening::{lens::*, *};
    /// # use std::time::Duration;
    /// # let mut world = World::default();
    /// # world.register_component::<Transform>();
    /// # world.register_resource::<TweenAnimator>();
    /// # world.init_resource::<TweenAnimator>();
    /// # let entity = world.spawn_empty().id();
    /// let tween = Tween::new(
    ///     EaseFunction::QuadraticInOut,
    ///     Duration::from_secs(1),
    ///     TransformPositionLens {
    ///         start: Vec3::ZERO,
    ///         end: Vec3::new(3.5, 0., 0.),
    ///     },
    /// );
    /// world.entity_mut(entity).tween(tween);
    /// ```
    ///
    /// This function is still useful if you want to store the [`TweenId`] of
    /// the new animation, to later access it to dynamically modify the playback
    /// (e.g. speed).
    #[inline]
    pub fn add<T>(&mut self, target: ComponentTarget, tweenable: T) -> TweenId
    where
        T: Tweenable + 'static,
    {
        self.anims.insert(TweenAnim::new(target.into(), tweenable))
    }

    /// Queue a prepared tweenable animation.
    ///
    /// See [`make()`] to prepare the animation closure.
    ///
    /// [`make()`]: Animator::make
    #[inline]
    pub(crate) fn queue(&mut self, target: AnimTarget, tweenable: impl Tweenable + 'static) {
        self.anims.insert(TweenAnim::new(target, tweenable));
    }

    /// Get a tweenable from its ID.
    ///
    /// This fails and returns `None` if the tweenable has completed and was
    /// removed from the animator's internal queue.
    pub fn get(&self, id: TweenId) -> Option<&TweenAnim> {
        self.anims.get(id)
    }

    /// Get a tweenable from its ID.
    ///
    /// This fails and returns `None` if the tweenable has completed and was
    /// removed from the animator's internal queue.
    pub fn get_mut(&mut self, id: TweenId) -> Option<&mut TweenAnim> {
        self.anims.get_mut(id)
    }

    /// Get an iterator over the active tweens.
    pub fn iter(&self) -> impl Iterator<Item = (TweenId, &TweenAnim)> {
        self.anims.iter()
    }

    /// Get a mutable iterator over the active tweens.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (TweenId, &mut TweenAnim)> {
        self.anims.iter_mut()
    }

    /// Play all queued animations.
    ///
    /// Loop over the internal queue of tweenable animations, apply them to
    /// their respective target [`Entity`], and prune all the completed
    /// ones (the ones returning [`TweenState::Completed`]). In the later case,
    /// send [`TweenCompletedEvent`]s if enabled on each individual tweenable.
    ///
    /// If you use the [`TweeningPlugin`], this is automatically called by the
    /// animation system the plugin registers. See the
    /// [`AnimationSystem::AnimationUpdate`] system set.
    pub fn play(
        &mut self,
        world: &mut World,
        delta_time: Duration,
        mut events: Mut<Events<TweenCompletedEvent>>,
        mut anim_events: Mut<Events<AnimCompletedEvent>>,
    ) {
        // Loop over active animations, tick them, and retain those which are still
        // active after that
        self.anims.retain(|tween_id, anim| {
            // Note: we use get_entity_mut([Entity; 1]) with an array of a single element to
            // get an EntityMut instead of an EntityWorldMut, as the former is
            // enough. This can allow optimizing by parallelizing tweening of
            // separate entities (which can't be done with EntityWorldMut has it
            // has exclusive World access). For now this has no consequence except
            // simplifying some borrow checker complications with World.
            //let ent_mut = &mut world.get_entity_mut([anim.target]).unwrap()[0];

            let Some(mut mut_untyped) = (match &anim.target {
                AnimTarget::Component(ComponentTarget {
                    entity,
                    component_id,
                }) => world.get_mut_by_id(*entity, *component_id),
                AnimTarget::Asset(AssetTarget {
                    resource_id,
                    asset_id,
                }) => {
                    if let Some(assets) = world.get_resource_mut_by_id(*resource_id) {
                        if let Some(resolver) = self.asset_resolver.get(resource_id) {
                            Some(resolver(assets, *asset_id))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }) else {
                return false;
            };

            // Scale delta time by this animation's speed. Reject negative speeds; use
            // backward playback to play in reverse direction.
            let delta_time = delta_time.mul_f32(anim.speed.max(0.));

            let mut notify_completed = |fraction: f32| {
                events.send(TweenCompletedEvent {
                    id: tween_id,
                    target: anim.target,
                    progress: fraction,
                });
            };

            // Apply the animation tweenable
            let state = anim.tweenable.step(
                tween_id,
                delta_time,
                mut_untyped.reborrow(),
                &mut notify_completed,
            );

            // Raise completed event
            if state == TweenState::Completed {
                anim_events.send(AnimCompletedEvent {
                    id: tween_id,
                    target: anim.target,
                });
            }

            state == TweenState::Active
        });
    }
}

// /// Component to control the animation of an asset.
// ///
// /// The animated asset is the asset referenced by a [`Handle<T>`] component
// /// located on the same entity as the [`AssetAnimator<T>`] itself.
// #[cfg(feature = "bevy_asset")]
// #[derive(Component)]
// pub struct AssetAnimator<T: Asset> {
//     /// Control if this animation is played or not.
//     pub state: AnimatorState,
//     tweenable: BoxedTweenable<T>,
//     speed: f32,
// }

// #[cfg(feature = "bevy_asset")]
// impl<T: Asset + std::fmt::Debug> std::fmt::Debug for AssetAnimator<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("AssetAnimator")
//             .field("state", &self.state)
//             .finish()
//     }
// }

// #[cfg(feature = "bevy_asset")]
// impl<T: Asset> AssetAnimator<T> {
//     /// Create a new asset animator component from a single tweenable.
//     #[must_use]
//     pub fn new(tween: impl Tweenable<T> + 'static) -> Self {
//         Self {
//             state: default(),
//             tweenable: Box::new(tween),
//             speed: 1.,
//         }
//     }

//     //animator_impl!();
// }

/// Trait to interpolate between two values.
/// Needed for color.
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
    use bevy::ecs::{change_detection::MaybeLocation, component::Tick};
    use slotmap::Key as _;

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

    #[cfg(feature = "bevy_asset")]
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

    #[cfg(feature = "bevy_asset")]
    impl Lens<DummyAsset> for DummyLens {
        fn lerp(&mut self, mut target: Mut<DummyAsset>, ratio: f32) {
            target.value = self.start.lerp(self.end, ratio);
        }
    }

    #[cfg(feature = "bevy_asset")]
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
        let tweening_direction = PlaybackDirection::default();
        assert_eq!(tweening_direction, PlaybackDirection::Forward);
    }

    #[test]
    fn animator_state() {
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

    #[test]
    fn animator_default() {
        let mut animator = TweenAnimator::default();
        assert!(animator.get(TweenId::null()).is_none());
        assert!(animator.iter().next().is_none());
        assert!(animator.iter_mut().next().is_none());
    }

    // #[test]
    // fn animator_with_state() {
    //     for state in [AnimatorState::Playing, AnimatorState::Paused] {
    //         let tween = Tween::<DummyComponent>::new(
    //             EaseFunction::QuadraticInOut,
    //             Duration::from_secs(1),
    //             DummyLens { start: 0., end: 1. },
    //         );
    //         let animator = Animator::new(tween).with_state(state);
    //         assert_eq!(animator.state, state);

    //         // impl Debug
    //         let debug_string = format!("{:?}", animator);
    //         assert_eq!(
    //             debug_string,
    //             format!("Animator {{ state: {:?} }}", animator.state)
    //         );
    //     }
    // }

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

    // #[test]
    // fn animator_speed() {
    //     let tween = Tween::<DummyComponent>::new(
    //         EaseFunction::QuadraticInOut,
    //         Duration::from_secs(1),
    //         DummyLens { start: 0., end: 1. },
    //     );

    //     let mut animator = Animator::new(tween);
    //     assert_approx_eq!(animator.speed(), 1.); // default speed

    //     animator.set_speed(2.4);
    //     assert_approx_eq!(animator.speed(), 2.4);

    //     let tween = Tween::<DummyComponent>::new(
    //         EaseFunction::QuadraticInOut,
    //         Duration::from_secs(1),
    //         DummyLens { start: 0., end: 1. },
    //     );

    //     let animator = Animator::new(tween).with_speed(3.5);
    //     assert_approx_eq!(animator.speed(), 3.5);
    // }

    // #[test]
    // fn animator_set_tweenable() {
    //     let tween = Tween::<DummyComponent>::new(
    //         EaseFunction::QuadraticInOut,
    //         Duration::from_secs(1),
    //         DummyLens { start: 0., end: 1. },
    //     );
    //     let mut animator = Animator::new(tween);

    //     let tween2 = Tween::<DummyComponent>::new(
    //         EaseFunction::QuadraticInOut,
    //         Duration::from_secs(2),
    //         DummyLens { start: 0., end: 1. },
    //     );
    //     animator.set_tweenable(tween2);

    //     assert_eq!(animator.tweenable().duration(), Duration::from_secs(2));
    // }

    // #[cfg(feature = "bevy_asset")]
    // #[test]
    // fn asset_animator_with_state() {
    //     for state in [AnimatorState::Playing, AnimatorState::Paused] {
    //         let tween = Tween::new(
    //             EaseFunction::QuadraticInOut,
    //             Duration::from_secs(1),
    //             DummyLens { start: 0., end: 1. },
    //         );
    //         let animator = AssetAnimator::new(tween).with_state(state);
    //         assert_eq!(animator.state, state);

    //         // impl Debug
    //         let debug_string = format!("{:?}", animator);
    //         assert_eq!(
    //             debug_string,
    //             format!("AssetAnimator {{ state: {:?} }}", animator.state)
    //         );
    //     }
    // }

    // #[cfg(feature = "bevy_asset")]
    // #[test]
    // fn asset_animator_controls() {
    //     let tween = Tween::new(
    //         EaseFunction::QuadraticInOut,
    //         Duration::from_secs(1),
    //         DummyLens { start: 0., end: 1. },
    //     );
    //     let mut animator = AssetAnimator::new(tween);
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

    // #[cfg(feature = "bevy_asset")]
    // #[test]
    // fn asset_animator_set_tweenable() {
    //     let tween = Tween::new(
    //         EaseFunction::QuadraticInOut,
    //         Duration::from_secs(1),
    //         DummyLens { start: 0., end: 1. },
    //     );
    //     let mut animator = AssetAnimator::new(tween);

    //     let tween2 = Tween::new(
    //         EaseFunction::QuadraticInOut,
    //         Duration::from_secs(2),
    //         DummyLens { start: 0., end: 1. },
    //     );
    //     animator.set_tweenable(tween2);

    //     assert_eq!(animator.tweenable().duration(), Duration::from_secs(2));
    // }
}
