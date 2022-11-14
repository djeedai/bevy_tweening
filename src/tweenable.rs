use std::{ops::DerefMut, time::Duration};

use bevy::prelude::*;

#[cfg(feature = "bevy_asset")]
use bevy::asset::{Asset, HandleId};

use crate::{EaseMethod, Lens, RepeatCount, RepeatStrategy, TweeningDirection};

/// The dynamic tweenable type.
///
/// When creating lists of tweenables, you will need to box them to create a
/// homogeneous array like so:
/// ```no_run
/// # use bevy::prelude::Transform;
/// # use bevy_tweening::{BoxedTweenable, Delay, Sequence, Tween};
/// #
/// # let delay: Delay<Transform> = unimplemented!();
/// # let tween: Tween<Transform> = unimplemented!();
///
/// Sequence::new([Box::new(delay) as BoxedTweenable<Transform>, tween.into()]);
/// ```
///
/// When using your own [`Tweenable`] types, APIs will be easier to use if you
/// implement [`From`]:
/// ```no_run
/// # use std::time::Duration;
/// # use bevy::prelude::{Entity, Events, Mut, Transform};
/// # use bevy_tweening::{BoxedTweenable, Sequence, Tweenable, TweenCompleted, TweenState, Targetable, TotalDuration};
/// #
/// # struct MyTweenable;
/// # impl Tweenable<Transform> for MyTweenable {
/// #     fn duration(&self) -> Duration  { unimplemented!() }
/// #     fn total_duration(&self) -> TotalDuration  { unimplemented!() }
/// #     fn set_elapsed(&mut self, elapsed: Duration)  { unimplemented!() }
/// #     fn elapsed(&self) -> Duration  { unimplemented!() }
/// #     fn tick<'a>(&mut self, delta: Duration, target: &'a mut dyn Targetable<Transform>, entity: Entity, events: &mut Mut<Events<TweenCompleted>>) -> TweenState  { unimplemented!() }
/// #     fn rewind(&mut self) { unimplemented!() }
/// # }
///
/// Sequence::new([Box::new(MyTweenable) as BoxedTweenable<_>]);
///
/// // OR
///
/// Sequence::new([MyTweenable]);
///
/// impl From<MyTweenable> for BoxedTweenable<Transform> {
///     fn from(t: MyTweenable) -> Self {
///         Box::new(t)
///     }
/// }
/// ```
pub type BoxedTweenable<T> = Box<dyn Tweenable<T> + 'static>;

/// Playback state of a [`Tweenable`].
///
/// This is returned by [`Tweenable::tick()`] to allow the caller to execute
/// some logic based on the updated state of the tweenable, like advanding a
/// sequence to its next child tweenable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweenState {
    /// The tweenable is still active, and did not reach its end state yet.
    Active,
    /// Animation reached its end state. The tweenable is idling at its latest
    /// time.
    ///
    /// Note that [`RepeatCount::Infinite`] tweenables never reach this state.
    Completed,
}

/// Event raised when a tween completed.
///
/// This event is raised when a tween completed. When looping, this is raised
/// once per iteration. In case the animation direction changes
/// ([`RepeatStrategy::MirroredRepeat`]), an iteration corresponds to a single
/// progress from one endpoint to the other, whatever the direction. Therefore a
/// complete cycle start -> end -> start counts as 2 iterations and raises 2
/// events (one when reaching the end, one when reaching back the start).
///
/// # Note
///
/// The semantic is slightly different from [`TweenState::Completed`], which
/// indicates that the tweenable has finished ticking and do not need to be
/// updated anymore, a state which is never reached for looping animation. Here
/// the [`TweenCompleted`] event instead marks the end of a single loop
/// iteration.
#[derive(Copy, Clone)]
pub struct TweenCompleted {
    /// The [`Entity`] the tween which completed and its animator are attached
    /// to.
    pub entity: Entity,
    /// An opaque value set by the user when activating event raising, used to
    /// identify the particular tween which raised this event. The value is
    /// passed unmodified from a call to [`with_completed_event()`]
    /// or [`set_completed_event()`].
    ///
    /// [`with_completed_event()`]: Tween::with_completed_event
    /// [`set_completed_event()`]: Tween::set_completed_event
    pub user_data: u64,
}

/// Calculate the progress fraction in \[0:1\] of the ratio between two
/// [`Duration`]s.
fn fraction_progress(n: Duration, d: Duration) -> f32 {
    // TODO - Replace with div_duration_f32() once it's stable
    (n.as_secs_f64() / d.as_secs_f64()).fract() as f32
}

#[derive(Debug)]
struct AnimClock {
    elapsed: Duration,
    duration: Duration,
    total_duration: TotalDuration,
    strategy: RepeatStrategy,
}

impl AnimClock {
    fn new(duration: Duration) -> Self {
        Self {
            elapsed: Duration::ZERO,
            duration,
            total_duration: compute_total_duration(duration, RepeatCount::default()),
            strategy: RepeatStrategy::default(),
        }
    }

    fn tick(&mut self, tick: Duration) -> (TweenState, i32) {
        self.set_elapsed(self.elapsed.saturating_add(tick))
    }

    fn times_completed(&self) -> u32 {
        (self.elapsed.as_nanos() / self.duration.as_nanos()) as u32
    }

    fn set_elapsed(&mut self, elapsed: Duration) -> (TweenState, i32) {
        let old_times_completed = self.times_completed();

        self.elapsed = elapsed;

        let state = match self.total_duration {
            TotalDuration::Finite(total_duration) => {
                if self.elapsed >= total_duration {
                    self.elapsed = total_duration;
                    TweenState::Completed
                } else {
                    TweenState::Active
                }
            }
            TotalDuration::Infinite => TweenState::Active,
        };

        (
            state,
            self.times_completed() as i32 - old_times_completed as i32,
        )
    }

    fn elapsed(&self) -> Duration {
        self.elapsed
    }

    fn state(&self) -> TweenState {
        match self.total_duration {
            TotalDuration::Finite(total_duration) => {
                if self.elapsed >= total_duration {
                    TweenState::Completed
                } else {
                    TweenState::Active
                }
            }
            TotalDuration::Infinite => TweenState::Active,
        }
    }

    fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
    }
}

/// Possibly infinite duration of an animation.
///
/// Used to measure the total duration of an animation including any looping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TotalDuration {
    /// The duration is finite, of the given value.
    Finite(Duration),
    /// The duration is infinite.
    Infinite,
}

fn compute_total_duration(duration: Duration, count: RepeatCount) -> TotalDuration {
    match count {
        RepeatCount::Finite(times) => TotalDuration::Finite(duration.saturating_mul(times)),
        RepeatCount::For(duration) => TotalDuration::Finite(duration),
        RepeatCount::Infinite => TotalDuration::Infinite,
    }
}

// TODO - Targetable et al. should be replaced with Mut->Mut from Bevy 0.9
// https://github.com/bevyengine/bevy/pull/6199

/// Trait to workaround the discrepancies of the change detection mechanisms of
/// assets and components.
pub trait Targetable<T> {
    /// Dereference the target, triggering any change detection, and return a
    /// mutable reference.
    fn target_mut(&mut self) -> &mut T;
}

pub struct ComponentTarget<'a, T: Component> {
    target: Mut<'a, T>,
}

impl<'a, T: Component> ComponentTarget<'a, T> {
    pub fn new(target: Mut<'a, T>) -> Self {
        Self { target }
    }
}

impl<'a, T: Component> Targetable<T> for ComponentTarget<'a, T> {
    fn target_mut(&mut self) -> &mut T {
        self.target.deref_mut()
    }
}

#[cfg(feature = "bevy_asset")]
pub struct AssetTarget<'a, T: Asset> {
    assets: ResMut<'a, Assets<T>>,
    pub handle: Handle<T>,
}

#[cfg(feature = "bevy_asset")]
impl<'a, T: Asset> AssetTarget<'a, T> {
    pub fn new(assets: ResMut<'a, Assets<T>>) -> Self {
        Self {
            assets,
            handle: Handle::weak(HandleId::default::<T>()),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.assets.contains(&self.handle)
    }
}

#[cfg(feature = "bevy_asset")]
impl<'a, T: Asset> Targetable<T> for AssetTarget<'a, T> {
    fn target_mut(&mut self) -> &mut T {
        self.assets.get_mut(&self.handle).unwrap()
    }
}

/// An animatable entity, either a single [`Tween`] or a collection of them.
pub trait Tweenable<T>: Send + Sync {
    /// Get the duration of a single iteration of the animation.
    ///
    /// Note that for [`RepeatStrategy::MirroredRepeat`], this is the duration
    /// of a single way, either from start to end or back from end to start.
    /// The total "loop" duration start -> end -> start to reach back the
    /// same state in this case is the double of the returned value.
    fn duration(&self) -> Duration;

    /// Get the total duration of the entire animation, including looping.
    ///
    /// For [`TotalDuration::Finite`], this is the number of repeats times the
    /// duration of a single iteration ([`duration()`]).
    ///
    /// [`duration()`]: Tweenable::duration
    fn total_duration(&self) -> TotalDuration;

    /// Set the current animation playback elapsed time.
    ///
    /// See [`elapsed()`] for details on the meaning. If `elapsed` is greater
    /// than or equal to [`duration()`], then the animation completes.
    ///
    /// Setting the elapsed time seeks the animation to a new position, but does
    /// not apply that change to the underlying component being animated. To
    /// force the change to apply, call [`tick()`] with a `delta` of
    /// `Duration::ZERO`.
    ///
    /// [`elapsed()`]: Tweenable::elapsed
    /// [`duration()`]: Tweenable::duration
    /// [`tick()`]: Tweenable::tick
    fn set_elapsed(&mut self, elapsed: Duration);

    /// Get the current elapsed duration.
    ///
    /// While looping, the exact value returned by [`duration()`] is never
    /// reached, since the tweenable loops over to zero immediately when it
    /// changes direction at either endpoint. Upon completion, the tweenable
    /// always reports the same value as [`duration()`].
    ///
    /// [`duration()`]: Tweenable::duration
    fn elapsed(&self) -> Duration;

    /// Tick the animation, advancing it by the given delta time and mutating
    /// the given target component or asset.
    ///
    /// This returns [`TweenState::Active`] if the tweenable didn't reach its
    /// final state yet (progress < `1.0`), or [`TweenState::Completed`] if
    /// the tweenable completed this tick. Only non-looping tweenables return
    /// a completed state, since looping ones continue forever.
    ///
    /// Calling this method with a duration of [`Duration::ZERO`] is valid, and
    /// updates the target to the current state of the tweenable without
    /// actually modifying the tweenable state. This is useful after certain
    /// operations like [`rewind()`] or [`set_progress()`] whose effect is
    /// otherwise only visible on target on next frame.
    ///
    /// [`rewind()`]: Tweenable::rewind
    /// [`set_progress()`]: Tweenable::set_progress
    fn tick<'a>(
        &mut self,
        delta: Duration,
        target: &'a mut dyn Targetable<T>,
        entity: Entity,
        events: &mut Mut<Events<TweenCompleted>>,
    ) -> TweenState;

    /// Rewind the animation to its starting state.
    ///
    /// Note that the starting state depends on the current direction. For
    /// [`TweeningDirection::Forward`] this is the start point of the lens,
    /// whereas for [`TweeningDirection::Backward`] this is the end one.
    fn rewind(&mut self);

    /// Set the current animation playback progress.
    ///
    /// See [`progress()`] for details on the meaning.
    ///
    /// Setting the progress seeks the animation to a new position, but does not
    /// apply that change to the underlying component being animated. To
    /// force the change to apply, call [`tick()`] with a `delta` of
    /// `Duration::ZERO`.
    ///
    /// [`progress()`]: Tweenable::progress
    /// [`tick()`]: Tweenable::tick
    fn set_progress(&mut self, progress: f32) {
        self.set_elapsed(self.duration().mul_f32(progress.max(0.)));
    }

    /// Get the current progress in \[0:1\] of the animation.
    ///
    /// While looping, the exact value `1.0` is never reached, since the
    /// tweenable loops over to `0.0` immediately when it changes direction at
    /// either endpoint. Upon completion, the tweenable always reports exactly
    /// `1.0`.
    fn progress(&self) -> f32 {
        let elapsed = self.elapsed();
        if let TotalDuration::Finite(total_duration) = self.total_duration() {
            if elapsed >= total_duration {
                return 1.;
            }
        }
        fraction_progress(elapsed, self.duration())
    }

    /// Get the number of times this tweenable completed.
    ///
    /// For looping animations, this returns the number of times a single
    /// playback was completed. In the case of
    /// [`RepeatStrategy::MirroredRepeat`] this corresponds to a playback in
    /// a single direction, so tweening from start to end and back to start
    /// counts as two completed times (one forward, one backward).
    fn times_completed(&self) -> u32 {
        (self.elapsed().as_nanos() / self.duration().as_nanos()) as u32
    }
}

macro_rules! impl_boxed {
    ($tweenable:ty) => {
        impl<T: 'static> From<$tweenable> for BoxedTweenable<T> {
            fn from(t: $tweenable) -> Self {
                Box::new(t)
            }
        }
    };
}

impl_boxed!(Tween<T>);
impl_boxed!(Sequence<T>);
impl_boxed!(Tracks<T>);
impl_boxed!(Delay<T>);

/// Type of a callback invoked when a [`Tween`] or [`Delay`] has completed.
///
/// See [`Tween::set_completed()`] or [`Delay::set_completed()`] for usage.
pub type CompletedCallback<T> = dyn Fn(Entity, &T) + Send + Sync + 'static;

/// Single tweening animation instance.
pub struct Tween<T> {
    ease_function: EaseMethod,
    clock: AnimClock,
    direction: TweeningDirection,
    lens: Box<dyn Lens<T> + Send + Sync + 'static>,
    on_completed: Option<Box<CompletedCallback<Tween<T>>>>,
    event_data: Option<u64>,
}

impl<T: 'static> Tween<T> {
    /// Chain another [`Tweenable`] after this tween, making a [`Sequence`] with
    /// the two.
    ///
    /// # Example
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::math::*;
    /// # use std::time::Duration;
    /// let tween1 = Tween::new(
    ///     EaseFunction::QuadraticInOut,
    ///     Duration::from_secs(1),
    ///     TransformPositionLens {
    ///         start: Vec3::ZERO,
    ///         end: Vec3::new(3.5, 0., 0.),
    ///     },
    /// );
    /// let tween2 = Tween::new(
    ///     EaseFunction::QuadraticInOut,
    ///     Duration::from_secs(1),
    ///     TransformRotationLens {
    ///         start: Quat::IDENTITY,
    ///         end: Quat::from_rotation_x(90.0_f32.to_radians()),
    ///     },
    /// );
    /// let seq = tween1.then(tween2);
    /// ```
    #[must_use]
    pub fn then(self, tween: impl Tweenable<T> + 'static) -> Sequence<T> {
        Sequence::with_capacity(2).then(self).then(tween)
    }
}

impl<T> Tween<T> {
    /// Create a new tween animation.
    ///
    /// # Example
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::math::Vec3;
    /// # use std::time::Duration;
    /// let tween = Tween::new(
    ///     EaseFunction::QuadraticInOut,
    ///     Duration::from_secs(1),
    ///     TransformPositionLens {
    ///         start: Vec3::ZERO,
    ///         end: Vec3::new(3.5, 0., 0.),
    ///     },
    /// );
    /// ```
    #[must_use]
    pub fn new<L>(ease_function: impl Into<EaseMethod>, duration: Duration, lens: L) -> Self
    where
        L: Lens<T> + Send + Sync + 'static,
    {
        Self {
            ease_function: ease_function.into(),
            clock: AnimClock::new(duration),
            direction: TweeningDirection::Forward,
            lens: Box::new(lens),
            on_completed: None,
            event_data: None,
        }
    }

    /// Enable raising a completed event.
    ///
    /// If enabled, the tween will raise a [`TweenCompleted`] event when the
    /// animation completed. This is similar to the [`with_completed()`]
    /// callback, but uses Bevy events instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::{ecs::event::EventReader, math::Vec3};
    /// # use std::time::Duration;
    /// let tween = Tween::new(
    ///     // [...]
    /// #    EaseFunction::QuadraticInOut,
    /// #    Duration::from_secs(1),
    /// #    TransformPositionLens {
    /// #        start: Vec3::ZERO,
    /// #        end: Vec3::new(3.5, 0., 0.),
    /// #    },
    /// )
    /// .with_completed_event(42);
    ///
    /// fn my_system(mut reader: EventReader<TweenCompleted>) {
    ///   for ev in reader.iter() {
    ///     assert_eq!(ev.user_data, 42);
    ///     println!("Entity {:?} raised TweenCompleted!", ev.entity);
    ///   }
    /// }
    /// ```
    ///
    /// [`with_completed()`]: Tween::with_completed
    #[must_use]
    pub fn with_completed_event(mut self, user_data: u64) -> Self {
        self.event_data = Some(user_data);
        self
    }

    /// Set a callback invoked when the delay completes.
    ///
    /// The callback when invoked receives as parameters the [`Entity`] on which
    /// the target and the animator are, as well as a reference to the
    /// current [`Tween`]. This is similar to [`with_completed_event()`], but
    /// with a callback instead.
    ///
    /// Only non-looping tweenables can complete.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::{ecs::event::EventReader, math::Vec3};
    /// # use std::time::Duration;
    /// let tween = Tween::new(
    ///     // [...]
    /// #    EaseFunction::QuadraticInOut,
    /// #    Duration::from_secs(1),
    /// #    TransformPositionLens {
    /// #        start: Vec3::ZERO,
    /// #        end: Vec3::new(3.5, 0., 0.),
    /// #    },
    /// )
    /// .with_completed(|entity, _tween| {
    ///   println!("Tween completed on entity {:?}", entity);
    /// });
    /// ```
    ///
    /// [`with_completed_event()`]: Tween::with_completed_event
    pub fn with_completed<C>(mut self, callback: C) -> Self
    where
        C: Fn(Entity, &Self) + Send + Sync + 'static,
    {
        self.on_completed = Some(Box::new(callback));
        self
    }

    /// Set the playback direction of the tween.
    ///
    /// The playback direction influences the mapping of the progress ratio (in
    /// \[0:1\]) to the actual ratio passed to the lens.
    /// [`TweeningDirection::Forward`] maps the `0` value of progress to the
    /// `0` value of the lens ratio. Conversely, [`TweeningDirection::Backward`]
    /// reverses the mapping, which effectively makes the tween play reversed,
    /// going from end to start.
    ///
    /// Changing the direction doesn't change any target state, nor any progress
    /// of the tween. Only the direction of animation from this moment
    /// potentially changes. To force a target state change, call
    /// [`Tweenable::tick()`] with a zero delta (`Duration::ZERO`).
    pub fn set_direction(&mut self, direction: TweeningDirection) {
        self.direction = direction;
    }

    /// Set the playback direction of the tween.
    ///
    /// See [`Tween::set_direction()`].
    #[must_use]
    pub fn with_direction(mut self, direction: TweeningDirection) -> Self {
        self.direction = direction;
        self
    }

    /// The current animation direction.
    ///
    /// See [`TweeningDirection`] for details.
    #[must_use]
    pub fn direction(&self) -> TweeningDirection {
        self.direction
    }

    /// Set the number of times to repeat the animation.
    #[must_use]
    pub fn with_repeat_count(mut self, count: RepeatCount) -> Self {
        self.clock.total_duration = compute_total_duration(self.clock.duration, count);
        self
    }

    /// Choose how the animation behaves upon a repetition.
    #[must_use]
    pub fn with_repeat_strategy(mut self, strategy: RepeatStrategy) -> Self {
        self.clock.strategy = strategy;
        self
    }

    /// Set a callback invoked when the animation completes.
    ///
    /// The callback when invoked receives as parameters the [`Entity`] on which
    /// the target and the animator are, as well as a reference to the
    /// current [`Tween`].
    ///
    /// Only non-looping tweenables can complete.
    pub fn set_completed<C>(&mut self, callback: C)
    where
        C: Fn(Entity, &Self) + Send + Sync + 'static,
    {
        self.on_completed = Some(Box::new(callback));
    }

    /// Clear the callback invoked when the animation completes.
    ///
    /// See also [`set_completed()`].
    ///
    /// [`set_completed()`]: Tween::set_completed
    pub fn clear_completed(&mut self) {
        self.on_completed = None;
    }

    /// Enable or disable raising a completed event.
    ///
    /// If enabled, the tween will raise a [`TweenCompleted`] event when the
    /// animation completed. This is similar to the [`set_completed()`]
    /// callback, but uses Bevy events instead.
    ///
    /// See [`with_completed_event()`] for details.
    ///
    /// [`set_completed()`]: Tween::set_completed
    /// [`with_completed_event()`]: Tween::with_completed_event
    pub fn set_completed_event(&mut self, user_data: u64) {
        self.event_data = Some(user_data);
    }

    /// Clear the event sent when the animation completes.
    ///
    /// See also [`set_completed_event()`].
    ///
    /// [`set_completed_event()`]: Tween::set_completed_event
    pub fn clear_completed_event(&mut self) {
        self.event_data = None;
    }
}

impl<T> Tweenable<T> for Tween<T> {
    fn duration(&self) -> Duration {
        self.clock.duration
    }

    fn total_duration(&self) -> TotalDuration {
        self.clock.total_duration
    }

    fn set_elapsed(&mut self, elapsed: Duration) {
        self.clock.set_elapsed(elapsed);
    }

    fn elapsed(&self) -> Duration {
        self.clock.elapsed()
    }

    fn tick<'a>(
        &mut self,
        delta: Duration,
        target: &'a mut dyn Targetable<T>,
        entity: Entity,
        events: &mut Mut<Events<TweenCompleted>>,
    ) -> TweenState {
        if self.clock.state() == TweenState::Completed {
            return TweenState::Completed;
        }

        // Tick the animation clock
        let (state, times_completed) = self.clock.tick(delta);
        let (progress, times_completed_for_direction) = match state {
            TweenState::Active => (self.progress(), times_completed),
            TweenState::Completed => (1., times_completed.max(1) - 1), // ignore last
        };
        if self.clock.strategy == RepeatStrategy::MirroredRepeat
            && times_completed_for_direction & 1 != 0
        {
            self.direction = !self.direction;
        }

        // Apply the lens, even if the animation finished, to ensure the state is
        // consistent
        let mut factor = progress;
        if self.direction.is_backward() {
            factor = 1. - factor;
        }
        let factor = self.ease_function.sample(factor);
        let target = target.target_mut();
        self.lens.lerp(target, factor);

        // If completed at least once this frame, notify the user
        if times_completed > 0 {
            if let Some(user_data) = &self.event_data {
                events.send(TweenCompleted {
                    entity,
                    user_data: *user_data,
                });
            }
            if let Some(cb) = &self.on_completed {
                cb(entity, self);
            }
        }

        state
    }

    fn rewind(&mut self) {
        if self.clock.strategy == RepeatStrategy::MirroredRepeat {
            // In mirrored mode, direction alternates each loop. To reset to the original
            // direction on Tween creation, we count the number of completions, ignoring the
            // last one if the Tween is currently in TweenState::Completed because that one
            // freezes all parameters.
            let mut times_completed = self.clock.times_completed();
            if self.clock.state() == TweenState::Completed {
                debug_assert!(times_completed > 0);
                times_completed -= 1;
            }
            if times_completed & 1 != 0 {
                self.direction = !self.direction;
            }
        }
        self.clock.reset();
    }
}

/// A sequence of tweens played back in order one after the other.
pub struct Sequence<T> {
    tweens: Vec<BoxedTweenable<T>>,
    index: usize,
    duration: Duration,
    elapsed: Duration,
}

impl<T> Sequence<T> {
    /// Create a new sequence of tweens.
    ///
    /// This method panics if the input collection is empty.
    #[must_use]
    pub fn new(items: impl IntoIterator<Item = impl Into<BoxedTweenable<T>>>) -> Self {
        let tweens: Vec<_> = items.into_iter().map(Into::into).collect();
        assert!(!tweens.is_empty());
        let duration = tweens
            .iter()
            .map(AsRef::as_ref)
            .map(Tweenable::duration)
            .sum();
        Self {
            tweens,
            index: 0,
            duration,
            elapsed: Duration::ZERO,
        }
    }

    /// Create a new sequence containing a single tween.
    #[must_use]
    pub fn from_single(tween: impl Tweenable<T> + 'static) -> Self {
        let duration = tween.duration();
        let boxed: BoxedTweenable<T> = Box::new(tween);
        Self {
            tweens: vec![boxed],
            index: 0,
            duration,
            elapsed: Duration::ZERO,
        }
    }

    /// Create a new sequence with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tweens: Vec::with_capacity(capacity),
            index: 0,
            duration: Duration::ZERO,
            elapsed: Duration::ZERO,
        }
    }

    /// Append a [`Tweenable`] to this sequence.
    #[must_use]
    pub fn then(mut self, tween: impl Tweenable<T> + 'static) -> Self {
        self.duration += tween.duration();
        self.tweens.push(Box::new(tween));
        self
    }

    /// Index of the current active tween in the sequence.
    #[must_use]
    pub fn index(&self) -> usize {
        self.index.min(self.tweens.len() - 1)
    }

    /// Get the current active tween in the sequence.
    #[must_use]
    pub fn current(&self) -> &dyn Tweenable<T> {
        self.tweens[self.index()].as_ref()
    }
}

impl<T> Tweenable<T> for Sequence<T> {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn total_duration(&self) -> TotalDuration {
        TotalDuration::Finite(self.duration)
    }

    fn set_elapsed(&mut self, elapsed: Duration) {
        // Set the total sequence progress
        self.elapsed = elapsed;

        // Find which tween is active in the sequence
        let mut accum_duration = Duration::ZERO;
        for index in 0..self.tweens.len() {
            let tween = &mut self.tweens[index];
            let tween_duration = tween.duration();
            if elapsed < accum_duration + tween_duration {
                self.index = index;
                let local_duration = elapsed - accum_duration;
                tween.set_elapsed(local_duration);
                // TODO?? set progress of other tweens after that one to 0. ??
                return;
            }
            tween.set_elapsed(tween.duration()); // ?? to prepare for next loop/rewind?
            accum_duration += tween_duration;
        }

        // None found; sequence ended
        self.index = self.tweens.len();
    }

    fn elapsed(&self) -> Duration {
        self.elapsed
    }

    fn tick<'a>(
        &mut self,
        mut delta: Duration,
        target: &'a mut dyn Targetable<T>,
        entity: Entity,
        events: &mut Mut<Events<TweenCompleted>>,
    ) -> TweenState {
        self.elapsed = self.elapsed.saturating_add(delta).min(self.duration);
        while self.index < self.tweens.len() {
            let tween = &mut self.tweens[self.index];
            let tween_remaining = tween.duration() - tween.elapsed();
            if let TweenState::Active = tween.tick(delta, target, entity, events) {
                return TweenState::Active;
            }

            tween.rewind();
            delta -= tween_remaining;
            self.index += 1;
        }

        TweenState::Completed
    }

    fn rewind(&mut self) {
        self.elapsed = Duration::ZERO;
        self.index = 0;
        for tween in &mut self.tweens {
            // or only first?
            tween.rewind();
        }
    }
}

/// A collection of [`Tweenable`] executing in parallel.
pub struct Tracks<T> {
    tracks: Vec<BoxedTweenable<T>>,
    duration: Duration,
    elapsed: Duration,
}

impl<T> Tracks<T> {
    /// Create a new [`Tracks`] from an iterator over a collection of
    /// [`Tweenable`].
    #[must_use]
    pub fn new(items: impl IntoIterator<Item = impl Into<BoxedTweenable<T>>>) -> Self {
        let tracks: Vec<_> = items.into_iter().map(Into::into).collect();
        let duration = tracks
            .iter()
            .map(AsRef::as_ref)
            .map(Tweenable::duration)
            .max()
            .unwrap();
        Self {
            tracks,
            duration,
            elapsed: Duration::ZERO,
        }
    }
}

impl<T> Tweenable<T> for Tracks<T> {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn total_duration(&self) -> TotalDuration {
        TotalDuration::Finite(self.duration)
    }

    fn set_elapsed(&mut self, elapsed: Duration) {
        self.elapsed = elapsed;

        for tweenable in &mut self.tracks {
            tweenable.set_elapsed(elapsed);
        }
    }

    fn elapsed(&self) -> Duration {
        self.elapsed
    }

    fn tick<'a>(
        &mut self,
        delta: Duration,
        target: &'a mut dyn Targetable<T>,
        entity: Entity,
        events: &mut Mut<Events<TweenCompleted>>,
    ) -> TweenState {
        self.elapsed = self.elapsed.saturating_add(delta).min(self.duration);
        let mut any_active = false;
        for tweenable in &mut self.tracks {
            let state = tweenable.tick(delta, target, entity, events);
            any_active = any_active || (state == TweenState::Active);
        }
        if any_active {
            TweenState::Active
        } else {
            TweenState::Completed
        }
    }

    fn rewind(&mut self) {
        self.elapsed = Duration::ZERO;
        for tween in &mut self.tracks {
            tween.rewind();
        }
    }
}

/// A time delay that doesn't animate anything.
///
/// This is generally useful for combining with other tweenables into sequences
/// and tracks, for example to delay the start of a tween in a track relative to
/// another track. The `menu` example (`examples/menu.rs`) uses this technique
/// to delay the animation of its buttons.
pub struct Delay<T> {
    timer: Timer,
    on_completed: Option<Box<CompletedCallback<Delay<T>>>>,
    event_data: Option<u64>,
}

impl<T: 'static> Delay<T> {
    /// Chain another [`Tweenable`] after this tween, making a [`Sequence`] with
    /// the two.
    #[must_use]
    pub fn then(self, tween: impl Tweenable<T> + 'static) -> Sequence<T> {
        Sequence::with_capacity(2).then(self).then(tween)
    }
}

impl<T> Delay<T> {
    /// Create a new [`Delay`] with a given duration.
    ///
    /// # Panics
    ///
    /// Panics if the duration is zero.
    #[must_use]
    pub fn new(duration: Duration) -> Self {
        assert!(!duration.is_zero());
        Self {
            timer: Timer::new(duration, TimerMode::Once),
            on_completed: None,
            event_data: None,
        }
    }

    /// Enable raising a completed event.
    ///
    /// If enabled, the tweenable will raise a [`TweenCompleted`] event when it
    /// completed. This is similar to the [`set_completed()`] callback, but
    /// uses Bevy events instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::{ecs::event::EventReader, math::Vec3, transform::components::Transform};
    /// # use std::time::Duration;
    /// let delay: Delay<Transform> = Delay::new(Duration::from_secs(5))
    ///   .with_completed_event(42);
    ///
    /// fn my_system(mut reader: EventReader<TweenCompleted>) {
    ///   for ev in reader.iter() {
    ///     assert_eq!(ev.user_data, 42);
    ///     println!("Entity {:?} raised TweenCompleted!", ev.entity);
    ///   }
    /// }
    /// ```
    ///
    /// [`set_completed()`]: Delay::set_completed
    #[must_use]
    pub fn with_completed_event(mut self, user_data: u64) -> Self {
        self.event_data = Some(user_data);
        self
    }

    /// Set a callback invoked when the delay completes.
    ///
    /// The callback when invoked receives as parameters the [`Entity`] on which
    /// the target and the animator are, as well as a reference to the
    /// current [`Delay`]. This is similar to [`with_completed_event()`], but
    /// with a callback instead.
    ///
    /// Only non-looping tweenables can complete.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::{ecs::event::EventReader, math::Vec3};
    /// # use std::time::Duration;
    /// let tween = Tween::new(
    ///     // [...]
    /// #    EaseFunction::QuadraticInOut,
    /// #    Duration::from_secs(1),
    /// #    TransformPositionLens {
    /// #        start: Vec3::ZERO,
    /// #        end: Vec3::new(3.5, 0., 0.),
    /// #    },
    /// )
    /// .with_completed(|entity, delay| {
    ///   println!("Delay of {} seconds elapsed on entity {:?}",
    ///     delay.duration().as_secs(), entity);
    /// });
    /// ```
    ///
    /// [`with_completed_event()`]: Tween::with_completed_event
    pub fn with_completed<C>(mut self, callback: C) -> Self
    where
        C: Fn(Entity, &Self) + Send + Sync + 'static,
    {
        self.on_completed = Some(Box::new(callback));
        self
    }

    /// Check if the delay completed.
    pub fn is_completed(&self) -> bool {
        self.timer.finished()
    }

    /// Get the current tweenable state.
    pub fn state(&self) -> TweenState {
        if self.is_completed() {
            TweenState::Completed
        } else {
            TweenState::Active
        }
    }

    /// Set a callback invoked when the animation completes.
    ///
    /// The callback when invoked receives as parameters the [`Entity`] on which
    /// the target and the animator are, as well as a reference to the
    /// current [`Tween`].
    ///
    /// Only non-looping tweenables can complete.
    pub fn set_completed<C>(&mut self, callback: C)
    where
        C: Fn(Entity, &Self) + Send + Sync + 'static,
    {
        self.on_completed = Some(Box::new(callback));
    }

    /// Clear the callback invoked when the animation completes.
    ///
    /// See also [`set_completed()`].
    ///
    /// [`set_completed()`]: Tween::set_completed
    pub fn clear_completed(&mut self) {
        self.on_completed = None;
    }

    /// Enable or disable raising a completed event.
    ///
    /// If enabled, the tween will raise a [`TweenCompleted`] event when the
    /// animation completed. This is similar to the [`set_completed()`]
    /// callback, but uses Bevy events instead.
    ///
    /// See [`with_completed_event()`] for details.
    ///
    /// [`set_completed()`]: Tween::set_completed
    /// [`with_completed_event()`]: Tween::with_completed_event
    pub fn set_completed_event(&mut self, user_data: u64) {
        self.event_data = Some(user_data);
    }

    /// Clear the event sent when the animation completes.
    ///
    /// See also [`set_completed_event()`].
    ///
    /// [`set_completed_event()`]: Tween::set_completed_event
    pub fn clear_completed_event(&mut self) {
        self.event_data = None;
    }
}

impl<T> Tweenable<T> for Delay<T> {
    fn duration(&self) -> Duration {
        self.timer.duration()
    }

    fn total_duration(&self) -> TotalDuration {
        TotalDuration::Finite(self.duration())
    }

    fn set_elapsed(&mut self, elapsed: Duration) {
        // need to reset() to clear finished() unfortunately
        self.timer.reset();
        self.timer.set_elapsed(elapsed);
        // set_elapsed() does not update finished() etc. which we rely on
        self.timer.tick(Duration::ZERO);
    }

    fn elapsed(&self) -> Duration {
        self.timer.elapsed()
    }

    fn tick<'a>(
        &mut self,
        delta: Duration,
        _target: &'a mut dyn Targetable<T>,
        entity: Entity,
        events: &mut Mut<Events<TweenCompleted>>,
    ) -> TweenState {
        let was_completed = self.is_completed();

        self.timer.tick(delta);

        let state = self.state();

        // If completed this frame, notify the user
        if (state == TweenState::Completed) && !was_completed {
            if let Some(user_data) = &self.event_data {
                events.send(TweenCompleted {
                    entity,
                    user_data: *user_data,
                });
            }
            if let Some(cb) = &self.on_completed {
                cb(entity, self);
            }
        }

        state
    }

    fn rewind(&mut self) {
        self.timer.reset();
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use bevy::ecs::{event::Events, system::SystemState};

    use super::*;
    use crate::{lens::*, test_utils::*};

    #[derive(Default, Copy, Clone)]
    struct CallbackMonitor {
        invoke_count: u64,
        last_reported_count: u32,
    }

    /// Utility to create a tween for testing.
    fn make_test_tween() -> Tween<Transform> {
        Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
    }

    /// Utility to create a test environment to tick a tween.
    fn make_test_env() -> (World, Entity) {
        let mut world = World::new();
        world.init_resource::<Events<TweenCompleted>>();
        let entity = world.spawn(Transform::default()).id();
        (world, entity)
    }

    /// Manually tick a test tweenable targeting a component.
    fn manual_tick_component<T: Component>(
        duration: Duration,
        tween: &mut dyn Tweenable<T>,
        world: &mut World,
        entity: Entity,
    ) -> TweenState {
        world.resource_scope(
            |world: &mut World, mut events: Mut<Events<TweenCompleted>>| {
                let transform = world.get_mut::<T>(entity).unwrap();
                let mut target = ComponentTarget::new(transform);
                tween.tick(duration, &mut target, entity, &mut events)
            },
        )
    }

    #[test]
    fn anim_clock_precision() {
        let duration = Duration::from_millis(1);
        let mut clock = AnimClock::new(duration);
        clock.total_duration = TotalDuration::Infinite;

        let test_ticks = [
            Duration::from_micros(123),
            Duration::from_millis(1),
            Duration::from_secs_f32(1. / 24.),
            Duration::from_secs_f32(1. / 30.),
            Duration::from_secs_f32(1. / 60.),
            Duration::from_secs_f32(1. / 120.),
            Duration::from_secs_f32(1. / 144.),
            Duration::from_secs_f32(1. / 240.),
        ];

        let mut times_completed = 0;
        let mut total_duration = Duration::ZERO;
        for i in 0..10_000_000 {
            let tick = test_ticks[i % test_ticks.len()];
            times_completed += clock.tick(tick).1;
            total_duration += tick;
        }

        assert_eq!(
            (total_duration.as_secs_f64() / duration.as_secs_f64()) as i32,
            times_completed
        );
    }

    /// Test ticking of a single tween in isolation.
    #[test]
    fn tween_tick() {
        for tweening_direction in &[TweeningDirection::Forward, TweeningDirection::Backward] {
            for (count, strategy) in &[
                (RepeatCount::Finite(1), RepeatStrategy::default()),
                (RepeatCount::Infinite, RepeatStrategy::Repeat),
                (RepeatCount::Finite(2), RepeatStrategy::Repeat),
                (RepeatCount::Infinite, RepeatStrategy::MirroredRepeat),
                (RepeatCount::Finite(2), RepeatStrategy::MirroredRepeat),
            ] {
                println!(
                    "TweeningType: count={count:?} strategy={strategy:?} dir={tweening_direction:?}",
                );

                // Create a linear tween over 1 second
                let mut tween = make_test_tween()
                    .with_direction(*tweening_direction)
                    .with_repeat_count(*count)
                    .with_repeat_strategy(*strategy);
                assert_eq!(tween.direction(), *tweening_direction);
                assert!(tween.on_completed.is_none());
                assert!(tween.event_data.is_none());

                let (mut world, entity) = make_test_env();
                let mut event_reader_system_state: SystemState<EventReader<TweenCompleted>> =
                    SystemState::new(&mut world);

                // Register callbacks to count started/ended events
                let callback_monitor = Arc::new(Mutex::new(CallbackMonitor::default()));
                let cb_mon_ptr = Arc::clone(&callback_monitor);
                let reference_entity = entity;
                tween.set_completed(move |completed_entity, tween| {
                    assert_eq!(completed_entity, reference_entity);
                    let mut cb_mon = cb_mon_ptr.lock().unwrap();
                    cb_mon.invoke_count += 1;
                    cb_mon.last_reported_count = tween.times_completed();
                });
                assert!(tween.on_completed.is_some());
                assert!(tween.event_data.is_none());
                assert_eq!(callback_monitor.lock().unwrap().invoke_count, 0);

                // Activate event sending
                const USER_DATA: u64 = 54789; // dummy
                tween.set_completed_event(USER_DATA);
                assert!(tween.event_data.is_some());
                assert_eq!(tween.event_data.unwrap(), USER_DATA);

                // Loop over 2.2 seconds, so greater than one ping-pong loop
                let tick_duration = Duration::from_millis(200);
                for i in 1..=11 {
                    // Calculate expected values
                    let (progress, times_completed, mut direction, expected_state, just_completed) =
                        match count {
                            RepeatCount::Finite(1) => {
                                let progress = (i as f32 * 0.2).min(1.0);
                                let times_completed = u32::from(i >= 5);
                                let state = if i < 5 {
                                    TweenState::Active
                                } else {
                                    TweenState::Completed
                                };
                                let just_completed = i == 5;
                                (
                                    progress,
                                    times_completed,
                                    TweeningDirection::Forward,
                                    state,
                                    just_completed,
                                )
                            }
                            RepeatCount::Finite(count) => {
                                let total_progress = i as f32 * 0.2;
                                let progress = if total_progress >= *count as f32 {
                                    1.
                                } else {
                                    total_progress.fract()
                                };
                                if *strategy == RepeatStrategy::Repeat {
                                    let times_completed = i / 5;
                                    let just_completed = i % 5 == 0;
                                    (
                                        progress,
                                        times_completed,
                                        TweeningDirection::Forward,
                                        if i < 10 {
                                            TweenState::Active
                                        } else {
                                            TweenState::Completed
                                        },
                                        just_completed,
                                    )
                                } else {
                                    let i5 = i % 5;
                                    let times_completed = i / 5;
                                    // Once Completed, the direction doesn't change
                                    let direction = if i >= 5 {
                                        TweeningDirection::Backward
                                    } else {
                                        TweeningDirection::Forward
                                    };
                                    let just_completed = i5 == 0;
                                    (
                                        progress,
                                        times_completed,
                                        direction,
                                        if i < 10 {
                                            TweenState::Active
                                        } else {
                                            TweenState::Completed
                                        },
                                        just_completed,
                                    )
                                }
                            }
                            RepeatCount::Infinite => {
                                let progress = (i as f32 * 0.2).fract();
                                if *strategy == RepeatStrategy::Repeat {
                                    let times_completed = i / 5;
                                    let just_completed = i % 5 == 0;
                                    (
                                        progress,
                                        times_completed,
                                        TweeningDirection::Forward,
                                        TweenState::Active,
                                        just_completed,
                                    )
                                } else {
                                    let i5 = i % 5;
                                    let times_completed = i / 5;
                                    let i10 = i % 10;
                                    let direction = if i10 >= 5 {
                                        TweeningDirection::Backward
                                    } else {
                                        TweeningDirection::Forward
                                    };
                                    let just_completed = i5 == 0;
                                    (
                                        progress,
                                        times_completed,
                                        direction,
                                        TweenState::Active,
                                        just_completed,
                                    )
                                }
                            }
                            RepeatCount::For(_) => panic!("Untested"),
                        };
                    let factor = if tweening_direction.is_backward() {
                        direction = !direction;
                        1. - progress
                    } else {
                        progress
                    };
                    let expected_translation = if direction.is_forward() {
                        Vec3::splat(progress)
                    } else {
                        Vec3::splat(1. - progress)
                    };
                    println!(
                        "Expected: progress={} factor={} times_completed={} direction={:?} state={:?} just_completed={} translation={:?}",
                        progress, factor, times_completed, direction, expected_state, just_completed, expected_translation
                    );

                    // Tick the tween
                    let actual_state =
                        manual_tick_component(tick_duration, &mut tween, &mut world, entity);

                    // Propagate events
                    {
                        let mut events = world.resource_mut::<Events<TweenCompleted>>();
                        events.update();
                    }

                    // Check actual values
                    assert_eq!(tween.direction(), direction);
                    assert_eq!(actual_state, expected_state);
                    assert_approx_eq!(tween.progress(), progress);
                    assert_eq!(tween.times_completed(), times_completed);
                    let transform = world.entity(entity).get::<Transform>().unwrap();
                    assert!(transform
                        .translation
                        .abs_diff_eq(expected_translation, 1e-5));
                    assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));
                    let cb_mon = callback_monitor.lock().unwrap();
                    assert_eq!(cb_mon.invoke_count, times_completed as u64);
                    assert_eq!(cb_mon.last_reported_count, times_completed);
                    {
                        let mut event_reader = event_reader_system_state.get_mut(&mut world);
                        let event = event_reader.iter().next();
                        if just_completed {
                            assert!(event.is_some());
                            if let Some(event) = event {
                                assert_eq!(event.entity, entity);
                                assert_eq!(event.user_data, USER_DATA);
                            }
                        } else {
                            assert!(event.is_none());
                        }
                    }
                }

                // Rewind
                tween.rewind();
                assert_eq!(tween.direction(), *tweening_direction); // does not change
                assert_approx_eq!(tween.progress(), 0.);
                assert_eq!(tween.times_completed(), 0);

                // Dummy tick to update target
                let actual_state =
                    manual_tick_component(Duration::ZERO, &mut tween, &mut world, entity);
                assert_eq!(actual_state, TweenState::Active);
                let expected_translation = if tweening_direction.is_backward() {
                    Vec3::ONE
                } else {
                    Vec3::ZERO
                };
                let transform = world.entity(entity).get::<Transform>().unwrap();
                assert!(transform
                    .translation
                    .abs_diff_eq(expected_translation, 1e-5));
                assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));

                // Clear callback
                tween.clear_completed();
                assert!(tween.on_completed.is_none());

                // Clear event sending
                tween.clear_completed_event();
                assert!(tween.event_data.is_none());
            }
        }
    }

    #[test]
    fn tween_dir() {
        let mut tween = make_test_tween();

        // Default
        assert_eq!(tween.direction(), TweeningDirection::Forward);
        assert_approx_eq!(tween.progress(), 0.0);

        // no-op
        tween.set_direction(TweeningDirection::Forward);
        assert_eq!(tween.direction(), TweeningDirection::Forward);
        assert_approx_eq!(tween.progress(), 0.0);

        // Backward
        tween.set_direction(TweeningDirection::Backward);
        assert_eq!(tween.direction(), TweeningDirection::Backward);
        // progress is independent of direction
        assert_approx_eq!(tween.progress(), 0.0);

        // Progress-invariant
        tween.set_direction(TweeningDirection::Forward);
        tween.set_progress(0.3);
        assert_approx_eq!(tween.progress(), 0.3);
        tween.set_direction(TweeningDirection::Backward);
        // progress is independent of direction
        assert_approx_eq!(tween.progress(), 0.3);

        let (mut world, entity) = make_test_env();

        // Progress always increases alongside the current direction
        tween.set_direction(TweeningDirection::Backward);
        assert_approx_eq!(tween.progress(), 0.3);
        manual_tick_component(Duration::from_millis(100), &mut tween, &mut world, entity);
        assert_approx_eq!(tween.progress(), 0.4);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.6), 1e-5));
    }

    #[test]
    fn tween_elapsed() {
        let mut tween = make_test_tween();

        let duration = tween.duration();
        let elapsed = tween.elapsed();

        assert_eq!(elapsed, Duration::ZERO);
        assert_eq!(duration, Duration::from_secs(1));

        for ms in [0, 1, 500, 100, 300, 999, 847, 1000, 900] {
            let elapsed = Duration::from_millis(ms);
            tween.set_elapsed(elapsed);
            assert_eq!(tween.elapsed(), elapsed);

            let progress = (elapsed.as_secs_f64() / duration.as_secs_f64()) as f32;
            assert_approx_eq!(tween.progress(), progress);

            let times_completed = u32::from(ms == 1000);
            assert_eq!(tween.times_completed(), times_completed);
        }
    }

    /// Test ticking a sequence of tweens.
    #[test]
    fn seq_tick() {
        let tween1 = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let tween2 = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(1),
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_x(90_f32.to_radians()),
            },
        );
        let mut seq = tween1.then(tween2);

        let (mut world, entity) = make_test_env();

        for i in 1..=16 {
            let state =
                manual_tick_component(Duration::from_millis(200), &mut seq, &mut world, entity);
            let transform = world.entity(entity).get::<Transform>().unwrap();
            if i < 5 {
                assert_eq!(state, TweenState::Active);
                let r = i as f32 * 0.2;
                assert_eq!(*transform, Transform::from_translation(Vec3::splat(r)));
            } else if i < 10 {
                assert_eq!(state, TweenState::Active);
                let alpha_deg = (18 * (i - 5)) as f32;
                assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(alpha_deg.to_radians()), 1e-5));
            } else {
                assert_eq!(state, TweenState::Completed);
                assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(90_f32.to_radians()), 1e-5));
            }
        }
    }

    /// Test crossing tween boundaries in one tick.
    #[test]
    fn seq_tick_boundaries() {
        let mut seq = Sequence::new((0..3).map(|i| {
            Tween::new(
                EaseMethod::Linear,
                Duration::from_secs(1),
                TransformPositionLens {
                    start: Vec3::splat(i as f32),
                    end: Vec3::splat((i + 1) as f32),
                },
            )
            .with_repeat_count(RepeatCount::Finite(1))
        }));

        let (mut world, entity) = make_test_env();

        // Tick halfway through the first tween, then in one tick:
        // - Finish the first tween
        // - Start and finish the second tween
        // - Start the third tween
        for delta_ms in [500, 2000] {
            manual_tick_component(
                Duration::from_millis(delta_ms),
                &mut seq,
                &mut world,
                entity,
            );
        }
        assert_eq!(seq.index(), 2);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(2.5), 1e-5));
    }

    /// Sequence::new() and various Sequence-specific methods
    #[test]
    fn seq_iter() {
        let mut seq = Sequence::new((1..5).map(|i| {
            Tween::new(
                EaseMethod::Linear,
                Duration::from_millis(200 * i),
                TransformPositionLens {
                    start: Vec3::ZERO,
                    end: Vec3::ONE,
                },
            )
        }));

        let mut progress = 0.;
        for i in 1..5 {
            assert_eq!(seq.index(), i - 1);
            assert_approx_eq!(seq.progress(), progress);
            let duration = Duration::from_millis(200 * i as u64);
            assert_eq!(seq.current().duration(), duration);
            progress += 0.25;
            seq.set_progress(progress);
            assert_eq!(seq.times_completed(), u32::from(i == 4));
        }

        seq.rewind();
        assert_eq!(seq.progress(), 0.);
        assert_eq!(seq.times_completed(), 0);
    }

    /// Sequence::from_single()
    #[test]
    fn seq_from_single() {
        let tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let seq = Sequence::from_single(tween);

        assert_eq!(seq.duration(), Duration::from_secs(1));
    }

    #[test]
    fn seq_elapsed() {
        let mut seq = Sequence::new((1..5).map(|i| {
            Tween::new(
                EaseMethod::Linear,
                Duration::from_millis(200 * i),
                TransformPositionLens {
                    start: Vec3::ZERO,
                    end: Vec3::ONE,
                },
            )
        }));

        let mut elapsed = Duration::ZERO;
        for i in 1..5 {
            assert_eq!(seq.index(), i - 1);
            assert_eq!(seq.elapsed(), elapsed);
            let duration = Duration::from_millis(200 * i as u64);
            assert_eq!(seq.current().duration(), duration);
            elapsed += duration;
            seq.set_elapsed(elapsed);
            assert_eq!(seq.times_completed(), u32::from(i == 4));
        }
    }

    /// Test ticking parallel tracks of tweens.
    #[test]
    fn tracks_tick() {
        let tween1 = Tween::new(
            EaseMethod::Linear,
            Duration::from_millis(1000),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let tween2 = Tween::new(
            EaseMethod::Linear,
            Duration::from_millis(800), // shorter
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_x(90_f32.to_radians()),
            },
        );
        let mut tracks = Tracks::new([tween1, tween2]);
        assert_eq!(tracks.duration(), Duration::from_secs(1)); // max(1., 0.8)

        let (mut world, entity) = make_test_env();

        for i in 1..=6 {
            let state =
                manual_tick_component(Duration::from_millis(200), &mut tracks, &mut world, entity);
            let transform = world.entity(entity).get::<Transform>().unwrap();
            if i < 5 {
                assert_eq!(state, TweenState::Active);
                assert_eq!(tracks.times_completed(), 0);
                let r = i as f32 * 0.2;
                assert_approx_eq!(tracks.progress(), r);
                let alpha_deg = 22.5 * i as f32;
                assert!(transform.translation.abs_diff_eq(Vec3::splat(r), 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(alpha_deg.to_radians()), 1e-5));
            } else {
                assert_eq!(state, TweenState::Completed);
                assert_eq!(tracks.times_completed(), 1);
                assert_approx_eq!(tracks.progress(), 1.);
                assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(90_f32.to_radians()), 1e-5));
            }
        }

        tracks.rewind();
        assert_eq!(tracks.times_completed(), 0);
        assert_approx_eq!(tracks.progress(), 0.);

        tracks.set_progress(0.9);
        assert_approx_eq!(tracks.progress(), 0.9);
        // tick to udpate state (set_progress() does not update state)
        let state = manual_tick_component(Duration::ZERO, &mut tracks, &mut world, entity);
        assert_eq!(state, TweenState::Active);
        assert_eq!(tracks.times_completed(), 0);

        tracks.set_progress(3.2);
        assert_approx_eq!(tracks.progress(), 1.);
        // tick to udpate state (set_progress() does not update state)
        let state = manual_tick_component(Duration::ZERO, &mut tracks, &mut world, entity);
        assert_eq!(state, TweenState::Completed);
        assert_eq!(tracks.times_completed(), 1); // no looping

        tracks.set_progress(-0.5);
        assert_approx_eq!(tracks.progress(), 0.);
        // tick to udpate state (set_progress() does not update state)
        let state = manual_tick_component(Duration::ZERO, &mut tracks, &mut world, entity);
        assert_eq!(state, TweenState::Active);
        assert_eq!(tracks.times_completed(), 0); // no looping
    }

    /// Delay::then()
    #[test]
    fn delay_then() {
        let seq: Sequence<Transform> =
            Delay::new(Duration::from_secs(1)).then(Delay::new(Duration::from_secs(2)));
        assert_eq!(seq.duration(), Duration::from_secs(3));
        assert_eq!(seq.tweens.len(), 2);
        for (i, t) in seq.tweens.iter().enumerate() {
            assert_eq!(t.duration(), Duration::from_secs(i as u64 + 1));
        }
    }

    /// Test ticking a delay.
    #[test]
    fn delay_tick() {
        let duration = Duration::from_secs(1);

        const USER_DATA: u64 = 42;
        let mut delay = Delay::new(duration).with_completed_event(USER_DATA);

        assert!(delay.event_data.is_some());
        assert_eq!(delay.event_data.unwrap(), USER_DATA);

        delay.clear_completed_event();
        assert!(delay.event_data.is_none());

        delay.set_completed_event(USER_DATA);
        assert!(delay.event_data.is_some());
        assert_eq!(delay.event_data.unwrap(), USER_DATA);

        {
            let tweenable: &dyn Tweenable<Transform> = &delay;
            assert_eq!(tweenable.duration(), duration);
            assert_approx_eq!(tweenable.progress(), 0.);
            assert_eq!(tweenable.elapsed(), Duration::ZERO);
        }

        // Dummy world and event writer
        let (mut world, entity) = make_test_env();
        let mut event_reader_system_state: SystemState<EventReader<TweenCompleted>> =
            SystemState::new(&mut world);

        // Register callbacks to count completed events
        let callback_monitor = Arc::new(Mutex::new(CallbackMonitor::default()));
        let cb_mon_ptr = Arc::clone(&callback_monitor);
        let reference_entity = entity;
        assert!(delay.on_completed.is_none());
        delay.set_completed(move |completed_entity, delay| {
            assert_eq!(completed_entity, reference_entity);
            let mut cb_mon = cb_mon_ptr.lock().unwrap();
            cb_mon.invoke_count += 1;
            cb_mon.last_reported_count = delay.times_completed();
        });
        assert!(delay.on_completed.is_some());
        assert_eq!(callback_monitor.lock().unwrap().invoke_count, 0);

        for i in 1..=6 {
            let state = manual_tick_component::<Transform>(
                Duration::from_millis(200),
                &mut delay,
                &mut world,
                entity,
            );

            // Propagate events
            {
                let mut events = world.resource_mut::<Events<TweenCompleted>>();
                events.update();
            }

            // Check state
            {
                assert_eq!(state, delay.state());

                let tweenable: &dyn Tweenable<Transform> = &delay;

                {
                    let mut event_reader = event_reader_system_state.get_mut(&mut world);
                    let event = event_reader.iter().next();
                    if i == 5 {
                        assert!(event.is_some());
                        let event = event.unwrap();
                        assert_eq!(event.entity, entity);
                        assert_eq!(event.user_data, USER_DATA);
                    } else {
                        assert!(event.is_none());
                    }
                }

                let times_completed = if i < 5 {
                    assert_eq!(state, TweenState::Active);
                    assert!(!delay.is_completed());
                    assert_eq!(tweenable.times_completed(), 0);
                    let r = i as f32 * 0.2;
                    assert_approx_eq!(tweenable.progress(), r);
                    0
                } else {
                    assert_eq!(state, TweenState::Completed);
                    assert!(delay.is_completed());
                    assert_eq!(tweenable.times_completed(), 1);
                    assert_approx_eq!(tweenable.progress(), 1.);
                    1
                };

                let cb_mon = callback_monitor.lock().unwrap();
                assert_eq!(cb_mon.invoke_count, times_completed as u64);
                assert_eq!(cb_mon.last_reported_count, times_completed);
            }
        }

        delay.rewind();
        assert_eq!(delay.times_completed(), 0);
        assert_approx_eq!(delay.progress(), 0.);
        let state = manual_tick_component(Duration::ZERO, &mut delay, &mut world, entity);
        assert_eq!(state, TweenState::Active);

        delay.set_progress(0.3);
        assert_eq!(delay.times_completed(), 0);
        assert_approx_eq!(delay.progress(), 0.3);
        delay.set_progress(1.);
        assert_eq!(delay.times_completed(), 1);
        assert_approx_eq!(delay.progress(), 1.);

        // Clear callback
        delay.clear_completed();
        assert!(delay.on_completed.is_none());

        // Clear event sending
        delay.clear_completed_event();
        assert!(delay.event_data.is_none());
    }

    #[test]
    fn delay_elapsed() {
        let mut delay: Delay<f32> = Delay::new(Duration::from_secs(1));
        let duration = delay.duration();
        for ms in [0, 1, 500, 100, 300, 999, 847, 1000, 900] {
            let elapsed = Duration::from_millis(ms);
            delay.set_elapsed(elapsed);
            assert_eq!(delay.elapsed(), elapsed);

            let progress = (elapsed.as_secs_f64() / duration.as_secs_f64()) as f32;
            assert_approx_eq!(delay.progress(), progress);

            let times_completed = u32::from(ms == 1000);
            assert_eq!(delay.times_completed(), times_completed);

            assert_eq!(delay.is_completed(), ms >= 1000);
            assert_eq!(
                delay.state(),
                if ms >= 1000 {
                    TweenState::Completed
                } else {
                    TweenState::Active
                }
            );
        }
    }

    #[test]
    #[should_panic]
    fn delay_zero_duration_panics() {
        let _: Delay<f32> = Delay::new(Duration::ZERO);
    }

    #[test]
    fn tween_repeat() {
        let mut tween = make_test_tween()
            .with_repeat_count(RepeatCount::Finite(5))
            .with_repeat_strategy(RepeatStrategy::Repeat);

        assert_approx_eq!(tween.progress(), 0.);

        let (mut world, entity) = make_test_env();

        // 10%
        let state =
            manual_tick_component(Duration::from_millis(100), &mut tween, &mut world, entity);
        assert_eq!(TweenState::Active, state);
        assert_eq!(0, tween.times_completed());
        assert_approx_eq!(tween.progress(), 0.1);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.1), 1e-5));

        // 130%
        let state =
            manual_tick_component(Duration::from_millis(1200), &mut tween, &mut world, entity);
        assert_eq!(TweenState::Active, state);
        assert_eq!(1, tween.times_completed());
        assert_approx_eq!(tween.progress(), 0.3);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.3), 1e-5));

        // 480%
        let state =
            manual_tick_component(Duration::from_millis(3500), &mut tween, &mut world, entity);
        assert_eq!(TweenState::Active, state);
        assert_eq!(4, tween.times_completed());
        assert_approx_eq!(tween.progress(), 0.8);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.8), 1e-5));

        // 500% - done
        let state =
            manual_tick_component(Duration::from_millis(200), &mut tween, &mut world, entity);
        assert_eq!(TweenState::Completed, state);
        assert_eq!(5, tween.times_completed());
        assert_approx_eq!(tween.progress(), 1.0);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));
    }

    #[test]
    fn tween_mirrored_rewind() {
        let mut tween = make_test_tween()
            .with_repeat_count(RepeatCount::Finite(4))
            .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        assert_approx_eq!(tween.progress(), 0.);

        let (mut world, entity) = make_test_env();

        // 10%
        let state =
            manual_tick_component(Duration::from_millis(100), &mut tween, &mut world, entity);
        assert_eq!(TweenState::Active, state);
        assert_eq!(TweeningDirection::Forward, tween.direction());
        assert_eq!(0, tween.times_completed());
        assert_approx_eq!(tween.progress(), 0.1);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.1), 1e-5));

        // rewind
        tween.rewind();
        assert_eq!(TweeningDirection::Forward, tween.direction());
        assert_eq!(0, tween.times_completed());
        assert_approx_eq!(tween.progress(), 0.);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.1), 1e-5)); // no-op, rewind doesn't apply Lens

        // 120% - mirror
        let state =
            manual_tick_component(Duration::from_millis(1200), &mut tween, &mut world, entity);
        assert_eq!(TweeningDirection::Backward, tween.direction());
        assert_eq!(TweenState::Active, state);
        assert_eq!(1, tween.times_completed());
        assert_approx_eq!(tween.progress(), 0.2);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.8), 1e-5));

        // rewind
        tween.rewind();
        assert_eq!(TweeningDirection::Forward, tween.direction()); // restored
        assert_eq!(0, tween.times_completed());
        assert_approx_eq!(tween.progress(), 0.);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.8), 1e-5)); // no-op, rewind doesn't apply Lens

        // 400% - done mirror (because Completed freezes the state)
        let state =
            manual_tick_component(Duration::from_millis(4000), &mut tween, &mut world, entity);
        assert_eq!(TweenState::Completed, state);
        assert_eq!(TweeningDirection::Backward, tween.direction()); // frozen from last loop
        assert_eq!(4, tween.times_completed());
        assert_approx_eq!(tween.progress(), 1.); // Completed
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));

        // rewind
        tween.rewind();
        assert_eq!(TweeningDirection::Forward, tween.direction()); // restored
        assert_eq!(0, tween.times_completed());
        assert_approx_eq!(tween.progress(), 0.);
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5)); // no-op, rewind doesn't apply Lens
    }
}
