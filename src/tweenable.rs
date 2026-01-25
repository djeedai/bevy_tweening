use std::{any::TypeId, cmp::Ordering, time::Duration};

use bevy::{ecs::change_detection::MutUntyped, prelude::*};

use crate::{AnimTargetKind, EaseMethod, Lens, PlaybackDirection, RepeatCount, RepeatStrategy};

/// The dynamic tweenable type.
///
/// When creating lists of tweenables, you will need to box them to create a
/// homogeneous array like so:
///
/// ```no_run
/// # use bevy::prelude::Transform;
/// # use bevy_tweening::{BoxedTweenable, Delay, Sequence, Tween};
/// #
/// # let delay: Delay = unimplemented!();
/// # let tween: Tween = unimplemented!();
///
/// Sequence::new([Box::new(delay) as BoxedTweenable, tween.into()]);
/// ```
///
/// When using your own [`Tweenable`] types, APIs will be easier to use if you
/// implement [`From`]:
///
/// ```no_run
/// # use std::{any::TypeId, time::Duration};
/// # use bevy::ecs::{system::{Commands, SystemId}, change_detection::MutUntyped};
/// # use bevy::prelude::*;
/// # use bevy_tweening::{BoxedTweenable, Sequence, Tweenable, CycleCompletedEvent, TweenState, TotalDuration};
/// #
/// # #[derive(Debug)]
/// # struct MyTweenable;
/// # impl Tweenable for MyTweenable {
/// #     fn cycle_duration(&self) -> Duration  { unimplemented!() }
/// #     fn total_duration(&self) -> TotalDuration  { unimplemented!() }
/// #     fn set_elapsed(&mut self, elapsed: Duration)  { unimplemented!() }
/// #     fn elapsed(&self) -> Duration  { unimplemented!() }
/// #     fn step(&mut self, tween_id: Entity, delta: Duration, target: MutUntyped, target_type_id: &TypeId, notify_cycle_completed: &mut dyn FnMut(),) -> (TweenState, bool)  { unimplemented!() }
/// #     fn rewind(&mut self) { unimplemented!() }
/// #     fn cycles_completed(&self) -> u32 { unimplemented!() }
/// #     fn cycle_fraction(&self) -> f32 { unimplemented!() }
/// #     fn target_type_id(&self) -> Option<TypeId> { unimplemented!() }
/// # }
///
/// Sequence::new([Box::new(MyTweenable) as BoxedTweenable]);
///
/// // OR
///
/// Sequence::new([MyTweenable]);
///
/// impl From<MyTweenable> for BoxedTweenable {
///     fn from(t: MyTweenable) -> Self {
///         Box::new(t)
///     }
/// }
/// ```
pub type BoxedTweenable = Box<dyn Tweenable + 'static>;

/// Helper trait to accept boxed and non-boxed tweenables in various functions.
pub trait IntoBoxedTweenable {
    /// Convert to a [`BoxedTweenable`].
    fn into_boxed(self) -> BoxedTweenable;
}

impl IntoBoxedTweenable for BoxedTweenable {
    fn into_boxed(self) -> BoxedTweenable {
        self
    }
}

impl<T: Tweenable + 'static> IntoBoxedTweenable for T {
    fn into_boxed(self) -> BoxedTweenable {
        Box::new(self)
    }
}

/// Playback state of a [`Tweenable`].
///
/// This is returned by [`Tweenable::step()`] to allow the caller to execute
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

/// Event raised when an animation completed a single cycle.
///
/// This event is raised when a [`Tweenable`] animation completed a single
/// cycle. In case the animation direction changes each cycle
/// ([`RepeatStrategy::MirroredRepeat`]), a cycle corresponds to a single
/// progress from one endpoint value of the lens to the other, whatever the
/// direction. Therefore a complete loop start -> end -> start counts as 2
/// cycles and raises 2 events (one when reaching the end value, one when
/// reaching back the start value).
///
/// # Note
///
/// The semantic is different from [`TweenState::Completed`], which indicates
/// that the tweenable has finished stepping and do not need to be updated
/// anymore, a state which is never reached for looping animation. Here the
/// [`CycleCompletedEvent`] instead marks the end of a single cycle.
#[derive(Copy, Clone, EntityEvent, Message)]
pub struct CycleCompletedEvent {
    /// The entity owning the tweenable animation which completed.
    ///
    /// This is the entity owning the [`TweenAnim`] component that the tweenable
    /// which completed is part of.
    ///
    /// [`TweenAnim`]: crate::TweenAnim
    #[event_target]
    pub anim_entity: Entity,
    /// The target the tweenable which completed and the [`TweenAnim`] it's
    /// part of are mutating. Note that an actual [`AnimTarget`] component might
    /// not be spawned in the ECS world, if the target is a component on the
    /// same entity as the one owning the [`TweenAnim`] ("implicit component
    /// targetting"). But this field is always equal to the valid value that
    /// would otherwise exist as component.
    ///
    /// [`TweenAnim`]: crate::TweenAnim
    /// [`AnimTarget`]: crate::AnimTarget
    pub target: AnimTargetKind,
}

#[derive(Debug)]
struct AnimClock {
    elapsed: Duration,
    cycle_duration: Duration,
    total_duration: TotalDuration,
    strategy: RepeatStrategy,
}

impl AnimClock {
    fn new(cycle_duration: Duration) -> Self {
        Self {
            elapsed: Duration::ZERO,
            cycle_duration,
            total_duration: TotalDuration::from_cycles(cycle_duration, RepeatCount::default()),
            strategy: RepeatStrategy::default(),
        }
    }

    fn tick(&mut self, tick: Duration) -> (TweenState, i32) {
        let mut next_elapsed = self.elapsed.saturating_add(tick);

        let mut extra_completed: i32 = 0;
        if !self.total_duration.is_finite() {
            // Infinite tweens loops around...
            let period = if self.strategy == RepeatStrategy::MirroredRepeat {
                // ...over 2 cycles if mirrored
                self.cycle_duration * 2
            } else {
                // ...over 1 cycle if not
                self.cycle_duration
            };
            if next_elapsed >= period {
                // Common case, just loop once
                next_elapsed -= period;
                extra_completed += 1;

                // In case of very large jumps, handle arbitrary cycle count
                if next_elapsed >= period {
                    let count = next_elapsed.div_duration_f64(period) as u32;
                    next_elapsed -= period * count;
                    extra_completed += count as i32;
                    debug_assert!(next_elapsed < period);
                }
            }
        };

        let (state, mut times_completed) =
            self.set_elapsed(next_elapsed, PlaybackDirection::Forward);

        if extra_completed > 0 && (times_completed < 0) {
            // The clock looped around, so returns -1. But we're already counting that loop
            // in the extra cycles calculated above. It can't return anything else than -1
            // or 0 because we clamped next_elapsed.
            debug_assert_eq!(-1, times_completed);
            times_completed = 0;
        }

        (state, times_completed + extra_completed)
    }

    fn tick_back(&mut self, mut tick: Duration) -> (TweenState, i32) {
        let mut next_elapsed = self.elapsed.saturating_sub(tick);

        if !self.total_duration.is_finite() && (tick >= self.elapsed) {
            // Infinite tweens loops around...
            let period = if self.strategy == RepeatStrategy::MirroredRepeat {
                // ...over 2 cycles if mirrored
                self.cycle_duration * 2
            } else {
                // ...over 1 cycle if not
                self.cycle_duration
            };

            // Consume some time to move back to t=0
            tick -= self.elapsed;

            // In case of very large jumps, handle arbitrary cycle count
            if tick >= period {
                let count = tick.div_duration_f64(period) as u32;
                tick -= period * count;
            }

            // Common case, just loop once
            debug_assert!(tick < period);
            next_elapsed = if tick == Duration::ZERO {
                Duration::ZERO
            } else {
                period - tick
            };
        };

        self.set_elapsed(next_elapsed, PlaybackDirection::Backward)
    }

    /// Get the elapsed cycle index, accounting for finite clock endpoint.
    fn cycle_index(&self) -> u32 {
        let index = self.elapsed.div_duration_f64(self.cycle_duration) as u32;
        if let TotalDuration::Finite(total_duration) = self.total_duration {
            if self.elapsed >= total_duration {
                return index - 1;
            }
        }
        index
    }

    /// Get the elapsed cycle fraction, accounting for finite clock endpoint.
    fn cycle_fraction(&self) -> f32 {
        let factor = self.elapsed.div_duration_f64(self.cycle_duration).fract() as f32;
        if let TotalDuration::Finite(total_duration) = self.total_duration {
            if self.elapsed >= total_duration {
                return 1.0;
            }
        }
        factor
    }

    /// Get the mirroring-aware cycle fraction, accounting for finite clock
    /// endpoint.
    fn mirrored_cycle_fraction(&self) -> f32 {
        let ratio = self.elapsed.div_duration_f64(self.cycle_duration);
        let index = ratio as u32;
        let factor = ratio.fract() as f32;
        if let TotalDuration::Finite(total_duration) = self.total_duration {
            if self.elapsed >= total_duration {
                if self.is_cycle_mirrored(index - 1) {
                    return 0.0;
                } else {
                    return 1.0;
                }
            }
        }
        if self.is_cycle_mirrored(index) {
            1.0 - factor
        } else {
            factor
        }
    }

    /// Check if the current cycle is a mirrored cycle.
    #[must_use]
    #[inline]
    pub fn is_cycle_mirrored(&self, index: u32) -> bool {
        if self.strategy == RepeatStrategy::MirroredRepeat {
            (index & 1) != 0
        } else {
            false
        }
    }

    fn times_completed(&self) -> u32 {
        self.elapsed.div_duration_f64(self.cycle_duration) as u32
    }

    fn set_elapsed(
        &mut self,
        elapsed: Duration,
        direction: PlaybackDirection,
    ) -> (TweenState, i32) {
        let old_times_completed = self.times_completed();

        self.elapsed = elapsed;

        let state = match self.total_duration {
            TotalDuration::Finite(total_duration) => {
                // Always clamp
                self.elapsed = self.elapsed.min(total_duration);

                if (direction.is_forward() && self.elapsed >= total_duration)
                    || (direction.is_backward() && self.elapsed == Duration::ZERO)
                {
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

    fn state(&self, playback_direction: PlaybackDirection) -> TweenState {
        match self.total_duration {
            TotalDuration::Finite(total_duration) => {
                if (playback_direction.is_forward() && self.elapsed >= total_duration)
                    || (playback_direction.is_backward() && self.elapsed == Duration::ZERO)
                {
                    TweenState::Completed
                } else {
                    TweenState::Active
                }
            }
            TotalDuration::Infinite => TweenState::Active,
        }
    }

    fn rewind(&mut self, direction: PlaybackDirection) {
        self.elapsed = match direction {
            PlaybackDirection::Forward => Duration::ZERO,
            PlaybackDirection::Backward => self.total_duration.as_finite().unwrap(),
        };
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

impl TotalDuration {
    /// Create a [`TotalDuration`] from single cycle duration and a
    /// [`RepeatCount`].
    pub fn from_cycles(cycle_duration: Duration, repeat_count: RepeatCount) -> Self {
        match repeat_count {
            RepeatCount::Finite(times) => {
                TotalDuration::Finite(cycle_duration.saturating_mul(times))
            }
            RepeatCount::For(duration) => TotalDuration::Finite(duration),
            RepeatCount::Infinite => TotalDuration::Infinite,
        }
    }

    /// Return `true` if this is a [`TotalDuration::Finite`].
    pub fn is_finite(&self) -> bool {
        matches!(self, TotalDuration::Finite(_))
    }

    /// Return this duration as a [`Duration`] if it's finite.
    pub fn as_finite(&self) -> Option<Duration> {
        match self {
            Self::Finite(duration) => Some(*duration),
            Self::Infinite => None,
        }
    }
}

impl From<Duration> for TotalDuration {
    fn from(value: Duration) -> Self {
        TotalDuration::Finite(value)
    }
}

impl std::ops::Add for TotalDuration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (TotalDuration::Finite(d0), TotalDuration::Finite(d1)) => {
                TotalDuration::Finite(d0 + d1)
            }
            _ => TotalDuration::Infinite,
        }
    }
}

impl std::iter::Sum for TotalDuration {
    fn sum<I: Iterator<Item = Self>>(mut iter: I) -> Self {
        let Some(mut acc) = iter.next() else {
            return TotalDuration::Finite(Duration::ZERO);
        };
        for td in iter {
            acc = acc + td;
        }
        acc
    }
}

impl PartialOrd for TotalDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalDuration {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (TotalDuration::Finite(d0), TotalDuration::Finite(d1)) => d0.cmp(d1),
            (TotalDuration::Finite(_), TotalDuration::Infinite) => Ordering::Less,
            (TotalDuration::Infinite, TotalDuration::Finite(_)) => Ordering::Greater,
            (TotalDuration::Infinite, TotalDuration::Infinite) => Ordering::Equal,
        }
    }
}

/// Tweening animation description, either a single [`Tween`] or a collection of
/// them.
pub trait Tweenable: Send + Sync {
    /// Get the duration of a single cycle of the animation.
    ///
    /// Note that for [`RepeatStrategy::MirroredRepeat`], this is the duration
    /// of a single way, either from start to end or back from end to start.
    /// The total "loop" duration start -> end -> start to reach back the
    /// same state in this case is the double of the cycle duration.
    #[must_use]
    fn cycle_duration(&self) -> Duration;

    /// Get the total duration of the entire animation, including repeating.
    ///
    /// For [`TotalDuration::Finite`], this is the number of repeats times the
    /// duration of a single cycle ([`cycle_duration()`]).
    ///
    /// [`cycle_duration()`]: Self::cycle_duration
    #[must_use]
    fn total_duration(&self) -> TotalDuration;

    /// Set the current animation playback elapsed time.
    ///
    /// See [`elapsed()`] for details on the meaning. For finite durations, if
    /// `elapsed` is greater than or equal to [`total_duration()`], then the
    /// animation completes. Animations with infinite duration never complete.
    ///
    /// Setting the elapsed time seeks the animation to a new position, but does
    /// not apply that change to the underlying component being animated yet. To
    /// force the change to apply, call [`step()`] with a `delta` of
    /// `Duration::ZERO`, or wait for it to be automatically called.
    ///
    /// [`elapsed()`]: Tweenable::elapsed
    /// [`total_duration()`]: Tweenable::total_duration
    /// [`step()`]: Tweenable::step
    fn set_elapsed(&mut self, elapsed: Duration);

    /// Get the current elapsed duration.
    ///
    /// The elapsed duration is the time from the start of the tweening
    /// animation. It includes all cycles; if the animation repeats (has more
    /// than one cycle), the value can be greater than one
    /// [`cycle_duration()`]. The value differs depending on whether the
    /// animation repeat infinitely or not:
    /// - For **finite** repeat counts, including no repeat at all (count = 1),
    ///   the value is always between `0` and [`total_duration()`]. It
    ///   represents the absolute position over the timeline of all cycles.
    /// - For **infinite** repeat, the value loops around after either 1 cycle
    ///   (for [`RepeatStrategy::Repeat`]) or 2 cycles (for
    ///   [`RepeatStrategy::MirroredRepeat`]). The latter is necessary to
    ///   account for one non-mirrored cycle and one mirrored one.
    ///
    /// [`cycle_duration()`]: Tweenable::cycle_duration
    /// [`total_duration()`]: Tweenable::total_duration
    #[must_use]
    fn elapsed(&self) -> Duration;

    /// Step the tweenable.
    ///
    /// Advance the internal clock of the animation by the specified amount of
    /// time. If the animation is currently playing backward
    /// ([`PlaybackDirection::Backward`]), the clock moves backward and the
    /// `delta` duration is subtracted from the [`elapsed()`] time instead of
    /// being added to it.
    ///
    /// Note that `delta = Duration::ZERO` is valid, and is sometimes useful to
    /// force applying the result of a state change to the underlying
    /// animation target.
    ///
    /// # Returns
    ///
    /// Returns the state of the tweenable after the step.
    ///
    /// [`elapsed()`]: Tweenable::elapsed
    fn step(
        &mut self,
        tween_id: Entity,
        delta: Duration,
        target: MutUntyped,
        target_type_id: &TypeId,
        notify_cycle_completed: &mut dyn FnMut(),
    ) -> (TweenState, bool);

    /// Rewind the animation to its starting state.
    ///
    /// Note that the starting state depends on the current direction. For
    /// [`PlaybackDirection::Forward`] this is the start point of the lens,
    /// whereas for [`PlaybackDirection::Backward`] this is the end one.
    ///
    /// # Panics
    ///
    /// This panics if the current playback direction is
    /// [`PlaybackDirection::Backward`] and the animation is infinitely
    /// repeating.
    fn rewind(&mut self);

    /// Get the number of cycles completed.
    ///
    /// For repeating animations, this returns the number of times a single
    /// playback cycle was completed. In the case of
    /// [`RepeatStrategy::MirroredRepeat`] this corresponds to a playback in
    /// a single direction, so tweening from start to end and back to start
    /// counts as two completed cycles (one forward, one backward).
    #[must_use]
    fn cycles_completed(&self) -> u32 {
        self.elapsed().div_duration_f64(self.cycle_duration()) as u32
    }

    /// Get the completion fraction in `[0:1]` of the current cycle.
    #[must_use]
    fn cycle_fraction(&self) -> f32 {
        self.elapsed()
            .div_duration_f64(self.cycle_duration())
            .fract() as f32
    }

    /// Get the [`TypeId`] this tweenable targets.
    ///
    /// This returns the type of the component or asset that the [`TweenAnim`]
    /// needs to fetch from the ECS `World` in order to resolve the animation
    /// target.
    ///
    /// # Returns
    ///
    /// Returns the type of the target, if any, or `None` if this tweenable is
    /// untyped. Typically only [`Delay`] is untyped, as this is the only
    /// tweenable which doesn't actually mutate the target, so it doesn't
    /// actually have any target type associated with it.
    ///
    /// [`TweenAnim`]: crate::TweenAnim
    #[must_use]
    fn target_type_id(&self) -> Option<TypeId>;
}

macro_rules! impl_boxed {
    ($tweenable:ty) => {
        impl From<$tweenable> for BoxedTweenable {
            fn from(t: $tweenable) -> Self {
                Box::new(t)
            }
        }
    };
}

impl_boxed!(Tween);
impl_boxed!(Sequence);
impl_boxed!(Delay);

type TargetAction = dyn FnMut(MutUntyped, f32) + Send + Sync + 'static;

/// Configuration to create a [`Tween`].
///
/// This is largely an internal type, only exposed due to other constraints.
#[doc(hidden)]
#[derive(Default, Clone, Copy)]
pub struct TweenConfig {
    /// Ease method.
    pub ease_method: EaseMethod,
    /// Playback direction.
    pub playback_direction: PlaybackDirection,
    /// Send [`CycleCompletedEvent`]?
    pub send_cycle_completed_event: bool,
    /// Cycle duration.
    pub cycle_duration: Duration,
    /// Repeat count.
    pub repeat_count: RepeatCount,
    /// Repeat strategy.
    pub repeat_strategy: RepeatStrategy,
}

/// Single tweening animation description.
///
/// A _tween_ is the basic building block of an animation. It describes a single
/// tweening animation between two values, accessed through a given [`Lens`].
/// The animation is composed of one or more cycles, and can be played forward
/// or backward. On completion, you can be notified via events, observers, or
/// the execution of a one-shot system.
///
/// _If you're looking for the runtime representation of a tweenable animation,
/// see [`TweenAnim`] instead._
///
/// # Cycles
///
/// A tween can be configured to repeat multiple, or even an infinity, of times.
/// Each repeat iteration is called a _cycle_. The duration of a single cycle is
/// the _cycle duration_, and the duration of the entire tween animation
/// including all cycles is the _total duration_.
#[doc = include_str!("../images/tween_cycles.svg")]
///
/// _An example tween with 5 cycles._
///
/// The number of cycles is configured through the [`RepeatCount`].
///
/// - [`RepeatCount::Finite`] directly sets a number of cycles. The total
///   duration is inferred from the cycle duration and number of cycles.
/// - [`RepeatCount::For`] selects a total duration for the animation, from
///   which a number of cycles is derived. In that case, the number of cycles
///   may be a fractional number; the last cycle is only partial, and may not
///   reach the endpoint of the lens.
/// - [`RepeatCount::Infinite`] enables infinitely-repeating cycles. In that
///   case the total duration of the animation is infinite, and the animation
///   itself is said to be infinite.
///
/// The _repeat strategy_ determines whether cycles are mirrored when they
/// repeat. By default, all cycles produce a linear _ratio_ monotonically
/// increasing from `0` to `1`. When mirrored, every other cycle instead
/// produces a _decreasing_ ratio from `1` to `0`. The repeat strategy is
/// configured with [`RepeatStrategy`].
#[doc = include_str!("../images/tween_mirrored.svg")]
///
/// _A tween with 5 cycles, using the mirrored repeat strategy._
///
/// Once the ratio in `[0:1]` has been calculated, it's passed through the
/// easing function to obtain the final interpolation factor that
/// [`Lens::lerp()`] receives.
///
/// # Elapsed time and playback direction
///
/// The _elapsed time_ of an animation represents the current time since the
/// start of the animation. This includes all cycles, and is bound by the total
/// duration of the animation. This is a property of an active animation, and is
/// available per instance from the [`TweenAnim`] representing the instance. So
/// a tween itself, which describes an animation without a specific target to
/// animate, doesn't have an elapsed time. You can think of the elapsed time as
/// the current time position on some animation timeline.
///
/// The tween however has a _playback direction_. By default, the playback
/// direction is [`PlaybackDirection::Forward`], and the animation plays forward
/// as described above. By instead using [`PlaybackDirection::Backward`], the
/// tween plays in reverse from end to start. Practically, this means that the
/// elapsed time _decreases_ from its current value back to zero, and the
/// animation completes at `t=0`. You can think of the playback direction as the
/// direction in which the time position moves on some animation timeline. Note
/// that as a result, because infinite animations ([`RepeatCount::Infinite`])
/// don't have an end time, they cannot be rewinded when the playback direction
/// is backward.
///
/// # Completion events and one-shot systems
///
/// Sometimes, you want to be notified of the completion of an animation cycle,
/// or the completion of the entire animation itself. To that end, the [`Tween`]
/// supports several mechanisms:
///
/// - Each time a _single_ cycle is completed, the tween can emit a
///   [`CycleCompletedEvent`]. The event is emitted as a buffered event, to be
///   read by another system through an [`MessageReader`]. For component targets,
///   observers are also triggered. Both of these are enabled through
///   [`with_cycle_completed_event()`] and [`set_cycle_completed_event()`].
///   Per-cycle events are disabled by default.
/// - At the end of all cycles, when the animation itself completes, the tween
///   emits an [`AnimCompletedEvent`]. This event is always emitted.
///
/// [`TweenAnim`]: crate::TweenAnim
/// [`with_cycle_completed_event()`]: Self::with_cycle_completed_event
/// [`set_cycle_completed_event()`]: Self::set_cycle_completed_event
/// [`AnimCompletedEvent`]: crate::AnimCompletedEvent
pub struct Tween {
    ease_method: EaseMethod,
    clock: AnimClock,
    /// Direction of playback the user asked for.
    playback_direction: PlaybackDirection,
    action: Box<TargetAction>,
    send_cycle_completed_event: bool,
    /// Type ID of the target.
    type_id: TypeId,
}

impl Tween {
    /// Create a new tween animation.
    ///
    /// The new animation is described by a given cycle duration, a repeat count
    /// which determines its total duration, as well as an easing function and a
    /// lens describing how the cycles affect the animation target. The target
    /// type is implicitly determined by the type `T` of the [`Lens<T>`]
    /// argument.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::math::{Vec3, curve::EaseFunction};
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
    #[inline]
    #[must_use]
    pub fn new<T, L>(
        ease_method: impl Into<EaseMethod>,
        cycle_duration: Duration,
        mut lens: L,
    ) -> Self
    where
        T: 'static,
        L: Lens<T> + Send + Sync + 'static,
    {
        let action = move |ptr: MutUntyped, ratio: f32| {
            // SAFETY: ptr was obtained from the same type, via the type_id saved below.
            #[allow(unsafe_code)]
            let target = unsafe { ptr.with_type::<T>() };
            lens.lerp(target, ratio);
        };
        Self {
            ease_method: ease_method.into(),
            clock: AnimClock::new(cycle_duration),
            playback_direction: PlaybackDirection::Forward,
            action: Box::new(action),
            send_cycle_completed_event: false,
            type_id: TypeId::of::<T>(),
        }
    }

    #[inline]
    #[must_use]
    pub(crate) fn from_config<T, L>(config: TweenConfig, mut lens: L) -> Self
    where
        T: 'static,
        L: Lens<T> + Send + Sync + 'static,
    {
        let action = move |ptr: MutUntyped, ratio: f32| {
            // SAFETY: ptr was obtained from the same type, via the type_id saved below.
            #[allow(unsafe_code)]
            let target = unsafe { ptr.with_type::<T>() };
            lens.lerp(target, ratio);
        };
        let this = Self {
            ease_method: config.ease_method,
            clock: AnimClock::new(config.cycle_duration),
            playback_direction: config.playback_direction,
            action: Box::new(action),
            send_cycle_completed_event: config.send_cycle_completed_event,
            type_id: TypeId::of::<T>(),
        };
        this.with_repeat(config.repeat_count, config.repeat_strategy)
    }

    /// Set the number of times to repeat the animation.
    ///
    /// The repeat count determines the number of cycles of the animation. See
    /// [the top-level `Tween` documentation] for details.
    ///
    /// [the top-level `Tween` documentation]: crate::Tween#cycles
    #[must_use]
    pub fn with_repeat_count(mut self, count: impl Into<RepeatCount>) -> Self {
        self.clock.total_duration =
            TotalDuration::from_cycles(self.clock.cycle_duration, count.into());
        self
    }

    /// Set the number of times to repeat the animation.
    ///
    /// The repeat count determines the number of cycles of the animation. See
    /// [the top-level `Tween` documentation] for details.
    ///
    /// [the top-level `Tween` documentation]: crate::Tween#cycles
    pub fn set_repeat_count(&mut self, count: impl Into<RepeatCount>) {
        self.clock.total_duration =
            TotalDuration::from_cycles(self.clock.cycle_duration, count.into());
    }

    /// Configure how the cycles repeat.
    ///
    /// This enables or disables cycle mirroring. See [the top-level `Tween`
    /// documentation] for details.
    ///
    /// [the top-level `Tween` documentation]: crate::Tween#cycles
    #[must_use]
    pub fn with_repeat_strategy(mut self, strategy: RepeatStrategy) -> Self {
        self.clock.strategy = strategy;
        self
    }

    /// Configure how the cycles repeat.
    ///
    /// This enables or disables cycle mirroring. See [the top-level `Tween`
    /// documentation] for details.
    ///
    /// [the top-level `Tween` documentation]: crate::Tween#cycles
    pub fn set_repeat_strategy(&mut self, strategy: RepeatStrategy) {
        self.clock.strategy = strategy;
    }

    /// Configure the animation repeat parameters.
    ///
    /// The repeat count determines the number of cycles of the animation. The
    /// repeat strategy enables or disables cycle mirrored repeat. See
    /// [the top-level `Tween` documentation] for details.
    ///
    /// [the top-level `Tween` documentation]: crate::Tween#cycles
    #[must_use]
    #[inline]
    pub fn with_repeat(self, count: impl Into<RepeatCount>, strategy: RepeatStrategy) -> Self {
        self.with_repeat_count(count).with_repeat_strategy(strategy)
    }

    /// Configure the animation repeat parameters.
    ///
    /// The repeat count determines the number of cycles of the animation. The
    /// repeat strategy enables or disables cycle mirrored repeat. See
    /// [the top-level `Tween` documentation] for details.
    ///
    /// [the top-level `Tween` documentation]: crate::Tween#cycles
    #[inline]
    pub fn set_repeat(&mut self, count: impl Into<RepeatCount>, strategy: RepeatStrategy) {
        self.set_repeat_count(count);
        self.set_repeat_strategy(strategy);
    }

    /// Enable raising a event on cycle completion.
    ///
    /// If enabled, the tween will raise a [`CycleCompletedEvent`] each time
    /// the tween completes a cycle (reaches or passes its cycle duration). In
    /// case of repeating tweens (repeat count > 1), the event is raised once
    /// per cycle. For mirrored repeats, a cycle is one travel from start to
    /// end **or** end to start, so the full loop start -> end -> start counts
    /// as 2 cycles and raises 2 events.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::{ecs::message::MessageReader, math::{Vec3, curve::EaseFunction}};
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
    /// // Raise a CycleCompletedEvent each cycle
    /// .with_cycle_completed_event(true);
    ///
    /// fn my_system(mut reader: MessageReader<CycleCompletedEvent>) {
    ///     for ev in reader.read() {
    ///         println!(
    ///             "Tween animation {:?} raised CycleCompletedEvent for target {:?}!",
    ///             ev.anim_entity, ev.target
    ///         );
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn with_cycle_completed_event(mut self, send: bool) -> Self {
        self.send_cycle_completed_event = send;
        self
    }

    /// Set whether the tween emits [`CycleCompletedEvent`].
    ///
    /// See [`with_cycle_completed_event()`] for details.
    ///
    /// [`with_cycle_completed_event()`]: Self::with_cycle_completed_event
    pub fn set_cycle_completed_event(&mut self, send: bool) {
        self.send_cycle_completed_event = send;
    }

    /// Set the playback direction of the tween.
    ///
    /// The playback direction controls whether the internal animation clock,
    /// and therefore also the elapsed time, both move forward or backward.
    ///
    /// Changing the direction doesn't change any target state, nor the elapsed
    /// time of the tween. Only the direction of playback from this moment
    /// potentially changes.
    pub fn set_playback_direction(&mut self, direction: PlaybackDirection) {
        self.playback_direction = direction;
    }

    /// Set the playback direction of the tween.
    ///
    /// See [`set_playback_direction()`] for details.
    ///
    /// [`set_playback_direction()`]: Self::set_playback_direction
    #[must_use]
    pub fn with_playback_direction(mut self, direction: PlaybackDirection) -> Self {
        self.playback_direction = direction;
        self
    }

    /// The current animation playback direction.
    ///
    /// This is the value set by the user with [`with_playback_direction()`] and
    /// [`set_playback_direction()`]. This is never changed by the animation
    /// playback itself.
    ///
    /// See [`PlaybackDirection`] for details.
    ///
    /// [`with_playback_direction()`]: Self::with_playback_direction
    /// [`set_playback_direction()`]: Self::set_playback_direction
    #[must_use]
    pub fn playback_direction(&self) -> PlaybackDirection {
        self.playback_direction
    }

    /// Chain another [`Tweenable`] after this tween, making a [`Sequence`] with
    /// the two.
    ///
    /// # Example
    /// ```
    /// # use bevy_tweening::{lens::*, *};
    /// # use bevy::math::{*,curve::EaseFunction};
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
    pub fn then(self, tween: impl Tweenable + 'static) -> Sequence {
        Sequence::with_capacity(2).then(self).then(tween)
    }

    /// Get the elapsed cycle index (numbered from 0), accounting for finite
    /// endpoint.
    ///
    /// If the elapsed time is equal to the total (finite) duration of the
    /// tween, then the cycle index is capped at the total number of cycles
    /// minus 1 (the tween doesn't loop when reaching the end of its last
    /// cycle). This means that for a tween with N cycles, the index is always
    /// in `0..N`, and therefore always `index < N`.
    #[must_use]
    #[inline]
    pub fn cycle_index(&self) -> u32 {
        self.clock.cycle_index()
    }

    /// Get the elapsed cycle fraction, accounting for finite endpoint.
    ///
    /// The elapsed cycle fraction is the fraction alonside one cycle where the
    /// tween currently is. This ignores any mirroring. If the elapsed time is
    /// equal to the total (finite) duration of the tween, then the cycle
    /// fraction is capped at `1.0` (the tween doesn't loop when reaching
    /// the end of its last cycle).
    ///
    /// The returned value is always in `[0:1]` for finite tweens, with the
    /// value `1.0` returned only when the last cycle is completed, and in
    /// `[0:1)` for infinite tweens as they never complete.
    #[must_use]
    #[inline]
    pub fn cycle_fraction(&self) -> f32 {
        self.clock.cycle_fraction()
    }

    /// Check if the current cycle is a mirrored cycle.
    ///
    /// When the repeat strategy is [`RepeatStrategy::MirroredRepeat`], every
    /// odd cycle index (numbered from 0) is mirrored when applying the tween's
    /// lens. For any other strategy or single-cycle tween, this is always
    /// `false`.
    #[must_use]
    #[inline]
    pub fn is_cycle_mirrored(&self) -> bool {
        self.clock.is_cycle_mirrored(self.clock.cycle_index())
    }
}

impl Tweenable for Tween {
    fn cycle_duration(&self) -> Duration {
        self.clock.cycle_duration
    }

    fn total_duration(&self) -> TotalDuration {
        self.clock.total_duration
    }

    fn set_elapsed(&mut self, elapsed: Duration) {
        self.clock.set_elapsed(elapsed, self.playback_direction);
    }

    fn elapsed(&self) -> Duration {
        self.clock.elapsed()
    }

    fn step(
        &mut self,
        _tween_id: Entity,
        delta: Duration,
        target: MutUntyped,
        target_type_id: &TypeId,
        notify_cycle_completed: &mut dyn FnMut(),
    ) -> (TweenState, bool) {
        debug_assert_eq!(self.type_id, *target_type_id);

        if self.clock.state(self.playback_direction) == TweenState::Completed {
            return (TweenState::Completed, false);
        }

        // Advance the animation clock
        let (state, times_completed) = if self.playback_direction.is_forward() {
            self.clock.tick(delta)
        } else {
            self.clock.tick_back(delta)
        };

        // Apply the lens, even if the animation completed, to ensure the state is
        // consistent.
        let fraction = self.clock.mirrored_cycle_fraction();
        let fraction = self.ease_method.sample(fraction);
        (self.action)(target, fraction);

        // If completed at least once this frame, notify the user
        if times_completed != 0 && self.send_cycle_completed_event {
            notify_cycle_completed();
        }

        (state, false)
    }

    fn rewind(&mut self) {
        self.clock.rewind(self.playback_direction);
    }

    fn target_type_id(&self) -> Option<TypeId> {
        Some(self.type_id)
    }
}

/// A sequence of tweenable animations played in order one after the other.
pub struct Sequence {
    tweens: Vec<BoxedTweenable>,
    index: usize,
    cycle_duration: TotalDuration,
    total_duration: TotalDuration,
    elapsed: Duration,
}

impl Sequence {
    /// Create a new sequence of tweens.
    ///
    /// The collection of tweens form a single cycle of the sequence. If any
    /// constituting item is of infinite duration, the sequence itself becomes
    /// of infinite duration.
    ///
    /// # Panics
    ///
    /// Panics if the input collection is empty.
    #[must_use]
    #[inline]
    pub fn new(items: impl IntoIterator<Item = impl Into<BoxedTweenable>>) -> Self {
        let tweens: Vec<_> = items.into_iter().map(Into::into).collect();
        assert!(!tweens.is_empty());

        let total_duration = tweens.iter().map(|tween| tween.total_duration()).sum();

        Self {
            tweens,
            index: 0,
            cycle_duration: total_duration,
            total_duration,
            elapsed: Duration::ZERO,
        }
    }

    /// Create a new sequence containing a single tweenable animation.
    #[must_use]
    #[inline]
    pub fn from_single(tweenable: impl Tweenable + 'static) -> Self {
        let total_duration = tweenable.total_duration();
        let boxed: BoxedTweenable = Box::new(tweenable);
        Self {
            tweens: vec![boxed],
            index: 0,
            cycle_duration: total_duration,
            total_duration,
            elapsed: Duration::ZERO,
        }
    }

    /// Create a new sequence with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let total_duration = TotalDuration::Finite(Duration::ZERO);
        Self {
            tweens: Vec::with_capacity(capacity),
            index: 0,
            cycle_duration: total_duration,
            total_duration,
            elapsed: Duration::ZERO,
        }
    }

    /// Append a [`Tweenable`] to this sequence.
    #[must_use]
    pub fn then(mut self, tween: impl Tweenable + 'static) -> Self {
        self.total_duration = self.total_duration + tween.total_duration();
        self.cycle_duration = self.cycle_duration + tween.total_duration();
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
    pub fn current(&self) -> &dyn Tweenable {
        self.tweens[self.index()].as_ref()
    }
}

impl Tweenable for Sequence {
    fn cycle_duration(&self) -> Duration {
        // FIXME - unwrap() because we can cycle over an infinite child, but that's
        // undefined behavior
        self.cycle_duration.as_finite().unwrap()
    }

    fn total_duration(&self) -> TotalDuration {
        self.total_duration
    }

    fn set_elapsed(&mut self, elapsed: Duration) {
        // Set the total sequence elapsed time
        self.elapsed = elapsed;

        // Find which tween is active in the sequence at that time
        let mut accum_duration = Duration::ZERO;
        for (index, tween) in self.tweens.iter_mut().enumerate() {
            // Get the total duration of this child
            let tween_duration = tween.total_duration();

            // If the child is of finite duration, and elapsed is still bigger, then set
            // that child as completed and move to the next.
            if let TotalDuration::Finite(duration) = tween_duration {
                if elapsed >= accum_duration + duration {
                    accum_duration += duration;
                    tween.set_elapsed(duration);
                    continue;
                }
            };

            // Found a non-completed child (either infinite, or just elapsed not large
            // enough to complete it). Set its partially-elapsed state.
            self.index = index;
            let local_duration = elapsed - accum_duration;
            tween.set_elapsed(local_duration);

            // For all subsequent children, reset them to zero, so that the state is
            // consistent
            for i in (index + 1)..self.tweens.len() {
                self.tweens[i].set_elapsed(Duration::ZERO);
            }

            return;
        }

        // None found; sequence ended
        self.index = self.tweens.len();
    }

    fn elapsed(&self) -> Duration {
        self.elapsed
    }

    fn step(
        &mut self,
        tween_id: Entity,
        mut delta: Duration,
        mut target: MutUntyped,
        target_type_id: &TypeId,
        notify_completed: &mut dyn FnMut(),
    ) -> (TweenState, bool) {
        // Early out
        if self.index >= self.tweens.len() {
            return (TweenState::Completed, false);
        }

        // Calculate the new elapsed time at the end of this tick
        self.elapsed = self.elapsed.saturating_add(delta);
        if let TotalDuration::Finite(total_duration) = self.total_duration {
            self.elapsed = self.elapsed.min(total_duration);
        }

        // Tick one or more tweenables until the new elapsed time is reached.
        while self.index < self.tweens.len() {
            // Tick the current tweenable
            let tween = &mut self.tweens[self.index];

            let prev_elapsed = tween.elapsed();

            if let (TweenState::Active, retarget) = tween.step(
                tween_id,
                delta,
                target.reborrow(),
                target_type_id,
                notify_completed,
            ) {
                return (TweenState::Active, retarget);
            }

            let TotalDuration::Finite(total_duration) = tween.total_duration() else {
                // Note: Rust can't figure it out, but this can never happen, because infinite
                // children will always return TweenState::Active. Just add this for safety.
                return (TweenState::Active, false);
            };

            // Child tween has completed. So it was finite, and consumed all the time left
            // between its previous elapsed time and its total duration.
            let consumed_duration = total_duration.saturating_sub(prev_elapsed);
            delta -= consumed_duration;
            self.index += 1;

            // If the target type changed, we need to ask the caller to retarget and step
            // again.
            if self.index < self.tweens.len() {
                // If the tweenable it untyped, it's guaranteed to not access the target, so we
                // can pass any target as argument.
                if let Some(type_id) = self.tweens[self.index].target_type_id() {
                    if type_id != *target_type_id {
                        return (TweenState::Active, true);
                    }
                }
            }
        }

        (TweenState::Completed, false)
    }

    fn rewind(&mut self) {
        self.elapsed = Duration::ZERO;
        self.index = 0;
        for tween in &mut self.tweens {
            // or only first?
            tween.rewind();
        }
    }

    fn target_type_id(&self) -> Option<TypeId> {
        // Loop over all children, because we want to skip all untyped ones (Delay).
        // Otherwise the animator will panic, because we can't create an untyped
        // animation.
        // for tween in &self.tweens {
        //     if let Some(type_id) = tween.target_type_id() {
        //         return Some(type_id);
        //     }
        // }
        //None

        // TEMP - not supporting multi-target yet, so assert here so users know
        let mut target_type_id = None;
        for tween in &self.tweens {
            if let Some(type_id) = tween.target_type_id() {
                assert!(target_type_id.is_none() || target_type_id == Some(type_id), "TODO: Cannot use tweenable animations with different targets inside the same Sequence. Create separate animations for each target.");
                target_type_id = Some(type_id);
            }
        }
        target_type_id
    }
}

/// A time delay that doesn't animate anything.
///
/// This is generally useful for combining with other tweenables into sequences
/// and tracks, for example to delay the start of a tween in a track relative to
/// another track. The `menu` example (`examples/menu.rs`) uses this technique
/// to delay the animation of its buttons.
#[derive(Debug)]
pub struct Delay {
    timer: Timer,
}

impl Delay {
    /// Chain another [`Tweenable`] after this tween, making a [`Sequence`] with
    /// the two.
    #[must_use]
    pub fn then(self, tween: impl Tweenable + 'static) -> Sequence {
        Sequence::with_capacity(2).then(self).then(tween)
    }

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
        }
    }

    /// Check if the delay completed.
    pub fn is_completed(&self) -> bool {
        self.timer.is_finished()
    }

    /// Get the current tweenable state.
    pub fn state(&self) -> TweenState {
        if self.is_completed() {
            TweenState::Completed
        } else {
            TweenState::Active
        }
    }
}

impl Tweenable for Delay {
    fn cycle_duration(&self) -> Duration {
        self.timer.duration()
    }

    fn total_duration(&self) -> TotalDuration {
        TotalDuration::Finite(self.cycle_duration())
    }

    fn set_elapsed(&mut self, elapsed: Duration) {
        // need to reset() to clear is_finished() unfortunately
        self.timer.reset();
        self.timer.set_elapsed(elapsed);
        // set_elapsed() does not update finished() etc. which we rely on
        self.timer.tick(Duration::ZERO);
    }

    fn elapsed(&self) -> Duration {
        self.timer.elapsed()
    }

    fn step(
        &mut self,
        _tween_id: Entity,
        delta: Duration,
        _target: MutUntyped,
        _target_type_id: &TypeId,
        _notify_completed: &mut dyn FnMut(),
    ) -> (TweenState, bool) {
        self.timer.tick(delta);

        let state = self.state();

        // // If completed this frame, notify the user
        // if (state == TweenState::Completed) && !was_completed {
        //     if self.send_completed_events {
        //         let event = CycleCompletedEvent {
        //             id: tween_id,
        //             entity,
        //             progress,
        //         };

        //         // send regular event
        //         events.send(event);

        //         // trigger all entity-scoped observers
        //         //commands.trigger_targets(event, entity);
        //     }
        //     if let Some(system_id) = &self.system_id {
        //         commands.run_system(*system_id);
        //     }
        // }

        (state, false)
    }

    fn rewind(&mut self) {
        self.timer.reset();
    }

    fn target_type_id(&self) -> Option<TypeId> {
        None
    }
}

#[cfg(test)]
mod tests {
    // use std::sync::{Arc, Mutex};

    use std::ops::{Deref as _, DerefMut as _};

    use bevy::ecs::{change_detection::MaybeLocation, change_detection::Tick, system::SystemState};

    use super::*;
    use crate::{lens::*, test_utils::assert_approx_eq};

    // #[derive(Default, Copy, Clone)]
    // struct CallbackMonitor {
    //     invoke_count: u64,
    //     last_reported_count: u32,
    // }

    // Check the behavior of the functions calculating cycle information
    #[test]
    fn anim_clock_cycles() {
        // The direction passed to AnimClock::set_elapsed() only affects the returned
        // state, which is not tested in this test, so passing any value should produce
        // the same test result here.
        for dummy in [PlaybackDirection::Forward, PlaybackDirection::Backward] {
            let cycle_duration = Duration::from_millis(100);
            let repeat_count = 4;
            let total_duration = cycle_duration * repeat_count;
            let mut clock = AnimClock::new(cycle_duration);
            clock.total_duration = TotalDuration::Finite(total_duration);

            assert_eq!(cycle_duration, clock.cycle_duration);
            assert_eq!(RepeatStrategy::Repeat, clock.strategy);

            assert_eq!(Duration::ZERO, clock.elapsed());
            assert_eq!(0, clock.cycle_index());
            assert_approx_eq!(0.0, clock.cycle_fraction());
            assert_approx_eq!(0.0, clock.mirrored_cycle_fraction());

            let dt = Duration::from_millis(30);
            clock.set_elapsed(dt, dummy);
            assert_eq!(dt, clock.elapsed());
            assert_eq!(0, clock.cycle_index());
            assert_approx_eq!(0.3, clock.cycle_fraction());
            assert_approx_eq!(0.3, clock.mirrored_cycle_fraction());

            let dt = Duration::from_millis(110);
            clock.set_elapsed(dt, dummy);
            assert_eq!(dt, clock.elapsed());
            assert_eq!(1, clock.cycle_index());
            assert_approx_eq!(0.1, clock.cycle_fraction());
            assert_approx_eq!(0.1, clock.mirrored_cycle_fraction());

            let dt = Duration::from_millis(400);
            clock.set_elapsed(dt, dummy);
            assert_eq!(dt, clock.elapsed());
            assert_eq!(3, clock.cycle_index()); // doesn't loop at end
            assert_approx_eq!(1.0, clock.cycle_fraction()); // doesn't loop at end
            assert_approx_eq!(1.0, clock.mirrored_cycle_fraction());

            let dt = Duration::from_millis(410); // > total_duration; clamped
            clock.set_elapsed(dt, dummy);
            assert_eq!(total_duration, clock.elapsed()); // clamped
            assert_eq!(3, clock.cycle_index()); // doesn't loop at end
            assert_approx_eq!(1.0, clock.cycle_fraction()); // doesn't loop at end
            assert_approx_eq!(1.0, clock.mirrored_cycle_fraction());

            clock.strategy = RepeatStrategy::MirroredRepeat;

            let dt = Duration::from_millis(110);
            clock.set_elapsed(dt, dummy);
            assert_eq!(dt, clock.elapsed());
            assert_eq!(1, clock.cycle_index());
            assert_approx_eq!(0.1, clock.cycle_fraction());
            assert_approx_eq!(0.9, clock.mirrored_cycle_fraction()); // mirrored

            let dt = Duration::from_millis(400);
            clock.set_elapsed(dt, dummy);
            assert_eq!(dt, clock.elapsed());
            assert_eq!(3, clock.cycle_index()); // doesn't loop at end
            assert_approx_eq!(1.0, clock.cycle_fraction()); // doesn't loop at end
            assert_approx_eq!(0.0, clock.mirrored_cycle_fraction()); // mirrored

            let dt = Duration::from_millis(410); // > total_duration; clamped
            clock.set_elapsed(dt, dummy);
            assert_eq!(total_duration, clock.elapsed()); // clamped
            assert_eq!(3, clock.cycle_index()); // doesn't loop at end
            assert_approx_eq!(1.0, clock.cycle_fraction()); // doesn't loop at end
            assert_approx_eq!(0.0, clock.mirrored_cycle_fraction()); // mirrored
        }
    }

    // Check the accumulated error of the clock over a long period
    #[test]
    fn anim_clock_precision() {
        let cycle_duration = Duration::from_millis(1);
        let mut clock = AnimClock::new(cycle_duration);
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
        let num_iter = 100_000_000; // 100m ms, >27h
        for i in 0..num_iter {
            let tick = test_ticks[i % test_ticks.len()];
            times_completed += clock.tick(tick).1;
            total_duration += tick;
        }

        assert_eq!(
            total_duration.div_duration_f64(cycle_duration) as i32,
            times_completed
        );
    }

    /// Utility to create a tween for testing.
    fn make_test_tween() -> Tween {
        Tween::new(
            EaseMethod::default(),
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
        world.init_resource::<Messages<CycleCompletedEvent>>();
        let entity = world.spawn(Transform::default()).id();
        (world, entity)
    }

    /// Manually tick a test tweenable targeting a component.
    fn manual_tick_component(
        anim_entity: Entity,
        duration: Duration,
        tween: &mut dyn Tweenable,
        world: &mut World,
        entity: Entity,
    ) -> TweenState {
        // Tick the given tween and apply its state to the given entity target
        let target_type_id = TypeId::of::<Transform>();
        let ret = world.resource_scope(
            |world: &mut World, mut events: Mut<Messages<CycleCompletedEvent>>| {
                let component_id = world.component_id::<Transform>().unwrap();
                let entity_mut = &mut world.get_entity_mut([entity]).unwrap()[0];
                if let Ok(mut target) = entity_mut.get_mut_by_id(component_id) {
                    let world_target = AnimTargetKind::Component { entity };
                    let mut notify_completed = || {
                        events.write(CycleCompletedEvent {
                            anim_entity,
                            target: world_target,
                        });
                    };
                    tween.step(
                        anim_entity,
                        duration,
                        target.reborrow(),
                        &target_type_id,
                        &mut notify_completed,
                    )
                } else {
                    (TweenState::Completed, false)
                }
            },
        );

        // Propagate events
        {
            let mut events = world.resource_mut::<Messages<CycleCompletedEvent>>();
            events.update();
        }

        ret.0
    }

    #[derive(Debug, Default, Clone, Copy, Component)]
    struct DummyComponent {
        _value: f32,
    }

    #[test]
    fn targetable_change_detect() {
        let mut c = DummyComponent::default();
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

        // No-op at start
        assert!(!target.is_added());
        assert!(!target.is_changed());

        // Immutable deref doesn't trigger change detection
        let _ = target.deref();
        assert!(!target.is_added());
        assert!(!target.is_changed());

        // Mutable deref triggers change detection
        let _ = target.deref_mut();
        assert!(!target.is_added());
        assert!(target.is_changed());
    }

    #[test]
    fn into_repeat_count() {
        let tween = Tween::new(
            EaseMethod::default(),
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_repeat_count(5);
        assert_eq!(
            tween.total_duration(),
            TotalDuration::Finite(Duration::from_secs(5))
        );

        let tween = Tween::new(
            EaseMethod::default(),
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        )
        .with_repeat_count(Duration::from_secs(3));
        assert_eq!(
            tween.total_duration(),
            TotalDuration::Finite(Duration::from_secs(3))
        );
    }

    /// Test ticking of a single tween in isolation.
    #[test]
    fn tween_tick() {
        for playback_direction in [PlaybackDirection::Forward, PlaybackDirection::Backward] {
            for (count, strategy) in [
                (RepeatCount::Finite(1), RepeatStrategy::default()),
                (RepeatCount::Infinite, RepeatStrategy::Repeat),
                (RepeatCount::Finite(2), RepeatStrategy::Repeat),
                (RepeatCount::Infinite, RepeatStrategy::MirroredRepeat),
                (RepeatCount::Finite(2), RepeatStrategy::MirroredRepeat),
            ] {
                println!(
                    "TweeningType: playback={playback_direction:?} count={count:?} strategy={strategy:?}",
                );

                // Create a linear tween over 1 second
                let mut tween = make_test_tween()
                    .with_playback_direction(playback_direction)
                    .with_repeat_count(count)
                    .with_repeat_strategy(strategy)
                    .with_cycle_completed_event(true);
                assert_eq!(tween.playback_direction(), playback_direction);
                assert!(tween.send_cycle_completed_event);

                // Note: for infinite duration playing backward, we need to start *somewhere*
                // since we can't start at infinity. So pick t=1s which is shorter than 11
                // iterations of the 100ms test tween created by make_test_tween(), to ensure we
                // reach t=0.
                let backward_start_time = tween
                    .total_duration()
                    .as_finite()
                    .unwrap_or(Duration::from_secs(1));
                if playback_direction == PlaybackDirection::Backward {
                    // Seek to end, so that backward playback actually does something
                    tween.set_elapsed(backward_start_time);
                }

                let (mut world, entity) = make_test_env();
                let mut event_reader_system_state: SystemState<MessageReader<CycleCompletedEvent>> =
                    SystemState::new(&mut world);

                // Loop over 2.2 seconds, so greater than one ping-pong loop (2 cycles in
                // MirroredRepeat)
                let tick_duration = Duration::from_millis(200);
                let tween_duration = Duration::from_secs(1);
                for i in 1..=11 {
                    // Calculate expected values
                    let (elapsed_ms, factor, mut direction, expected_state, just_completed) =
                        match count {
                            RepeatCount::Finite(1) => {
                                let (elapsed_ms, state) = if playback_direction.is_forward() {
                                    if i < 5 {
                                        (i * 200i32, TweenState::Active)
                                    } else {
                                        (1000i32, TweenState::Completed)
                                    }
                                } else if i < 5 {
                                    (1000i32 - i * 200i32, TweenState::Active)
                                } else {
                                    (0i32, TweenState::Completed)
                                };
                                let just_completed = i == 5;
                                (
                                    elapsed_ms,
                                    elapsed_ms as f32 / 1000.0,
                                    PlaybackDirection::Forward,
                                    state,
                                    just_completed,
                                )
                            }
                            RepeatCount::Finite(count) => {
                                if strategy == RepeatStrategy::Repeat {
                                    let just_completed = i % 5 == 0;
                                    let elapsed_ms = if playback_direction.is_forward() {
                                        // 0.2, 0.4, 0.6, 0.8, 0.0, 0.2, 0.4, ... (x count)
                                        (i * 200) % 1000
                                    } else {
                                        // 0.8, 0.6, 0.4, 0.2, 0.0, 0.8, 0.6, ... (x count)
                                        800 - (((i - 1) * 200 - 1000) % 1000 + 1000) % 1000
                                    };
                                    let factor = if i >= 10 {
                                        if playback_direction.is_forward() {
                                            1.0
                                        } else {
                                            0.0
                                        }
                                    } else {
                                        elapsed_ms as f32 / 1000.0
                                    };
                                    let total_duration_ms = count as i32 * 1000;
                                    let elapsed_ms = if playback_direction.is_forward() {
                                        (i * 200).min(total_duration_ms)
                                    } else {
                                        (total_duration_ms - i * 200).max(0)
                                    };
                                    // The test is calibrates such that at i==10 the forward tick
                                    // reaches the total duration, and the backward tick reaches
                                    // t=0, both of which yield a Completed state
                                    let state = if i >= 10 {
                                        TweenState::Completed
                                    } else {
                                        TweenState::Active
                                    };
                                    (
                                        elapsed_ms,
                                        factor,
                                        PlaybackDirection::Forward,
                                        state,
                                        just_completed,
                                    )
                                } else {
                                    let i5 = i % 5;
                                    let just_completed = i5 == 0;

                                    // Inifinite-repeat unclamped value
                                    let elapsed_ms = if playback_direction.is_forward() {
                                        // i    | 1    2    3    4    5    6    7    8    9    10
                                        // t(s) | 0.2  0.4  0.6  0.8  1.0  0.8  0.6  0.4  0.2  0.0
                                        ((i - 5) * 200).rem_euclid(2000) - 1000
                                    } else {
                                        // 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 0.8, 0.6, 0.4, 0.2, 0.0,
                                        // ...
                                        ((i + 5) * 200).rem_euclid(2000) - 1000
                                    }
                                    .abs();

                                    // Now clamp to 'count' repeats
                                    let (elapsed_ms, state) = if i < 10 {
                                        (elapsed_ms, TweenState::Active)
                                    } else {
                                        (0, TweenState::Completed)
                                    };
                                    let ratio = elapsed_ms as f32 / 1000.;

                                    let total_duration_ms = count as i32 * 1000;
                                    let elapsed_ms = if playback_direction.is_forward() {
                                        (i * 200).min(total_duration_ms)
                                    } else {
                                        (total_duration_ms - i * 200).max(0)
                                    };

                                    // Once Completed, the direction doesn't change
                                    let direction = if playback_direction.is_forward() {
                                        //           v [completion]
                                        // 2468X 86420 00
                                        // ffffb bbbbb bb
                                        if i >= 5 {
                                            PlaybackDirection::Backward
                                        } else {
                                            PlaybackDirection::Forward
                                        }
                                    } else {
                                        //           v [completion]
                                        // 86420 2468X XX
                                        // bbbbb fffff ff
                                        if i <= 5 {
                                            PlaybackDirection::Backward
                                        } else {
                                            PlaybackDirection::Forward
                                        }
                                    };

                                    (elapsed_ms, ratio, direction, state, just_completed)
                                }
                            }
                            RepeatCount::Infinite => {
                                if strategy == RepeatStrategy::Repeat {
                                    let just_completed = i % 5 == 0;
                                    let elapsed_ms = if playback_direction.is_forward() {
                                        // 0.2, 0.4, 0.6, 0.8, 0.0, 0.2, 0.4, ...
                                        (i * 200) % 1000
                                    } else {
                                        // 0.8, 0.6, 0.4, 0.2, 0.0, 0.8, 0.6, ...
                                        800 - (((i - 1) * 200 - 1000) % 1000 + 1000) % 1000
                                    };
                                    (
                                        elapsed_ms,
                                        elapsed_ms as f32 / 1000.0,
                                        PlaybackDirection::Forward,
                                        TweenState::Active,
                                        just_completed,
                                    )
                                } else {
                                    let elapsed_ms = if playback_direction.is_forward() {
                                        // 0.2, 0.4, 0.6, 0.8, 1.0, 0.8, 0.6, 0.4, 0.2, 0.0, 0.2,
                                        // 0.4, ...
                                        ((i - 5) * 200).rem_euclid(2000) - 1000
                                    } else {
                                        // 0.8, 0.6, 0.4, 0.2, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 0.8,
                                        // 0.6, ...
                                        (i * 200).rem_euclid(2000) - 1000
                                    }
                                    .abs();
                                    let factor = elapsed_ms as f32 / 1000.0;
                                    let elapsed_ms = if playback_direction.is_forward() {
                                        // 0.2, 0.4, 0.6, 0.8, 1.0, 1.2, 1.4, 1.6, ... (seconds)
                                        (i * 200).rem_euclid(2000)
                                    } else {
                                        // 0.8, 0.6, 0.4, 0.2, 0.0, 1.8, 1.6, 1.4, ... (seconds)
                                        (2000i32 - (i - 5) * 200).rem_euclid(2000)
                                    };
                                    let direction = if playback_direction.is_forward() {
                                        // 2468X 86420 24
                                        // ffffb bbbbf ff
                                        if (i % 10) >= 5 {
                                            PlaybackDirection::Backward
                                        } else {
                                            PlaybackDirection::Forward
                                        }
                                    } else {
                                        // 86420 2468X 86
                                        // bbbbb fffff bb
                                        if ((i - 1) % 10) >= 5 {
                                            PlaybackDirection::Backward
                                        } else {
                                            PlaybackDirection::Forward
                                        }
                                    };
                                    let just_completed = (i % 5) == 0;
                                    (
                                        elapsed_ms,
                                        factor,
                                        direction,
                                        TweenState::Active,
                                        just_completed,
                                    )
                                }
                            }
                            RepeatCount::For(_) => panic!("Untested"),
                        };
                    if playback_direction.is_backward() {
                        direction = !direction;
                    }
                    let expected_translation = Vec3::splat(factor);
                    let elapsed = Duration::from_millis(elapsed_ms as u64);
                    let cycles_completed = match count {
                        RepeatCount::Infinite => elapsed,
                        RepeatCount::Finite(count) => elapsed.min(tween_duration * count),
                        RepeatCount::For(time) => elapsed.min(time),
                    }
                    .div_duration_f64(tween_duration)
                        as u32;
                    println!(
                        "+ Expected: elapsed={:?} factor={} times_completed={} direction={:?} state={:?} just_completed={} translation={:?}",
                        elapsed, factor, cycles_completed, direction, expected_state, just_completed, expected_translation
                    );

                    // Tick the tween
                    let actual_state = manual_tick_component(
                        Entity::PLACEHOLDER, // unused in this test
                        tick_duration,
                        &mut tween,
                        &mut world,
                        entity,
                    );

                    // Check actual values
                    assert_eq!(tween.is_cycle_mirrored(), direction != playback_direction);
                    assert_eq!(actual_state, expected_state);
                    assert_eq!(tween.elapsed(), elapsed);
                    assert_eq!(tween.cycles_completed(), cycles_completed);
                    let transform = world.entity(entity).get::<Transform>().unwrap();
                    assert_approx_eq!(expected_translation, transform.translation, 1e-5);
                    assert_approx_eq!(Quat::IDENTITY, transform.rotation, 1e-5);

                    // Messages are only sent when playing forward
                    if playback_direction.is_forward() {
                        //let component_id = world.component_id::<Transform>().unwrap();
                        let mut event_reader = event_reader_system_state.get_mut(&mut world);
                        let event = event_reader.read().next();
                        if just_completed {
                            assert!(event.is_some());
                            if let Some(event) = event {
                                let AnimTargetKind::Component {
                                    entity: comp_target,
                                } = &event.target
                                else {
                                    panic!("Expected AnimTargetKind::Component");
                                };
                                assert_eq!(*comp_target, entity);
                            }
                        } else {
                            assert!(event.is_none());
                        }
                    }
                }

                // Can't rewind infinite tweens moving backward, they don't have an endpoint
                if tween.total_duration().is_finite()
                    || (playback_direction != PlaybackDirection::Backward)
                {
                    // Rewind
                    println!("+ Rewind");
                    tween.rewind();
                    assert_eq!(tween.playback_direction(), playback_direction); // does not change
                    if playback_direction.is_forward() {
                        assert_eq!(tween.elapsed(), Duration::ZERO);
                        assert_eq!(tween.cycles_completed(), 0);
                    } else {
                        assert_eq!(tween.elapsed(), backward_start_time);
                        let cycles_completed = match count {
                            RepeatCount::Infinite => backward_start_time,
                            RepeatCount::Finite(count) => {
                                backward_start_time.min(tween_duration * count)
                            }
                            RepeatCount::For(time) => backward_start_time.min(time),
                        }
                        .div_duration_f64(tween_duration)
                            as u32;
                        assert_eq!(cycles_completed, tween.cycles_completed());
                    }

                    // Dummy tick to update target
                    let actual_state = manual_tick_component(
                        Entity::PLACEHOLDER, // unused in this test
                        Duration::ZERO,
                        &mut tween,
                        &mut world,
                        entity,
                    );
                    assert_eq!(TweenState::Active, actual_state);
                    let expected_translation = if playback_direction.is_backward()
                        && strategy != RepeatStrategy::MirroredRepeat
                    {
                        Vec3::ONE
                    } else {
                        Vec3::ZERO
                    };
                    let transform = world.entity(entity).get::<Transform>().unwrap();
                    assert_approx_eq!(expected_translation, transform.translation, 1e-5);
                    assert_approx_eq!(Quat::IDENTITY, transform.rotation, 1e-5);
                }

                // Clear event sending
                tween.set_cycle_completed_event(false);
                assert!(!tween.send_cycle_completed_event);
            }
        }
    }

    #[test]
    fn tween_dir() {
        let mut tween = make_test_tween();

        // Default
        assert_eq!(tween.playback_direction(), PlaybackDirection::Forward);
        assert_eq!(tween.elapsed(), Duration::ZERO);

        // no-op
        tween.set_playback_direction(PlaybackDirection::Forward);
        assert_eq!(tween.playback_direction(), PlaybackDirection::Forward);
        assert_eq!(tween.elapsed(), Duration::ZERO);

        // Backward
        tween.set_playback_direction(PlaybackDirection::Backward);
        assert_eq!(tween.playback_direction(), PlaybackDirection::Backward);
        assert_eq!(tween.elapsed(), Duration::ZERO);

        // Elapsed-invariant
        let d300 = Duration::from_millis(300);
        tween.set_playback_direction(PlaybackDirection::Forward);
        tween.set_elapsed(d300);
        assert_eq!(tween.elapsed(), d300);
        tween.set_playback_direction(PlaybackDirection::Backward);
        assert_eq!(tween.elapsed(), d300);

        let (mut world, entity) = make_test_env();

        // Progress always increases alongside the current direction
        tween.set_playback_direction(PlaybackDirection::Backward);
        assert_eq!(tween.elapsed(), d300);
        manual_tick_component(
            Entity::PLACEHOLDER, // unused in this test
            Duration::from_millis(100),
            &mut tween,
            &mut world,
            entity,
        );
        assert_eq!(tween.elapsed(), Duration::from_millis(200)); // 300 - 100
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::splat(0.2), transform.translation, 1e-5);
    }

    #[test]
    fn tween_elapsed() {
        let mut tween = make_test_tween();

        let cycle_duration = tween.cycle_duration();
        let elapsed = tween.elapsed();

        assert_eq!(elapsed, Duration::ZERO);
        assert_eq!(cycle_duration, Duration::from_secs(1));

        for ms in [0, 1, 500, 100, 300, 999, 847, 1000, 900] {
            let elapsed = Duration::from_millis(ms);
            tween.set_elapsed(elapsed);
            assert_eq!(tween.elapsed(), elapsed);

            let times_completed = u32::from(ms == 1000);
            assert_eq!(tween.cycles_completed(), times_completed);
        }
    }

    /// Test ticking a sequence of tweens.
    #[test]
    fn seq_tick() {
        let tween1 = Tween::new(
            EaseMethod::default(),
            Duration::from_secs(1),
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let tween2 = Tween::new(
            EaseMethod::default(),
            Duration::from_secs(1),
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_x(90_f32.to_radians()),
            },
        );
        let mut seq = tween1.then(tween2);

        let (mut world, entity) = make_test_env();

        for i in 1..=16 {
            let state = manual_tick_component(
                Entity::PLACEHOLDER, // unused in this test
                Duration::from_millis(200),
                &mut seq,
                &mut world,
                entity,
            );
            let transform = world.entity(entity).get::<Transform>().unwrap();
            if i < 5 {
                assert_eq!(state, TweenState::Active);
                let r = i as f32 * 0.2;
                assert_approx_eq!(Transform::from_translation(Vec3::splat(r)), *transform);
            } else if i < 10 {
                assert_eq!(state, TweenState::Active);
                let alpha_deg = (18 * (i - 5)) as f32;
                assert_approx_eq!(Vec3::ONE, transform.translation);
                assert_approx_eq!(
                    Quat::from_rotation_x(alpha_deg.to_radians()),
                    transform.rotation
                );
            } else {
                assert_eq!(state, TweenState::Completed);
                assert_approx_eq!(Vec3::ONE, transform.translation);
                assert_approx_eq!(
                    Quat::from_rotation_x(90_f32.to_radians()),
                    transform.rotation
                );
            }
        }
    }

    /// Test crossing tween boundaries in one tick.
    #[test]
    fn seq_tick_boundaries() {
        let mut seq = Sequence::new((0..3).map(|i| {
            Tween::new(
                EaseMethod::default(),
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
                Entity::PLACEHOLDER, // unused in this test
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
                EaseMethod::default(),
                Duration::from_millis(200 * i),
                TransformPositionLens {
                    start: Vec3::ZERO,
                    end: Vec3::ONE,
                },
            )
        }));

        let mut time = Duration::ZERO;
        for i in 1..5 {
            assert_eq!(seq.index(), i - 1);
            assert_eq!(time, seq.elapsed());
            let dt = Duration::from_millis(200 * i as u64);
            assert_eq!(dt, seq.current().cycle_duration());
            time += dt;
            seq.set_elapsed(time);
            assert_eq!(seq.cycles_completed(), u32::from(i == 4));
        }

        seq.rewind();
        assert_eq!(Duration::ZERO, seq.elapsed());
        assert_eq!(0, seq.cycles_completed());
    }

    /// Sequence::from_single()
    #[test]
    fn seq_from_single() {
        let dt = Duration::from_secs(1);
        let tween = Tween::new(
            EaseMethod::default(),
            dt,
            TransformPositionLens {
                start: Vec3::ZERO,
                end: Vec3::ONE,
            },
        );
        let seq = Sequence::from_single(tween);

        assert_eq!(1, seq.tweens.len());
        assert_eq!(dt, seq.cycle_duration());
        assert_eq!(TotalDuration::Finite(dt), seq.total_duration());
    }

    #[test]
    fn seq_elapsed() {
        let mut seq = Sequence::new((1..5).map(|i| {
            Tween::new(
                EaseMethod::default(),
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
            assert_eq!(seq.current().cycle_duration(), duration);
            elapsed += duration;
            seq.set_elapsed(elapsed);
            assert_eq!(seq.cycles_completed(), u32::from(i == 4));
        }
    }

    /// Delay::then()
    #[test]
    fn delay_then() {
        let dt1 = Duration::from_secs(1);
        let dt2 = Duration::from_secs(2);
        let seq: Sequence = Delay::new(dt1).then(Delay::new(dt2));
        assert_eq!(2, seq.tweens.len());
        assert_eq!(dt1 + dt2, seq.cycle_duration());
        assert_eq!(TotalDuration::Finite(dt1 + dt2), seq.total_duration());
        for (i, tweenable) in seq.tweens.iter().enumerate() {
            let dt = Duration::from_secs(i as u64 + 1);
            assert_eq!(dt, tweenable.cycle_duration());
            assert_eq!(TotalDuration::Finite(dt), tweenable.total_duration());
        }
    }

    /// Test ticking a delay.
    #[test]
    fn delay_tick() {
        let total_dt = Duration::from_secs(1);
        let mut delay = Delay::new(total_dt);

        {
            let tweenable: &dyn Tweenable = &delay;
            assert_eq!(total_dt, tweenable.cycle_duration());
            assert_eq!(TotalDuration::Finite(total_dt), tweenable.total_duration());
            assert_eq!(Duration::ZERO, tweenable.elapsed());
        }

        // Dummy world and event writer
        let (mut world, entity) = make_test_env();

        let mut time = Duration::ZERO;
        for i in 1..=6 {
            let dt = Duration::from_millis(200);
            time += dt;
            let state = manual_tick_component(
                Entity::PLACEHOLDER, // unused in this test
                dt,
                &mut delay,
                &mut world,
                entity,
            );

            // Check state
            {
                assert_eq!(state, delay.state());

                let tweenable: &dyn Tweenable = &delay;

                if i < 5 {
                    assert_eq!(state, TweenState::Active);
                    assert!(!delay.is_completed());
                    assert_eq!(0, tweenable.cycles_completed());
                    assert_eq!(dt * i, tweenable.elapsed());
                } else {
                    assert_eq!(state, TweenState::Completed);
                    assert!(delay.is_completed());
                    assert_eq!(1, tweenable.cycles_completed());
                    assert_eq!(total_dt, tweenable.elapsed());
                }
            }
        }

        delay.rewind();
        assert_eq!(0, delay.cycles_completed());
        assert_eq!(Duration::ZERO, delay.elapsed());
        let state: TweenState = manual_tick_component(
            Entity::PLACEHOLDER, // unused in this test
            Duration::ZERO,
            &mut delay,
            &mut world,
            entity,
        );
        assert_eq!(TweenState::Active, state);

        let dt = Duration::from_millis(300);
        delay.set_elapsed(dt);
        assert_eq!(0, delay.cycles_completed());
        assert_eq!(dt, delay.elapsed());
        let dt = Duration::from_millis(1200);
        delay.set_elapsed(dt);
        assert_eq!(1, delay.cycles_completed());
        assert_eq!(total_dt, delay.elapsed());
    }

    #[test]
    fn delay_elapsed() {
        let dt = Duration::from_secs(1);
        let mut delay = Delay::new(dt);
        assert_eq!(dt, delay.cycle_duration());
        assert_eq!(TotalDuration::Finite(dt), delay.total_duration());
        for ms in [0, 1, 500, 100, 300, 999, 847, 1000, 900] {
            let elapsed = Duration::from_millis(ms);
            delay.set_elapsed(elapsed);
            assert_eq!(elapsed, delay.elapsed());

            let times_completed = u32::from(ms == 1000);
            assert_eq!(times_completed, delay.cycles_completed());

            assert_eq!(ms >= 1000, delay.is_completed());
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
        let _ = Delay::new(Duration::ZERO);
    }

    #[test]
    fn tween_repeat() {
        let mut tween = make_test_tween()
            .with_repeat_count(RepeatCount::Finite(5))
            .with_repeat_strategy(RepeatStrategy::Repeat);

        assert_eq!(Duration::ZERO, tween.elapsed());

        let (mut world, entity) = make_test_env();

        let mut time = Duration::ZERO;

        // 10%
        let dt = Duration::from_millis(100);
        time += dt;
        let state = manual_tick_component(
            Entity::PLACEHOLDER, // unused in this test
            dt,
            &mut tween,
            &mut world,
            entity,
        );
        assert_eq!(TweenState::Active, state);
        assert_eq!(0, tween.cycles_completed());
        assert_eq!(time, tween.elapsed());
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::splat(0.1), transform.translation, 1e-5);

        // 130%
        let dt = Duration::from_millis(1200);
        time += dt;
        let state = manual_tick_component(
            Entity::PLACEHOLDER, // unused in this test
            dt,
            &mut tween,
            &mut world,
            entity,
        );
        assert_eq!(TweenState::Active, state);
        assert_eq!(1, tween.cycles_completed());
        assert_eq!(time, tween.elapsed());
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::splat(0.3), transform.translation, 1e-5);

        // 480%
        let dt = Duration::from_millis(3500);
        time += dt;
        let state = manual_tick_component(
            Entity::PLACEHOLDER, // unused in this test
            dt,
            &mut tween,
            &mut world,
            entity,
        );
        assert_eq!(TweenState::Active, state);
        assert_eq!(4, tween.cycles_completed());
        assert_eq!(time, tween.elapsed());
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::splat(0.8), transform.translation, 1e-5);

        // 500% - done
        let dt = Duration::from_millis(200);
        time += dt;
        let state = manual_tick_component(
            Entity::PLACEHOLDER, // unused in this test
            dt,
            &mut tween,
            &mut world,
            entity,
        );
        assert_eq!(TweenState::Completed, state);
        assert_eq!(5, tween.cycles_completed());
        assert_eq!(time, tween.elapsed());
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::ONE, transform.translation, 1e-5);
    }

    #[test]
    fn tween_mirrored_rewind() {
        let mut tween = make_test_tween()
            .with_repeat_count(RepeatCount::Finite(4))
            .with_repeat_strategy(RepeatStrategy::MirroredRepeat);

        assert_eq!(Duration::ZERO, tween.elapsed());

        let (mut world, entity) = make_test_env();

        // 10%
        let dt = Duration::from_millis(100);
        let state = manual_tick_component(
            Entity::PLACEHOLDER, // unused in this test
            dt,
            &mut tween,
            &mut world,
            entity,
        );
        assert_eq!(TweenState::Active, state);
        assert!(!tween.is_cycle_mirrored());
        assert_eq!(0, tween.cycles_completed());
        assert_eq!(dt, tween.elapsed());
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::splat(0.1), transform.translation, 1e-5);

        // rewind
        tween.rewind();
        assert!(!tween.is_cycle_mirrored());
        assert_eq!(0, tween.cycles_completed());
        assert_eq!(Duration::ZERO, tween.elapsed());
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::splat(0.1), transform.translation, 1e-5); // no-op, rewind doesn't apply Lens

        // 120% - mirror
        let dt = Duration::from_millis(1200);
        let state = manual_tick_component(
            Entity::PLACEHOLDER, // unused in this test
            dt,
            &mut tween,
            &mut world,
            entity,
        );
        assert!(tween.is_cycle_mirrored());
        assert_eq!(TweenState::Active, state);
        assert_eq!(1, tween.cycles_completed());
        assert_eq!(dt, tween.elapsed());
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::splat(0.8), transform.translation, 1e-5);

        // rewind
        assert!(tween.is_cycle_mirrored());
        tween.rewind();
        assert!(!tween.is_cycle_mirrored()); //restored
        assert_eq!(0, tween.cycles_completed());
        assert_eq!(Duration::ZERO, tween.elapsed());
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::splat(0.8), transform.translation, 1e-5); // no-op, rewind doesn't apply Lens

        // 400% - done mirror (because Completed freezes the state)
        let dt = Duration::from_millis(4000);
        let state = manual_tick_component(
            Entity::PLACEHOLDER, // unused in this test
            dt,
            &mut tween,
            &mut world,
            entity,
        );
        assert_eq!(TweenState::Completed, state);
        assert!(tween.is_cycle_mirrored()); // frozen from last loop
        assert_eq!(4, tween.cycles_completed());
        assert_eq!(dt, tween.elapsed()); // Completed
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::ZERO, transform.translation, 1e-5);

        // rewind
        assert!(tween.is_cycle_mirrored());
        tween.rewind();
        assert!(!tween.is_cycle_mirrored()); // restored
        assert_eq!(0, tween.cycles_completed());
        assert_eq!(Duration::ZERO, tween.elapsed());
        let transform = world.entity(entity).get::<Transform>().unwrap();
        assert_approx_eq!(Vec3::ZERO, transform.translation, 1e-5); // no-op, rewind doesn't apply Lens
    }
}
