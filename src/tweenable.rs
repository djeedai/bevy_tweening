use bevy::prelude::*;
use std::time::Duration;

use crate::{EaseMethod, Lens, TweenState, TweeningDirection, TweeningType};

/// An animatable entity, either a single [`Tween`] or a collection of them.
pub trait Tweenable<T>: Send + Sync {
    /// Get the total duration of the animation.
    ///
    /// For [`TweeningType::PingPong`], this is the duration of a single way, either from start
    /// to end or back from end to start. The total loop duration start -> end -> start in this
    /// case is the double of the returned value.
    fn duration(&self) -> Duration;

    /// Get the current progress in \[0:1\] (non-looping) or \[0:1\[ (looping) of the animation.
    ///
    /// For looping animations, this reports the progress of the current iteration,
    /// in the current direction:
    /// - [`TweeningType::Loop`] is 0 at start and 1 at end. The exact value 1.0 is never reached,
    ///   since the tweenable loops over to 0.0 immediately.
    /// - [`TweeningType::PingPong`] is 0 at the source endpoint and 1 and the destination one,
    ///   which are respectively the start/end for [`TweeningDirection::Forward`], or the end/start
    ///   for [`TweeningDirection::Backward`]. The exact value 1.0 is never reached, since the tweenable
    ///   loops over to 0.0 immediately when it changes direction at either endpoint.
    fn progress(&self) -> f32;

    /// Tick the animation, advancing it by the given delta time and mutating the
    /// given target component or asset
    ///
    /// This changes the tweenable state to [`TweenState::Running`] before updating it.
    /// If the tick brings the tweenable to its end, the state changes to [`TweenState::Ended`].
    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState;

    /// Stop the animation. This changes its state to [`TweenState::Stopped`].
    fn stop(&mut self);
}

impl<T> Tweenable<T> for Box<dyn Tweenable<T> + Send + Sync + 'static> {
    fn duration(&self) -> Duration {
        self.as_ref().duration()
    }
    fn progress(&self) -> f32 {
        self.as_ref().progress()
    }
    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState {
        self.as_mut().tick(delta, target)
    }
    fn stop(&mut self) {
        self.as_mut().stop()
    }
}

/// Trait for boxing a [`Tweenable`] trait object.
pub trait IntoBoxDynTweenable<T> {
    /// Convert the current object into a boxed [`Tweenable`].
    fn into_box_dyn(this: Self) -> Box<dyn Tweenable<T> + Send + Sync + 'static>;
}

impl<T, U: Tweenable<T> + Send + Sync + 'static> IntoBoxDynTweenable<T> for U {
    fn into_box_dyn(this: U) -> Box<dyn Tweenable<T> + Send + Sync + 'static> {
        Box::new(this)
    }
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

impl<T: 'static> Tween<T> {
    /// Chain another [`Tweenable`] after this tween, making a sequence with the two.
    pub fn then(self, tween: impl Tweenable<T> + Send + Sync + 'static) -> Sequence<T> {
        Sequence::from_single(self).then(tween)
    }
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
}

impl<T> Tweenable<T> for Tween<T> {
    fn duration(&self) -> Duration {
        self.timer.duration()
    }

    /// Current animation progress ratio between 0 and 1.
    ///
    /// For reversed playback ([`TweeningDirection::Backward`]), the ratio goes from 0 at the
    /// end point (beginning of backward playback) to 1 at the start point (end of backward
    /// playback).
    fn progress(&self) -> f32 {
        match self.direction {
            TweeningDirection::Forward => self.timer.percent(),
            TweeningDirection::Backward => self.timer.percent_left(),
        }
    }

    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState {
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
            if self.tweening_type == TweeningType::Once {
                self.state = TweenState::Ended;
            }

            // This is always true for non ping-pong, and is true for ping-pong when
            // coming back to start after a full cycle start -> end -> start.
            if self.direction == TweeningDirection::Forward {
                if let Some(cb) = &mut self.on_ended {
                    cb();
                }
            }
        }

        self.state
    }

    fn stop(&mut self) {
        self.state = TweenState::Stopped;
        self.timer.reset();
    }
}

/// A sequence of tweens played back in order one after the other.
pub struct Sequence<T> {
    tweens: Vec<Box<dyn Tweenable<T> + Send + Sync + 'static>>,
    index: usize,
    state: TweenState,
    duration: Duration,
    time: Duration,
}

impl<T> Sequence<T> {
    /// Create a new sequence of tweens.
    pub fn new(items: impl IntoIterator<Item = impl IntoBoxDynTweenable<T>>) -> Self {
        let tweens: Vec<_> = items
            .into_iter()
            .map(IntoBoxDynTweenable::into_box_dyn)
            .collect();
        let duration = tweens.iter().map(|t| t.duration()).sum();
        Sequence {
            tweens,
            index: 0,
            state: TweenState::Stopped,
            duration,
            time: Duration::from_secs(0),
        }
    }

    /// Create a new sequence containing a single tween.
    pub fn from_single(tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        let duration = tween.duration();
        Sequence {
            tweens: vec![Box::new(tween)],
            index: 0,
            state: TweenState::Stopped,
            duration,
            time: Duration::from_secs(0),
        }
    }

    /// Append a [`Tweenable`] to this sequence.
    pub fn then(mut self, tween: impl Tweenable<T> + Send + Sync + 'static) -> Self {
        self.duration += tween.duration();
        self.tweens.push(Box::new(tween));
        self
    }

    /// Index of the current active tween in the sequence.
    pub fn index(&self) -> usize {
        self.index.min(self.tweens.len() - 1)
    }

    /// Get the current active tween in the sequence.
    pub fn current(&self) -> &dyn Tweenable<T> {
        self.tweens[self.index()].as_ref()
    }
}

impl<T> Tweenable<T> for Sequence<T> {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn progress(&self) -> f32 {
        self.time.as_secs_f32() / self.duration.as_secs_f32()
    }

    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState {
        if self.index < self.tweens.len() {
            self.state = TweenState::Running;
            self.time = (self.time + delta).min(self.duration);
            let tween = &mut self.tweens[self.index];
            let state = tween.tick(delta, target);
            if state == TweenState::Ended {
                tween.stop();
                self.index += 1;
                if self.index >= self.tweens.len() {
                    self.state = TweenState::Ended;
                }
            }
        }
        self.state
    }

    fn stop(&mut self) {
        if self.state != TweenState::Stopped {
            self.state = TweenState::Stopped;
            if self.index < self.tweens.len() {
                let tween = &mut self.tweens[self.index];
                tween.stop();
            }
        }
    }
}

/// A collection of [`Tweenable`] executing in parallel.
pub struct Tracks<T> {
    tracks: Vec<Box<dyn Tweenable<T> + Send + Sync + 'static>>,
    duration: Duration,
    time: Duration,
}

impl<T> Tracks<T> {
    /// Create a new [`Tracks`] from an iterator over a collection of [`Tweenable`].
    pub fn new(items: impl IntoIterator<Item = impl IntoBoxDynTweenable<T>>) -> Self {
        let tracks: Vec<_> = items
            .into_iter()
            .map(IntoBoxDynTweenable::into_box_dyn)
            .collect();
        let duration = tracks.iter().map(|t| t.duration()).max().unwrap();
        Tracks {
            tracks,
            duration,
            time: Duration::from_secs(0),
        }
    }
}

impl<T> Tweenable<T> for Tracks<T> {
    fn duration(&self) -> Duration {
        self.duration
    }

    fn progress(&self) -> f32 {
        self.time.as_secs_f32() / self.duration.as_secs_f32()
    }

    fn tick(&mut self, delta: Duration, target: &mut T) -> TweenState {
        let mut any_running = false;
        for tweenable in &mut self.tracks {
            let state = tweenable.tick(delta, target);
            any_running = any_running || (state == TweenState::Running);
        }
        if any_running {
            self.time = (self.time + delta).min(self.duration);
            TweenState::Running
        } else {
            TweenState::Ended
        }
    }

    fn stop(&mut self) {
        for seq in &mut self.tracks {
            seq.stop();
        }
    }
}

/// A time delay that doesn't animate anything.
///
/// This is generally useful for combining with other tweenables into sequences and tracks,
/// for example to delay the start of a tween in a track relative to another track. The `menu`
/// example (`examples/menu.rs`) uses this technique to delay the animation of its buttons.
pub struct Delay {
    timer: Timer,
}

impl Delay {
    /// Create a new [`Delay`] with a given duration.
    pub fn new(duration: Duration) -> Self {
        Delay {
            timer: Timer::new(duration, false),
        }
    }

    /// Chain another [`Tweenable`] after this tween, making a sequence with the two.
    pub fn then<T>(self, tween: impl Tweenable<T> + Send + Sync + 'static) -> Sequence<T> {
        Sequence::from_single(self).then(tween)
    }
}

impl<T> Tweenable<T> for Delay {
    fn duration(&self) -> Duration {
        self.timer.duration()
    }

    fn progress(&self) -> f32 {
        self.timer.percent()
    }

    fn tick(&mut self, delta: Duration, _: &mut T) -> TweenState {
        self.timer.tick(delta);
        if self.timer.finished() {
            TweenState::Ended
        } else {
            TweenState::Running
        }
    }

    fn stop(&mut self) {
        self.timer.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TransformPositionLens, TransformRotationLens};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

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
                let (ratio, ec, dir, expected_state) = match tweening_type {
                    TweeningType::Once => {
                        let r = (i as f32 * 0.2).min(1.0);
                        let ec = if i >= 5 { 1 } else { 0 };
                        let state = if i >= 5 {
                            TweenState::Ended
                        } else {
                            TweenState::Running
                        };
                        (r, ec, TweeningDirection::Forward, state)
                    }
                    TweeningType::Loop => {
                        let r = (i as f32 * 0.2).fract();
                        let ec = i / 5;
                        (r, ec, TweeningDirection::Forward, TweenState::Running)
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
                        (r, ec, dir, TweenState::Running)
                    }
                };
                println!(
                    "Expected; r={} ec={} dir={:?} state={:?}",
                    ratio, ec, dir, expected_state
                );

                // Tick the tween
                let actual_state = tween.tick(tick_duration, &mut transform);

                // Check actual values
                assert_eq!(tween.direction(), dir);
                assert_eq!(actual_state, expected_state);
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
        let mut seq = tween1.then(tween2);
        let mut transform = Transform::default();
        for i in 1..=16 {
            let state = seq.tick(Duration::from_secs_f32(0.2), &mut transform);
            if i < 5 {
                assert_eq!(state, TweenState::Running);
                let r = i as f32 * 0.2;
                assert_eq!(transform, Transform::from_translation(Vec3::splat(r)));
            } else if i < 10 {
                assert_eq!(state, TweenState::Running);
                let alpha_deg = (36 * (i - 5)) as f32;
                assert!(transform.translation.abs_diff_eq(Vec3::splat(1.), 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(alpha_deg.to_radians()), 1e-5));
            } else {
                assert_eq!(state, TweenState::Ended);
                assert!(transform.translation.abs_diff_eq(Vec3::splat(1.), 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(180_f32.to_radians()), 1e-5));
            }
        }
    }

    /// Test ticking parallel tracks of tweens.
    #[test]
    fn tracks_tick() {
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
            Duration::from_secs_f32(0.8), // shorter
            TransformRotationLens {
                start: Quat::IDENTITY,
                end: Quat::from_rotation_x(180_f32.to_radians()),
            },
        );
        let mut tracks = Tracks::new([tween1, tween2]);
        let mut transform = Transform::default();
        for i in 1..=6 {
            let state = tracks.tick(Duration::from_secs_f32(0.2), &mut transform);
            if i < 5 {
                assert_eq!(state, TweenState::Running);
                let r = i as f32 * 0.2;
                let alpha_deg = (45 * i) as f32;
                assert!(transform.translation.abs_diff_eq(Vec3::splat(r), 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(alpha_deg.to_radians()), 1e-5));
            } else {
                assert_eq!(state, TweenState::Ended);
                assert!(transform.translation.abs_diff_eq(Vec3::splat(1.), 1e-5));
                assert!(transform
                    .rotation
                    .abs_diff_eq(Quat::from_rotation_x(180_f32.to_radians()), 1e-5));
            }
        }
    }
}
