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
//! commands.spawn((
//!     // Spawn an entity to animate the position of.
//!     Transform::default(),
//!     // Add an Animator component to control and execute the animation.
//!     Animator::new(tween),
//! ));
//! # }
//! ```
//!
//! Note that this example leverages the fact [`TweeningPlugin`] automatically
//! adds the necessary system to animate [`Transform`] components. However, for
//! most other components and assets, you need to manually add those systems to
//! your `App`.
//!
//! # System setup
//!
//! Adding the [`TweeningPlugin`] to your app provides the basic setup for using
//! 🍃 Bevy Tweening. However, additional setup is required depending on the
//! components and assets you want to animate:
//!
//! - To ensure a component `C` is animated, the
//!   [`component_animator_system::<C>`] system must run each frame, in addition
//!   of adding an [`Animator::<C>`] component to the same Entity as `C`.
//!
//! - To ensure an asset `A` is animated, the [`asset_animator_system::<A>`]
//!   system must run each frame, in addition of adding an [`AssetAnimator<A>`]
//!   component to any Entity. Animating assets also requires the `bevy_asset`
//!   feature (enabled by default).
//!
//! By default, 🍃 Bevy Tweening adopts a minimalist approach, and the
//! [`TweeningPlugin`] will only add systems to animate components and assets
//! for which a [`Lens`] is provided by 🍃 Bevy Tweening itself. This means that
//! any other Bevy component or asset (either built-in from Bevy itself, or
//! custom) requires manually scheduling the appropriate system.
//!
//! | Component or Asset | Animation system added by `TweeningPlugin`? |
//! |---|---|
//! | [`Transform`]          | Yes                           |
//! | [`Sprite`]             | Only if `bevy_sprite` feature |
//! | [`ColorMaterial`]      | Only if `bevy_sprite` feature |
//! | [`Node`]               | Only if `bevy_ui` feature     |
//! | [`Text`]               | Only if `bevy_text` feature   |
//! | All other components   | No                            |
//!
//! To add a system for a component `C`, use:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_tweening::*;
//! # let mut app = App::default();
//! # #[derive(Component)] struct C;
//! app.add_systems(
//!     Update,
//!     component_animator_system::<C>.in_set(AnimationSystem::AnimationUpdate),
//! );
//! ```
//!
//! Similarly for an asset `A`, use the `asset_animator_system`. This is only
//! available with the `bevy_asset` feature.
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
//! [`Transform::translation`]: https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.Transform.html#structfield.translation
//! [`Entity`]: https://docs.rs/bevy/0.16.0/bevy/ecs/entity/struct.Entity.html
//! [`Query`]: https://docs.rs/bevy/0.16.0/bevy/ecs/system/struct.Query.html
//! [`ColorMaterial`]: https://docs.rs/bevy/0.16.0/bevy/sprite/struct.ColorMaterial.html
//! [`Sprite`]: https://docs.rs/bevy/0.16.0/bevy/sprite/struct.Sprite.html
//! [`Node`]: https://docs.rs/bevy/0.16.0/bevy/ui/struct.Node.html#structfield.position
//! [`TextColor`]: https://docs.rs/bevy/0.16.0/bevy/text/struct.TextColor.html
//! [`Transform`]: https://docs.rs/bevy/0.16.0/bevy/transform/components/struct.Transform.html

use std::{any::TypeId, time::Duration};

use bevy::{
    asset::UntypedAssetId, ecs::change_detection::MutUntyped, platform::collections::HashMap,
    prelude::*,
};
pub use lens::Lens;
pub use plugin::{AnimationSystem, TweeningPlugin};
use slotmap::{new_key_type, SlotMap};
pub use tweenable::{
    BoxedTweenable, Delay, Sequence, TotalDuration, Tween, TweenAssetExtensions, TweenCompleted,
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

/// What to do when a tween animation needs to be repeated.
///
/// Only applicable when [`RepeatCount`] is greater than the animation duration.
///
/// Default: `Repeat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatStrategy {
    /// Reset the animation back to its starting position.
    ///
    /// When playback reaches the end of the animation, it jumps directly back
    /// to the animation start. This can create discontinuities if the animation
    /// is not authored to be looping.
    Repeat,
    /// Follow a ping-pong pattern, changing the direction each time an endpoint
    /// is reached.
    ///
    /// A complete cycle start -> end -> start always counts as 2 loop
    /// iterations for the various operations where looping matters. That
    /// is, a 1 second animation will take 2 seconds to end up back where it
    /// started.
    ///
    /// This strategy ensures that there's no discontinuity in the animation,
    /// since there's no jump.
    MirroredRepeat,
}

impl Default for RepeatStrategy {
    fn default() -> Self {
        Self::Repeat
    }
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
/// This function is applied to the animation fraction `t` representing the
/// playback position over the animation duration. The result is used to
/// interpolate the animator target.
///
/// In general a [`Lens`] should perform a linear interpolation over its target,
/// and the non-linear behavior (for example, bounciness, etc.) comes from this
/// function. This ensures the same [`Lens`] can be reused in multiple contexts,
/// while the "shape" of the animation is controlled independently.
///
/// Default: `Linear`.
#[derive(Clone, Copy)]
pub enum EaseMethod {
    /// Follow [`EaseFunction`].
    EaseFunction(EaseFunction),
    /// Discrete interpolation. The eased value will jump from start to end when
    /// stepping over the discrete limit, which must be value between 0 and 1.
    Discrete(f32),
    /// Use a custom function to interpolate the value.
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
///
/// Default: `Forward`.
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

new_key_type! {
    /// Unique identifier for a tweenable.
    pub struct TweenId;
}

/// Extensions for [`EntityCommands`] to queue tween-based animations.
pub trait EntityCommandsTweeningExtensions<'a> {
    /// Queue the given [`Tweenable`].
    fn tween<T>(&mut self, tweenable: T) -> &mut Self
    where
        T: Tweenable + 'static;

    /// Queue a new tween animation to move the current entity.
    ///
    /// The entity must have a [`Transform`] component. The tween animation will
    /// be initialized with the current [`Transform::translation`] as its
    /// starting point, and the given endpoint and duration.
    ///
    /// Note that the starting point position is saved when the command is
    /// applied, generally after the current system when [`apply_deferred()`]
    /// runs. So any change to [`Transform::translation`] between this call and
    /// [`apply_deferred()`] will be taken into account.
    fn move_to(&mut self, end: Vec3, duration: Duration) -> &mut Self;
}

/// Build an [`EntityCommand`] which queues the new tweenable into the
/// [`Animator`].
fn make_tween_command<T>(tweenable: T) -> impl EntityCommand
where
    T: Tweenable + 'static,
{
    move |mut entity: EntityWorldMut| {
        let e = entity.id();
        entity.resource_mut::<TweenAnimator>().queue(e, tweenable);
    }
}

fn make_transform_from_command(end: Vec3, duration: Duration) -> impl EntityCommand {
    move |mut entity: EntityWorldMut| {
        let start = entity.get::<Transform>().unwrap().translation;
        let lens = lens::TransformPositionLens { start, end };
        let tween = Tween::new(EaseFunction::Linear, duration, lens);
        let e = entity.id();
        entity.resource_mut::<TweenAnimator>().queue(e, tween);
    }
}

impl<'a> EntityCommandsTweeningExtensions<'a> for EntityCommands<'a> {
    fn tween<T>(&mut self, tweenable: T) -> &mut EntityCommands<'a>
    where
        T: Tweenable + 'static,
    {
        self.queue(make_tween_command(tweenable))
    }

    fn move_to(&mut self, end: Vec3, duration: Duration) -> &mut EntityCommands<'a> {
        self.queue(make_transform_from_command(end, duration))
    }
}

/// Event raised when a [`TweenAnim`] completed.
#[derive(Copy, Clone, Event)]
pub struct AnimCompleted {
    /// The ID of the tween animation which completed.
    pub id: TweenId,
    /// The [`Entity`] the animation which completed is attached to.
    pub entity: Entity,
}

///
#[derive(Debug, Clone, Copy)]
pub enum AnimTarget {
    ///
    Entity {
        ///
        type_id: TypeId,
        ///
        entity: Entity,
    },
    ///
    Asset {
        ///
        type_id: TypeId,
        ///
        asset_id: UntypedAssetId,
    },
}

// impl AnimTarget {
//     pub fn resolve<'w>(&self, world: &'w mut World) ->
// Result<AnimResolvedTarget<'w>, ()> {         match self {
//             AnimTarget::Entity(target) => {
//                 let ent_mut = &mut
// world.get_entity_mut([*target]).unwrap()[0];
// Ok(AnimResolvedTarget::Entity(ent_mut))             }
//             AnimTarget::Asset(asset_id) => {
//                 if let Some(comp_id) =
// world.components().get_resource_id(asset_id.type_id()) {
// if let Some(mut_untyped) = world.get_resource_mut_by_id(comp_id) {
//                         return Ok(AnimResolvedTarget::Asset(mut_untyped));
//                     }
//                 }
//                 Err(())
//             }
//         }
//     }
// }

// pub enum AnimResolvedTarget<'w> {
//     Entity(&'w mut EntityMut<'w>),
//     Asset(MutUntyped<'w>),
// }

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
    pub fn new(target: Entity, tweenable: impl Tweenable + 'static) -> Self {
        let type_id = tweenable.type_id().expect(
            "Typeless tweenable like Delay can't be inserted as root tweenable in an animation.",
        );
        Self {
            //target: AnimTarget::Entity(target),
            target: AnimTarget::Entity {
                type_id,
                entity: target,
            },
            tweenable: Box::new(tweenable),
            state: PlaybackState::Playing,
            speed: 1.,
        }
    }

    /// Create a new tween animation for an asset.
    pub fn new_asset(
        type_id: TypeId,
        asset_id: UntypedAssetId,
        tweenable: impl Tweenable + 'static,
    ) -> Self {
        let tween_type_id = tweenable.type_id().expect(
            "Typeless tweenable like Delay can't be inserted as root tweenable in an animation.",
        );
        assert_eq!(
            type_id, tween_type_id,
            "The tweenable type doesn't match the Assets<A> type of the target asset."
        );
        Self {
            //target: AnimTarget::Entity(target),
            target: AnimTarget::Asset { type_id, asset_id },
            tweenable: Box::new(tweenable),
            state: PlaybackState::Playing,
            speed: 1.,
        }
    }

    /// Stop animation playback and rewind the animation.
    ///
    /// This changes the animator state to [`AnimatorState::Paused`] and  rewind
    /// its tweenable.
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
    /// Assets<A>, untyped AssetId } into an untyped pointer to the asset A
    /// itself. This is necessary because there's no UntypedAssets interface in
    /// Bevy. The TypeId key must be the type of the Assets<A> type itself.
    asset_resolver: HashMap<
        TypeId,
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
    /// extensions provided by [`EntityCommandsTweening`] to directly create and
    /// queue tweenable animations on a given [`EntityCommands`], like this:
    ///
    /// ```
    /// # use bevy::prelude::*;
    /// # use bevy_tweening::{lens::*, *};
    /// # use std::time::Duration;
    /// # let mut world = World::default();
    /// # let entity = Entity::PLACEHOLDER;
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
    pub fn add<T>(&mut self, target: Entity, tweenable: T) -> TweenId
    where
        T: Tweenable + 'static,
    {
        self.anims.insert(TweenAnim::new(target, tweenable))
    }

    /// Add a new asset animation to the animator queue.
    ///
    /// This creates a new animation with the given tweenable, targeting the
    /// asset with the given ID.
    ///
    /// # Panics
    ///
    /// Panics if the target type of the tweenable (which can be queried with
    /// [`Tweenable::type_id()`]) is different from `Assets<A>`. In general if
    /// the tweenable was created with _e.g._ [`Tween::new_asset<A>()`] then
    /// [`Tweenable::type_id()`] correctly returns `Assets<A>`. Note that the
    /// type is **NOT** the asset type `A` itself, but rather `Assets<A>`.
    #[inline]
    pub fn add_asset<A, T, I>(&mut self, asset_id: I, tweenable: T) -> TweenId
    where
        A: Asset,
        T: Tweenable + 'static,
        I: Into<AssetId<A>>,
    {
        let type_id = TypeId::of::<Assets<A>>();
        assert_eq!(
            Some(type_id),
            tweenable.type_id(),
            "Tweenable has different target type than Assets<A>."
        );

        self.asset_resolver.entry(type_id).or_insert_with(|| {
            Box::new(
                // Convert ( Mut<Assets<A>>, AssetId<A> ) -> Mut<A>
                |assets: MutUntyped, asset_id: UntypedAssetId| -> MutUntyped {
                    // SAFETY: The type ID was checked above with an assert to make sure the one
                    // stored in the Tweenable (which is where the untyped value comes from) is
                    // the same as Assets<A>.
                    #[allow(unsafe_code)]
                    let assets = unsafe { assets.with_type::<Assets<A>>() };

                    let asset_id = asset_id.typed::<A>();
                    let asset: Mut<A> = assets.map_unchanged(|a| {
                        // FIXME - replace with get_mut_untracked() to skip ECS change detection; see https://github.com/bevyengine/bevy/issues/13104
                        a.get_mut(asset_id).unwrap()
                    });

                    asset.into()
                },
            )
        });

        let asset_id: AssetId<A> = asset_id.into();
        self.anims
            .insert(TweenAnim::new_asset(type_id, asset_id.untyped(), tweenable))
    }

    /// Queue a prepared tweenable animation.
    ///
    /// See [`make()`] to prepare the animation closure.
    ///
    /// [`make()`]: Animator::make
    #[inline]
    pub(crate) fn queue(&mut self, target: Entity, tweenable: impl Tweenable + 'static) {
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
    /// send [`TweenCompleted`] events if enabled on each individual tweenable.
    pub fn play(
        &mut self,
        world: &mut World,
        delta_time: Duration,
        mut events: Mut<Events<TweenCompleted>>,
        mut anim_events: Mut<Events<AnimCompleted>>,
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

            // FIXME
            let mut target_entity = Entity::PLACEHOLDER;

            let Some(mut mut_untyped) = (match &anim.target {
                AnimTarget::Entity { entity, type_id } => {
                    target_entity = *entity;
                    if let Some(component_id) = world.components().get_id(*type_id) {
                        world.get_mut_by_id(*entity, component_id)
                    } else {
                        None
                    }
                }
                AnimTarget::Asset { type_id, asset_id } => {
                    if let Some(resource_id) = world.components().get_resource_id(*type_id) {
                        if let Some(assets) = world.get_resource_mut_by_id(resource_id) {
                            if let Some(resolver) = self.asset_resolver.get(type_id) {
                                Some(resolver(assets, *asset_id))
                            } else {
                                None
                            }
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

            // Apply the animation tweenable
            let state = anim.tweenable.step(
                tween_id,
                delta_time,
                mut_untyped.reborrow(),
                events.reborrow(),
            );

            // Raise completed event
            if state == TweenState::Completed {
                anim_events.send(AnimCompleted {
                    id: tween_id,
                    entity: target_entity,
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
        let tweening_direction = TweeningDirection::default();
        assert_eq!(tweening_direction, TweeningDirection::Forward);
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
