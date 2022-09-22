use std::time::Duration;

use bevy::prelude::*;

use crate::{EaseMethod, Lens, RepeatCount, RepeatStrategy, TweeningDirection};

/// The dynamic tweenable type.
///
/// When creating lists of tweenables, you will need to box them to create a
/// homogeneous array like so:
/// ```no_run
/// # use bevy::prelude::Transform;
/// # use bevy_tweening::{BoxedTweenable, Delay, Sequence, Tween};
/// #
/// # let delay: Delay = unimplemented!();
/// # let tween: Tween<Transform> = unimplemented!();
///
/// Sequence::new([Box::new(delay) as BoxedTweenable<Transform>, tween.into()]);
/// ```
///
/// When using your own [`Tweenable`] types, APIs will be easier to use if you
/// implement [`From`]:
/// ```no_run
/// # use std::time::Duration;
/// # use bevy::prelude::{Entity, EventWriter, Transform};
/// # use bevy_tweening::{BoxedTweenable, Sequence, Tweenable, TweenCompleted, TweenState};
/// #
/// # struct MyTweenable;
/// # impl Tweenable<Transform> for MyTweenable {
/// #     fn duration(&self) -> Duration  { unimplemented!() }
/// #     fn set_progress(&mut self, progress: f32)  { unimplemented!() }
/// #     fn progress(&self) -> f32  { unimplemented!() }
/// #     fn tick(&mut self, delta: Duration, target: &mut Transform, entity: Entity, event_writer: &mut EventWriter<TweenCompleted>) -> TweenState  { unimplemented!() }
/// #     fn times_completed(&self) -> u32  { unimplemented!() }
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
pub type BoxedTweenable<T> = Box<dyn Tweenable<T> + Send + Sync + 'static>;

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
/// If the tick delta is high enough that multiple completions have occurred in
/// its interval, only one event will be sent rather one event per completion
/// that theoretically occurred.
///
/// # Note
///
/// The semantic is slightly different from [`TweenState::Completed`], which
/// indicates that the tweenable has finished ticking and does not need to be
/// updated anymore, a state which is never reached for looping animation. Here
/// the [`TweenCompleted`] event instead marks the end of a single loop
/// iteration.
#[derive(Copy, Clone, Debug)]
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

#[derive(Debug)]
struct AnimClock {
    elapsed: Duration,
    duration: Duration,
    times_completed: u32,
    total_duration: TotalDuration,
    strategy: RepeatStrategy,
}

impl AnimClock {
    fn new(duration: Duration) -> Self {
        Self {
            elapsed: Duration::ZERO,
            duration,
            total_duration: compute_total_duration(duration, RepeatCount::default()),
            times_completed: 0,
            strategy: RepeatStrategy::default(),
        }
    }

    fn record_completions(&mut self, times_completed: u32) {
        self.times_completed = self.times_completed.saturating_add(times_completed);
    }

    fn tick(&mut self, tick: Duration) -> u32 {
        let duration = self.duration.as_nanos();

        let before = self.elapsed.as_nanos() / duration;
        self.elapsed = self.elapsed.saturating_add(tick);
        if let TotalDuration::Finite(duration) = self.total_duration {
            self.elapsed = self.elapsed.min(duration);
        }
        (self.elapsed.as_nanos() / duration - before) as u32
    }

    fn set_progress(&mut self, progress: f32) {
        self.elapsed = self.duration.mul_f32(progress.max(0.));
    }

    fn progress(&self) -> f32 {
        self.elapsed.as_secs_f32() / self.duration.as_secs_f32()
    }

    fn state(&self) -> TweenState {
        match self.total_duration {
            TotalDuration::Finite(duration) => {
                if self.elapsed >= duration {
                    TweenState::Completed
                } else {
                    TweenState::Active
                }
            }
            TotalDuration::Infinite => TweenState::Active,
        }
    }

    fn reset(&mut self) {
        self.times_completed = 0;
        self.elapsed = Duration::ZERO;
    }
}

#[derive(Debug)]
enum TotalDuration {
    Finite(Duration),
    Infinite,
}

fn compute_total_duration(duration: Duration, count: RepeatCount) -> TotalDuration {
    match count {
        RepeatCount::Finite(times) => TotalDuration::Finite(duration.saturating_mul(times)),
        RepeatCount::For(duration) => TotalDuration::Finite(duration),
        RepeatCount::Infinite => TotalDuration::Infinite,
    }
}

/// An animatable entity, either a single [`Tween`] or a collection of them.
pub trait Tweenable<T>: Send + Sync {
    /// Get the total duration of the animation.
    ///
    /// This is always the duration of a single iteration, even when looping.
    ///
    /// Note that for [`RepeatStrategy::MirroredRepeat`], this is the duration
    /// of a single way, either from start to end or back from end to start.
    /// The total "loop" duration start -> end -> start to reach back the
    /// same state in this case is the double of the returned value.
    fn duration(&self) -> Duration;

    /// Set the current animation playback progress.
    ///
    /// See [`progress()`] for details on the meaning.
    ///
    /// [`progress()`]: Tweenable::progress
    fn set_progress(&mut self, progress: f32);

    /// Get the current progress in \[0:1\] of the animation.
    ///
    /// While looping, the exact value `1.0` is never reached, since the
    /// tweenable loops over to `0.0` immediately when it changes direction at
    /// either endpoint. Upon completion, the tweenable always reports exactly
    /// `1.0`.
    fn progress(&self) -> f32;

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
    fn tick(
        &mut self,
        delta: Duration,
        target: &mut T,
        entity: Entity,
        event_writer: &mut EventWriter<TweenCompleted>,
    ) -> TweenState;

    /// Get the number of times this tweenable completed.
    ///
    /// For looping animations, this returns the number of times a single
    /// playback was completed. In the case of
    /// [`RepeatStrategy::MirroredRepeat`] this corresponds to a playback in
    /// a single direction, so tweening from start to end and back to start
    /// counts as two completed times (one forward, one backward).
    fn times_completed(&self) -> u32;

    /// Rewind the animation to its starting state.
    ///
    /// Note that the starting state depends on the current direction. For
    /// [`TweeningDirection::Forward`] this is the start point of the lens,
    /// whereas for [`TweeningDirection::Backward`] this is the end one.
    fn rewind(&mut self);
}

impl<T> From<Delay> for BoxedTweenable<T> {
    fn from(d: Delay) -> Self {
        Box::new(d)
    }
}

impl<T: 'static> From<Sequence<T>> for BoxedTweenable<T> {
    fn from(s: Sequence<T>) -> Self {
        Box::new(s)
    }
}

impl<T: 'static> From<Tracks<T>> for BoxedTweenable<T> {
    fn from(t: Tracks<T>) -> Self {
        Box::new(t)
    }
}

impl<T: 'static> From<Tween<T>> for BoxedTweenable<T> {
    fn from(t: Tween<T>) -> Self {
        Box::new(t)
    }
}

/// Type of a callback invoked when a [`Tween`] has completed.
///
/// See [`Tween::set_completed()`] for usage.
pub type CompletedCallback<T> = dyn Fn(Entity, &Tween<T>) + Send + Sync + 'static;

/// Single tweening animation instance.
pub struct Tween<T> {
    ease_method: EaseMethod,
    clock: AnimClock,
    direction: TweeningDirection,
    lens: Box<dyn Lens<T> + Send + Sync + 'static>,
    on_completed: Option<Box<CompletedCallback<T>>>,
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
    ///     Duration::from_secs_f32(1.0),
    ///     TransformPositionLens {
    ///         start: Vec3::ZERO,
    ///         end: Vec3::new(3.5, 0., 0.),
    ///     },
    /// );
    /// let tween2 = Tween::new(
    ///     EaseFunction::QuadraticInOut,
    ///     Duration::from_secs_f32(1.0),
    ///     TransformRotationLens {
    ///         start: Quat::IDENTITY,
    ///         end: Quat::from_rotation_x(90.0_f32.to_radians()),
    ///     },
    /// );
    /// let seq = tween1.then(tween2);
    /// ```
    #[must_use]
    pub fn then(self, tween: impl Tweenable<T> + Send + Sync + 'static) -> Sequence<T> {
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
    ///     Duration::from_secs_f32(1.0),
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
            ease_method: ease_function.into(),
            clock: AnimClock::new(duration),
            direction: TweeningDirection::Forward,
            lens: Box::new(lens),
            on_completed: None,
            event_data: None,
        }
    }

    /// Enable or disable raising a completed event.
    ///
    /// If enabled, the tween will raise a [`TweenCompleted`] event when the
    /// animation completed. This is similar to the [`set_completed()`]
    /// callback, but uses Bevy events instead.
    ///
    /// # Example
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::{ecs::event::EventReader, math::Vec3};
    /// # use std::time::Duration;
    /// let tween = Tween::new(
    ///     // [...]
    /// #    EaseFunction::QuadraticInOut,
    /// #    Duration::from_secs_f32(1.0),
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
    /// [`set_completed()`]: Tween::set_completed
    #[must_use]
    pub fn with_completed_event(mut self, user_data: u64) -> Self {
        self.event_data = Some(user_data);
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
    pub fn clear_completed_event(&mut self) {
        self.event_data = None;
    }
}

impl<T> Tweenable<T> for Tween<T> {
    fn duration(&self) -> Duration {
        self.clock.duration
    }

    fn set_progress(&mut self, progress: f32) {
        self.clock.set_progress(progress);
    }

    fn progress(&self) -> f32 {
        self.clock.progress()
    }

    fn tick(
        &mut self,
        delta: Duration,
        target: &mut T,
        entity: Entity,
        event_writer: &mut EventWriter<TweenCompleted>,
    ) -> TweenState {
        if self.clock.state() == TweenState::Completed {
            return TweenState::Completed;
        }

        // Tick the animation clock
        let times_completed = self.clock.tick(delta);
        self.clock.record_completions(times_completed);
        if self.clock.strategy == RepeatStrategy::MirroredRepeat && times_completed & 1 != 0 {
            self.direction = !self.direction;
        }
        let progress = self.progress();

        // Apply the lens, even if the animation finished, to ensure the state is
        // consistent
        let mut factor = progress;
        if self.direction.is_backward() {
            factor = 1. - factor;
        }
        let factor = self.ease_method.sample(factor);
        self.lens.lerp(target, factor);

        // If completed at least once this frame, notify the user
        if times_completed > 0 {
            if let Some(user_data) = &self.event_data {
                event_writer.send(TweenCompleted {
                    entity,
                    user_data: *user_data,
                });
            }
            if let Some(cb) = &self.on_completed {
                cb(entity, self);
            }
        }

        self.clock.state()
    }

    fn times_completed(&self) -> u32 {
        self.clock.times_completed
    }

    fn rewind(&mut self) {
        self.clock.reset();
    }
}

/// A sequence of tweens played back in order one after the other.
pub struct Sequence<T> {
    tweens: Vec<BoxedTweenable<T>>,
    index: usize,
    duration: Duration,
    time: Duration,
    times_completed: u32,
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
            time: Duration::ZERO,
            times_completed: 0,
        }
    }

    /// Create a new sequence containing a single tween.
    #[must_use]
    pub fn from_single(tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        let duration = tween.duration();
        let boxed: BoxedTweenable<T> = Box::new(tween);
        Self {
            tweens: vec![boxed],
            index: 0,
            duration,
            time: Duration::ZERO,
            times_completed: 0,
        }
    }

    /// Create a new sequence with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tweens: Vec::with_capacity(capacity),
            index: 0,
            duration: Duration::ZERO,
            time: Duration::ZERO,
            times_completed: 0,
        }
    }

    /// Append a [`Tweenable`] to this sequence.
    #[must_use]
    pub fn then(mut self, tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
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

    fn set_progress(&mut self, progress: f32) {
        self.times_completed = if progress >= 1. { 1 } else { 0 };
        let progress = progress.clamp(0., 1.); // not looping
                                               // Set the total sequence progress
        let total_elapsed_secs = self.duration().as_secs_f64() * progress as f64;
        self.time = Duration::from_secs_f64(total_elapsed_secs);

        // Find which tween is active in the sequence
        let mut accum_duration = 0.;
        for index in 0..self.tweens.len() {
            let tween = &mut self.tweens[index];
            let tween_duration = tween.duration().as_secs_f64();
            if total_elapsed_secs < accum_duration + tween_duration {
                self.index = index;
                let local_duration = total_elapsed_secs - accum_duration;
                tween.set_progress((local_duration / tween_duration) as f32);
                // TODO?? set progress of other tweens after that one to 0. ??
                return;
            }
            tween.set_progress(1.); // ?? to prepare for next loop/rewind?
            accum_duration += tween_duration;
        }

        // None found; sequence ended
        self.index = self.tweens.len();
    }

    fn progress(&self) -> f32 {
        self.time.as_secs_f32() / self.duration.as_secs_f32()
    }

    fn tick(
        &mut self,
        mut delta: Duration,
        target: &mut T,
        entity: Entity,
        event_writer: &mut EventWriter<TweenCompleted>,
    ) -> TweenState {
        self.time = (self.time + delta).min(self.duration);
        while self.index < self.tweens.len() {
            let tween = &mut self.tweens[self.index];
            let tween_remaining = tween.duration().mul_f32(1.0 - tween.progress());
            if let TweenState::Active = tween.tick(delta, target, entity, event_writer) {
                return TweenState::Active;
            }

            tween.rewind();
            delta -= tween_remaining;
            self.index += 1;
        }

        self.times_completed = 1;
        TweenState::Completed
    }

    fn times_completed(&self) -> u32 {
        self.times_completed
    }

    fn rewind(&mut self) {
        self.time = Duration::ZERO;
        self.index = 0;
        self.times_completed = 0;
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
    time: Duration,
    times_completed: u32,
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
            time: Duration::ZERO,
            times_completed: 0,
        }
    }
}

impl<T> Tweenable<T> for Tracks<T> {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn set_progress(&mut self, progress: f32) {
        self.times_completed = if progress >= 1. { 1 } else { 0 }; // not looping
        let progress = progress.clamp(0., 1.); // not looping
        let time_secs = self.duration.as_secs_f64() * progress as f64;
        self.time = Duration::from_secs_f64(time_secs);
        for tweenable in &mut self.tracks {
            let progress = time_secs / tweenable.duration().as_secs_f64();
            tweenable.set_progress(progress as f32);
        }
    }

    fn progress(&self) -> f32 {
        self.time.as_secs_f32() / self.duration.as_secs_f32()
    }

    fn tick(
        &mut self,
        delta: Duration,
        target: &mut T,
        entity: Entity,
        event_writer: &mut EventWriter<TweenCompleted>,
    ) -> TweenState {
        self.time = (self.time + delta).min(self.duration);
        let mut any_active = false;
        for tweenable in &mut self.tracks {
            let state = tweenable.tick(delta, target, entity, event_writer);
            any_active = any_active || (state == TweenState::Active);
        }
        if any_active {
            TweenState::Active
        } else {
            self.times_completed = 1;
            TweenState::Completed
        }
    }

    fn times_completed(&self) -> u32 {
        self.times_completed
    }

    fn rewind(&mut self) {
        self.time = Duration::ZERO;
        self.times_completed = 0;
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
pub struct Delay {
    timer: Timer,
}

impl Delay {
    /// Create a new [`Delay`] with a given duration.
    #[must_use]
    pub fn new(duration: Duration) -> Self {
        Self {
            timer: Timer::new(duration, false),
        }
    }

    /// Chain another [`Tweenable`] after this tween, making a sequence with the
    /// two.
    #[must_use]
    pub fn then<T>(self, tween: impl Tweenable<T> + Send + Sync + 'static) -> Sequence<T> {
        Sequence::with_capacity(2).then(self).then(tween)
    }
}

impl<T> Tweenable<T> for Delay {
    fn duration(&self) -> Duration {
        self.timer.duration()
    }

    fn set_progress(&mut self, progress: f32) {
        // need to reset() to clear finished() unfortunately
        self.timer.reset();
        self.timer.set_elapsed(Duration::from_secs_f64(
            self.timer.duration().as_secs_f64() * progress as f64,
        ));
        // set_elapsed() does not update finished() etc. which we rely on
        self.timer.tick(Duration::ZERO);
    }

    fn progress(&self) -> f32 {
        self.timer.percent()
    }

    fn tick(
        &mut self,
        delta: Duration,
        _target: &mut T,
        _entity: Entity,
        _event_writer: &mut EventWriter<TweenCompleted>,
    ) -> TweenState {
        self.timer.tick(delta);
        if self.timer.finished() {
            TweenState::Completed
        } else {
            TweenState::Active
        }
    }

    fn times_completed(&self) -> u32 {
        if self.timer.finished() {
            1
        } else {
            0
        }
    }

    fn rewind(&mut self) {
        self.timer.reset();
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::Write,
        iter::{once, repeat},
        sync::{Arc, Mutex},
        time::Duration,
    };

    use bevy::ecs::{event::Events, system::SystemState};
    use goldenfile::Mint;
    use rstest::rstest;

    use crate::lens::*;

    use super::*;

    /// Utility to compare floating-point values with a tolerance.
    fn abs_diff_eq(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() < tol
    }

    fn create_event_reader_writer<'w, 's>() -> (
        World,
        SystemState<EventReader<'w, 's, TweenCompleted>>,
        SystemState<EventWriter<'w, 's, TweenCompleted>>,
    ) {
        let mut world = World::new();
        world.init_resource::<Events<TweenCompleted>>();

        let event_reader_system_state: SystemState<EventReader<TweenCompleted>> =
            SystemState::new(&mut world);
        let event_writer_system_state: SystemState<EventWriter<TweenCompleted>> =
            SystemState::new(&mut world);

        (world, event_reader_system_state, event_writer_system_state)
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
            times_completed += clock.tick(tick);
            total_duration += tick;
        }

        assert_eq!(
            (total_duration.as_nanos() / duration.as_nanos()) as u32,
            times_completed
        );
    }

    #[rstest]
    fn tween_rewind(#[values(1. / 3., 0.5, 1.)] starting_progress: f32) {
        let mut tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let mut transform = Transform::default();

        let (mut world, _, mut event_writer) = create_event_reader_writer();

        tween.set_progress(starting_progress);
        // Apply progress change to the transform
        tween.tick(
            Duration::ZERO,
            &mut transform,
            Entity::from_raw(0),
            &mut event_writer.get_mut(&mut world),
        );
        // TODO(https://github.com/djeedai/bevy_tweening/issues/43) decide if this is acceptable behavior.
        if starting_progress != 1. {
            assert!(transform
                .translation
                .abs_diff_eq(Vec3::splat(starting_progress), 1e-5));
        }

        // Rewind
        tween.rewind();
        assert!(abs_diff_eq(tween.progress(), 0., 1e-5));
        assert_eq!(
            tween.tick(
                Duration::ZERO,
                &mut transform,
                Entity::from_raw(0),
                &mut event_writer.get_mut(&mut world),
            ),
            TweenState::Active
        );
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));
        assert_eq!(tween.times_completed(), 0);
    }

    fn write_tween(
        mut tween: Tween<Transform>,
        deltas: impl IntoIterator<Item = Duration>,
        output: impl Write + Send + Sync + 'static,
    ) {
        let output = Arc::new(Mutex::new(output));

        {
            let output = output.clone();
            tween.set_completed_event(69420);
            tween.set_completed(move |entity, _| {
                writeln!(output.lock().unwrap(), "Completion received for {entity:?}").unwrap();
            });
        }

        let (mut world, mut event_reader, mut event_writer) = create_event_reader_writer();

        writeln!(
            output.lock().unwrap(),
            "Initial tween state:\nLens={:#?}\nClock={:#?}\nDirection={:?}",
            tween.lens,
            tween.clock,
            tween.direction
        )
        .unwrap();

        let mut transform = Transform::default();
        let mut elapsed = Duration::ZERO;
        for delta in deltas {
            writeln!(output.lock().unwrap(), "\nTick by {delta:?}:").unwrap();

            let state = tween.tick(
                delta,
                &mut transform,
                Entity::from_raw(42),
                &mut event_writer.get_mut(&mut world),
            );

            elapsed = elapsed.saturating_add(delta); // TODO replace with elapsed API
            let mut output = output.lock().unwrap();
            writeln!(output, "{elapsed:?}/{:?} elapsed", tween.duration()).unwrap();
            writeln!(output, "Direction: {:?}", tween.direction()).unwrap();
            writeln!(output, "State: {:?}", state).unwrap();
            writeln!(output, "Progress: {:?}", tween.progress()).unwrap();
            writeln!(output, "Total completions: {:?}", tween.times_completed()).unwrap();
            writeln!(output, "Transform: {:?}", transform).unwrap();

            {
                let mut events = world.get_resource_mut::<Events<TweenCompleted>>().unwrap();
                events.update();
                let mut event_reader = event_reader.get_mut(&mut world);

                for event in event_reader.iter() {
                    writeln!(output, "Event received: {event:?}").unwrap();
                }
            }
        }
    }

    #[rstest]
    fn tween_tick_with_defaults_and_directions(
        #[values(TweeningDirection::Forward, TweeningDirection::Backward)]
        direction: TweeningDirection,
    ) {
        let tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_direction(direction);

        let mut mint = Mint::new("testdata");
        let goldenfile = mint
            .new_goldenfile(format!(
                "tween_tick_with_defaults_going_{direction:?}.stdout"
            ))
            .unwrap();
        write_tween(
            tween,
            once(Duration::ZERO).chain(repeat(Duration::from_millis(200)).take(6)),
            goldenfile,
        );
    }

    #[test]
    fn tween_tick_loop_finite() {
        let tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_direction(TweeningDirection::Forward)
        .with_repeat_count(RepeatCount::Finite(3))
        .with_repeat_strategy(RepeatStrategy::Repeat);

        let mut mint = Mint::new("testdata");
        let goldenfile = mint
            .new_goldenfile("tween_tick_loop_finite.stdout")
            .unwrap();
        write_tween(
            tween,
            once(Duration::ZERO).chain(repeat(Duration::from_secs(1) / 3).take(10)),
            goldenfile,
        );
    }

    #[test]
    fn tween_tick_loop_infinite_large_jump() {
        let duration = Duration::from_secs(4) / 3;
        let tween = Tween::new(
            EaseMethod::Linear,
            duration,
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_direction(TweeningDirection::Forward)
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::Repeat);

        let mut mint = Mint::new("testdata");
        let goldenfile = mint
            .new_goldenfile("tween_tick_loop_infinite_large_jump.stdout")
            .unwrap();
        write_tween(
            tween,
            [duration * 10, Duration::MAX, Duration::MAX],
            goldenfile,
        );
    }

    #[test]
    fn tween_tick_loop_finite_large_jump() {
        let duration = Duration::from_secs(4) / 3;
        let tween = Tween::new(
            EaseMethod::Linear,
            duration,
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_direction(TweeningDirection::Forward)
        .with_repeat_count(RepeatCount::Finite(100))
        .with_repeat_strategy(RepeatStrategy::Repeat);

        let mut mint = Mint::new("testdata");
        let goldenfile = mint
            .new_goldenfile("tween_tick_loop_finite_large_jump.stdout")
            .unwrap();
        write_tween(
            tween,
            [duration * 10, Duration::MAX, Duration::MAX],
            goldenfile,
        );
    }

    #[test]
    fn tween_tick_loop_ping_pong() {
        let tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(1) / 3,
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_direction(TweeningDirection::Forward)
        .with_repeat_count(RepeatCount::Infinite)
        .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        let mut mint = Mint::new("testdata");
        let goldenfile = mint
            .new_goldenfile("tween_tick_loop_ping_pong.stdout")
            .unwrap();
        write_tween(
            tween,
            repeat(Duration::from_millis(200)).take(4),
            goldenfile,
        );
    }

    #[test]
    fn tween_tick_loop_partial_completion() {
        let tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs(2) / 3,
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_direction(TweeningDirection::Forward)
        .with_repeat_count(RepeatCount::For(Duration::from_secs_f64(1.42)))
        .with_repeat_strategy(RepeatStrategy::Repeat);

        let mut mint = Mint::new("testdata");
        let goldenfile = mint
            .new_goldenfile("tween_tick_loop_partial_completion.stdout")
            .unwrap();
        write_tween(
            tween,
            repeat(Duration::from_millis(400)).take(4),
            goldenfile,
        );
    }

    #[test]
    fn tween_direction() {
        let mut tween = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(1.0),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );

        // Default
        assert_eq!(tween.direction(), TweeningDirection::Forward);
        assert!(abs_diff_eq(tween.progress(), 0.0, 1e-5));

        // no-op
        tween.set_direction(TweeningDirection::Forward);
        assert_eq!(tween.direction(), TweeningDirection::Forward);
        assert!(abs_diff_eq(tween.progress(), 0.0, 1e-5));

        // Backward
        tween.set_direction(TweeningDirection::Backward);
        assert_eq!(tween.direction(), TweeningDirection::Backward);
        // progress is independent of direction
        assert!(abs_diff_eq(tween.progress(), 0.0, 1e-5));

        // Progress-invariant
        tween.set_direction(TweeningDirection::Forward);
        tween.set_progress(0.3);
        assert!(abs_diff_eq(tween.progress(), 0.3, 1e-5));
        tween.set_direction(TweeningDirection::Backward);
        // progress is independent of direction
        assert!(abs_diff_eq(tween.progress(), 0.3, 1e-5));

        let (mut world, _, mut event_writer) = create_event_reader_writer();

        // Progress always increases alongside the current direction
        let dummy_entity = Entity::from_raw(0);
        let mut transform = Transform::default();
        tween.set_direction(TweeningDirection::Backward);
        assert!(abs_diff_eq(tween.progress(), 0.3, 1e-5));
        tween.tick(
            Duration::from_secs_f32(0.1),
            &mut transform,
            dummy_entity,
            &mut event_writer.get_mut(&mut world),
        );
        assert!(abs_diff_eq(tween.progress(), 0.4, 1e-5));
        assert!(transform.translation.abs_diff_eq(Vec3::splat(0.6), 1e-5));
    }

    /// Test ticking a sequence of tweens.
    #[test]
    fn seq_tick() {
        let tween1 = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(1.0),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let tween2 = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(1.0),
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_x(90_f32.to_radians()),
            },
        );
        let mut seq = tween1.then(tween2);
        let mut transform = Transform::default();

        // Dummy world and event writer
        let mut world = World::new();
        world.insert_resource(Events::<TweenCompleted>::default());
        let mut system_state: SystemState<EventWriter<TweenCompleted>> =
            SystemState::new(&mut world);
        let mut event_writer = system_state.get_mut(&mut world);

        for i in 1..=16 {
            let state = seq.tick(
                Duration::from_secs_f32(0.2),
                &mut transform,
                Entity::from_raw(0),
                &mut event_writer,
            );
            if i < 5 {
                assert_eq!(state, TweenState::Active);
                let r = i as f32 * 0.2;
                assert_eq!(transform, Transform::from_translation(Vec3::splat(r)));
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
        let mut transform = Transform::default();

        // Dummy world and event writer
        let mut world = World::new();
        world.insert_resource(Events::<TweenCompleted>::default());
        let mut system_state: SystemState<EventWriter<TweenCompleted>> =
            SystemState::new(&mut world);
        let mut event_writer = system_state.get_mut(&mut world);

        // Tick halfway through the first tween, then in one tick:
        // - Finish the first tween
        // - Start and finish the second tween
        // - Start the third tween
        for delta in [0.5, 2.0] {
            seq.tick(
                Duration::from_secs_f32(delta),
                &mut transform,
                Entity::from_raw(0),
                &mut event_writer,
            );
        }
        assert_eq!(seq.index(), 2);
        assert!(transform.translation.abs_diff_eq(Vec3::splat(2.5), 1e-5));
    }

    /// Sequence::new() and various Sequence-specific methods
    #[test]
    fn seq_iter() {
        let mut seq = Sequence::new((1..5).map(|i| {
            Tween::new(
                EaseMethod::Linear,
                Duration::from_secs_f32(0.2 * i as f32),
                TransformPositionLens {
                    start: Vec3::ZERO,
                    end: Vec3::ONE,
                },
            )
        }));

        let mut progress = 0.;
        for i in 1..5 {
            assert_eq!(seq.index(), i - 1);
            assert!((seq.progress() - progress).abs() < 1e-5);
            let secs = 0.2 * i as f32;
            assert_eq!(seq.current().duration(), Duration::from_secs_f32(secs));
            progress += 0.25;
            seq.set_progress(progress);
            assert_eq!(seq.times_completed(), if i == 4 { 1 } else { 0 });
        }

        seq.rewind();
        assert_eq!(seq.progress(), 0.);
        assert_eq!(seq.times_completed(), 0);
    }

    /// Test ticking parallel tracks of tweens.
    #[test]
    fn tracks_tick() {
        let tween1 = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(1.),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let tween2 = Tween::new(
            EaseMethod::Linear,
            Duration::from_secs_f32(0.8), // shorter
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_x(90_f32.to_radians()),
            },
        );
        let mut tracks = Tracks::new([tween1, tween2]);
        assert_eq!(tracks.duration(), Duration::from_secs_f32(1.)); // max(1., 0.8)

        let mut transform = Transform::default();

        // Dummy world and event writer
        let mut world = World::new();
        world.insert_resource(Events::<TweenCompleted>::default());
        let mut system_state: SystemState<EventWriter<TweenCompleted>> =
            SystemState::new(&mut world);
        let mut event_writer = system_state.get_mut(&mut world);

        for i in 1..=6 {
            let state = tracks.tick(
                Duration::from_secs_f32(0.2),
                &mut transform,
                Entity::from_raw(0),
                &mut event_writer,
            );
            if i < 5 {
                assert_eq!(state, TweenState::Active);
                assert_eq!(tracks.times_completed(), 0);
                let r = i as f32 * 0.2;
                assert!((tracks.progress() - r).abs() < 1e-5);
                let alpha_deg = 22.5 * i as f32;
                assert!(transform.translation.abs_diff_eq(Vec3::splat(r), 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(alpha_deg.to_radians()), 1e-5));
            } else {
                assert_eq!(state, TweenState::Completed);
                assert_eq!(tracks.times_completed(), 1);
                assert!((tracks.progress() - 1.).abs() < 1e-5);
                assert!(transform.translation.abs_diff_eq(Vec3::ONE, 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(90_f32.to_radians()), 1e-5));
            }
        }

        tracks.rewind();
        assert_eq!(tracks.times_completed(), 0);
        assert!(tracks.progress().abs() < 1e-5);

        tracks.set_progress(0.9);
        assert!((tracks.progress() - 0.9).abs() < 1e-5);
        // tick to udpate state (set_progress() does not update state)
        let state = tracks.tick(
            Duration::from_secs_f32(0.),
            &mut transform,
            Entity::from_raw(0),
            &mut event_writer,
        );
        assert_eq!(state, TweenState::Active);
        assert_eq!(tracks.times_completed(), 0);

        tracks.set_progress(3.2);
        assert!((tracks.progress() - 1.).abs() < 1e-5);
        // tick to udpate state (set_progress() does not update state)
        let state = tracks.tick(
            Duration::from_secs_f32(0.),
            &mut transform,
            Entity::from_raw(0),
            &mut event_writer,
        );
        assert_eq!(state, TweenState::Completed);
        assert_eq!(tracks.times_completed(), 1); // no looping

        tracks.set_progress(-0.5);
        assert!(tracks.progress().abs() < 1e-5);
        // tick to udpate state (set_progress() does not update state)
        let state = tracks.tick(
            Duration::from_secs_f32(0.),
            &mut transform,
            Entity::from_raw(0),
            &mut event_writer,
        );
        assert_eq!(state, TweenState::Active);
        assert_eq!(tracks.times_completed(), 0); // no looping
    }

    /// Test ticking a delay.
    #[test]
    fn delay_tick() {
        let duration = Duration::from_secs_f32(1.0);
        let mut delay = Delay::new(duration);
        {
            let tweenable: &dyn Tweenable<Transform> = &delay;
            assert_eq!(tweenable.duration(), duration);
            assert!(tweenable.progress().abs() < 1e-5);
        }

        let mut transform = Transform::default();

        // Dummy world and event writer
        let mut world = World::new();
        world.insert_resource(Events::<TweenCompleted>::default());
        let mut system_state: SystemState<EventWriter<TweenCompleted>> =
            SystemState::new(&mut world);
        let mut event_writer = system_state.get_mut(&mut world);

        for i in 1..=6 {
            let state = delay.tick(
                Duration::from_secs_f32(0.2),
                &mut transform,
                Entity::from_raw(0),
                &mut event_writer,
            );
            {
                let tweenable: &dyn Tweenable<Transform> = &delay;
                if i < 5 {
                    assert_eq!(state, TweenState::Active);
                    let r = i as f32 * 0.2;
                    assert!((tweenable.progress() - r).abs() < 1e-5);
                } else {
                    assert_eq!(state, TweenState::Completed);
                    assert!((tweenable.progress() - 1.).abs() < 1e-5);
                }
            }
        }
    }
}
