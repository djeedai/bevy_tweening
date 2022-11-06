# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `RepeatCount` and `RepeatStrategy` for more granular control over animation looping.
- Added `with_repeat_count()` and `with_repeat_strategy()` builder methods to `Tween<T>`.
- Added a `speed()` getter on `Animator<T>` and `AssetAnimator<T>`.
- Added `set_elapsed(Duration)` and `elapsed() -> Duration` to the `Tweenable<T>` trait. Those methods are preferable over `set_progress()` and `progress()` as they avoid the conversion to floating-point values and any rounding errors.
- Added a new `bevy_text` feature for `Text`-related built-in lenses.

### Changed

- Removed the `tweening_type` parameter from the signature of `Tween<T>::new()`; use `with_repeat_count()` and `with_repeat_strategy()` instead.
- Animators now always have a tween (instead of it being optional). This means the default animator implementation was removed.
- `Delay::new()` now panics if the `duration` is zero. This prevents creating no-op `Delay` objects, and avoids an internal edge case producing wrong results.
- Tweens moving to `TweenState::Completed` are now guaranteed to freeze their state. In particular, this means that their direction will not flip at the end of the last loop if their repeat strategy is `RepeatStrategy::MirroredRepeat`.
- Moved the `TextColorLens` lens from the `bevy_ui` feature to the new `bevy_text` one, to allow using it without the Bevy UI crate.

### Removed

- Removed `Tweenable::is_looping()`, which was not implemented for most tweenables.
- Removed `TweeningType` in favor of `RepeatCount` and `RepeatStrategy`.

### Fixed

- Fixed the animator speed feature, which got broken in #44.

## [0.5.0] - 2022-08-04

### Added

- Added `is_forward()` and `is_backward()` convenience helpers to `TweeningDirection`.
- Added `Tween::set_direction()` and `Tween::with_direction()` which allow configuring the playback direction of a tween, allowing to play it backward from end to start.
- Added support for dynamically changing an animation's speed with `Animator::set_speed`.
- Added `AnimationSystem` label to tweening tick systems.
- Added a `BoxedTweenable` trait to make working with `Box<dyn Tweenable + ...>` easier.

### Changed

- Compatible with Bevy 0.8
- Double boxing in `Sequence` and `Tracks` was fixed. As a result, any custom tweenables
  should implement `From` for `BoxedTweenable` to make those APIs easier to use.

## [0.4.0] - 2022-04-16

### Changed

- Compatible with Bevy 0.7
- Better dependencies: Introduced features `bevy_sprite` and `bevy_ui` taking a dependency on the same-named crates of Bevy, and removed the forced dependency on `bevy/render`. The new features are enabled by default, for discoverability, and only impact the prebuilt lenses. The library now builds without any Bevy optional feature.

## [0.3.3] - 2022-03-05

### Added

- Added new built-in rotation lenses based on angle interpolation, to allow rotation animations larger than a half turn:
  - `TransformRotateXLens`
  - `TransformRotateYLens`
  - `TransformRotateZLens`
  - `TransformRotateAxisLens`

## [0.3.2] - 2022-02-24

### Added

- Implemented `Default` for `TweeningType` (= `Once`), `EaseMethod` (= `Linear`), `TweeningDirection` (= `Forward`).
- Added `Tweenable::is_looping()`, `Tweenable::set_progress()`, `Tweenable::times_completed()`, and `Tweenable::rewind()`.
- Added `Animator::set_progress()`, `Animator::progress()`, `Animator::stop()`, and `Animator::rewind()`.
- Added `AssetAnimator::set_progress()`, `AssetAnimator::progress()`, `AssetAnimator::stop()`, and `AssetAnimator::rewind()`.
- Added the `TweenCompleted` event, raised when a `Tween<T>` completed its animation if that feature was previously activated with `set_completed_event()` or `with_completed_event()`.

### Changed

- `TweenState` now contains only two states: `Active` and `Completed`. Looping animations are always active, and non-looping ones are completed once they reach their end point.
- Merged the `started` and `ended` callbacks into a `completed` one (`Tween::set_completed()` and `Tween::clear_completed()`), which is invoked when the tween completes a single iteration. That is, for non-looping animations, when `TweenState::Completed` is reached. And for looping animations, once per iteration (going from start -> end, or from end -> start).

### Removed

- Removed `Tweenable::stop()`. Tweenables do not have a "stop" state anymore, they are only either active or completed. The playback state is only relevant on the `Animator` or `AssetAnimator` which controls them.

### Fixed

- Fixed a bug with the alpha value of colored lenses being too large (`TextColorLens`, `SpriteColorLens`, `ColorMaterialColorLens`).

## [0.3.1] - 2022-02-12

### Added

- Add user callbacks on tween started (`Tween::set_started`) and ended (`Tween::set_ended`).
- Implement `Default` for `AnimatorState` as `AnimatorState::Playing`.
- Added `Animator::with_state()` and `AssetAnimator::with_state()`, builder-like functions to override the default `AnimatorState`.
- Added `Tween::is_looping()` returning true for all but `TweeningType::Once`.
- Added the `Tweenable<T>` trait, implemented by the `Tween<T>` and `Delay<T>` animation, and by the `Tracks<T>` and `Sequence<T>` animation collections.
- Added `IntoBoxDynTweenable<T>`, a trait to convert a `Tweenable<T>` trait object into a boxed variant.
- Publicly exposed `Sequence<T>`, a sequence of `Tweenable<T>` running one after the other.
- Publicly exposed `Tracks<T>`, a collection of `Tweenable<T>` running in parallel.
- Publicly exposed `TweenState`, the playback state of a single `Tweenable<T>` item.
- Added `Tween<T>::then()` and `Sequence<T>::then()` to append a `Tweenable<T>` to a sequence (creating a new sequence in the case of `Tween<T>::then()`).
- Added `tweenable()` and `tweenable_mut()` on the `Animator<T>` and `AssetAnimator<T>` to access their top-level `Tweenable<T>`.
- Implemented `Default` for `Animator<T>` and `AssetAnimator<T>`, creating an animator without any tweenable item (no-op).
- Added `Delay` tweenable for a time delay between other tweens.
- Added a new `menu` example demonstrating in particular the `Delay` tweenable.

### Changed

- Moved tween duration out of the `TweeningType` enum, which combined with the removal of the "pause" feature in loops makes it a C-like enum.
- The `Sequence<T>` progress now reports the progress of the total sequence. Individual sub-tweenables cannot be accessed.
- Updated the `sequence` example to add some text showing the current sequence progress.
- Modified the signature of `new()` for `Animator<T>` and `AssetAnimator<T>` to take a single `Tweenable<T>` instead of trying to build a `Tween<T>` internally. This allows passing any `Tweenable<T>` as the top-level animatable item of an animator, and avoids the overhead of maintaining a `Tracks<T>` internally in each animator when the most common use case is likely to use a single `Tween<T>` or a `Sequence<T>` without parallelism.

### Removed

- Removed the "pause" feature in-between loops of `TweeningType::Loop` and `TweeningType::PingPong`, which can be replaced if needed by a sequence including a `Delay` tweenable. Removed `Tween::is_paused()`.
- Removed `new_single()` and `new_seq()` on the `Animator<T>` and `AssetAnimator<T>`. Users should explicitly create a `Tween<T>` or `Sequence<T>` instead, and use `new()`.

### Fixed

- Fix missing public export of `component_animator_system()` and `asset_animator_system()` preventing the animation of all but the built-in items.

## [0.3.0] - 2022-01-28

### Added

- Add `Tween<T>` describing a single tween animation, independently of its target (asset or component).
- Add `Tween<T>::is_paused()` to query when a tweening animation is in its pause phase, if any.
- Add `Tween<T>::direction()` to query the playback direction of a tweening animation (forward or backward).
- Add `Tween<T>::progress()` to query the progres ratio in [0:1] of a tweening animation.
- Enable multiple lenses per animator via "tracks", the ability to add multiple tween animations running in parallel on the same component.
- Enable sequences of tween animations running serially, one after the other, for each track of an animator, allowing to create more complex animations.

### Fixed

- Perform spherical linear interpolation (slerp) for `Quat` rotation of `Transform` animation via `TransformRotationLens`, instead of mere linear interpolation leaving the quaternion non-normalized.

## [0.2.0] - 2022-01-09

### Changed

- Update to Bevy 0.6
- Update to rust edition 2021
- Force Cargo resolver v2

### Added

- Added built-in lens `SpriteColorLens`, since the color of a `Sprite` is now an intrinsic property of the component in Bevy 0.6, and does not use `ColorMaterial` anymore.

## [0.1.0] - 2021-12-24

Initial version for Bevy 0.5
