# 🍃 Bevy Tweening 0.14 migration guide

This migration guide describes the major changes to the 🍃 Bevy Tweening API
operated in v0.14, and how to upgrade code using previous versions.

## Semantic

The new version 0.14 introduces new and/or clarified semantics for many concepts.

### Animation timeline and `Duration`

In previous verisons, the mental model for time related quantities was often poorly defined,
and led to various confusions and edge cases.

In v0.14, we removed all references to "progress", which was a floating-point value
loosely defined as the fraction of the animation already performed, over its total duration.
Instead, all time-related operations are performed directly with the `Duration` Rust type.

Users should now adopt the following mental model for animation time:

- Each animation has a _timeline_ starting at `t=0` and extending to the _total duration_
  of the animation, which can be infinite if _e.g._ the animation loops forever.
  The timeline is a concept to help build a mental model; there's no timeline type in code.
- The animation is composed of one or more _cycles_, which repeat eith a given number of times,
  or until a duration is elapsed. This is configured by the `RepeatCount` type.
- The _elapsed_ time of an animation is the position on its timeline. It ranges from `0`
  to its total duration (except for infinitely repeating animations; see details below).
- In general, cycles are repeated identically. However, as an alternative, the user can configure
  the _repeat strategy_ to mirror every other cycle. Mirrored cycles animate their target
  in reverse order. However, note that **the elapsed time value is unaffected by mirroring,
  and continues to be expressed as an absolute time position** on the animation timeline.
  This is a major semantic change compared to previous versions, and greatly clarifies
  the implementation and the timeline mental model.

Standard (non-mirrored) repeating of cycles:

```txt
  ratio           cycle duration
  ^                   |<->|
1 |  /   /   /   /   /|  /|  /   /   /   /
  | /   /   /   /   / | / | /   /   /   /
  |/   /   /   /   /  |/  |/   /   /   /   /
0 *-----------------------------------------*------> timeline
  0     <------ total duration ------>      |
```

Mirrored repeating of cycles:

```txt
  ratio          cycle duration
  ^                  |<->|
1 |  /\    /\    /\  |  /|\    /\    /\    /\
  | /  \  /  \  /  \ | / | \  /  \  /  \  /  
  |/    \/    \/    \|/  |  \/    \/    \/   
0 *-----------------------------------------*-----> timeline
  0     <------ total duration ------>      |
```

Note that **the cycle duration is half the duration of the "loop" formed by the mirroring**.
For this reason, 🍃 Bevy Tweening now avoids using the term "loop", to prevent confusion.

In the above, the `ratio` is a value in `[0:1]` which gets passed to the easing function.
The output is then fed to `Lens::lerp()` to calculate the animation target state.
When using a linear easing function, `ratio` is exactly the lerp fraction.

Consequence of the above, functions like `Tweenable::set_elapsed()` operate on the timeline,
using absolute duration. This means that calling `set_elapsed(2.5s)` on an animation composed
of 3 cycles of 1s each effectively puts the animation position at half of the third cycle.

### Playback direction

Related to the animation timeline and absolute duration position,
the _playback direction_ determines whether calls to `Tweenable::step()` move the animation
time forward or backward **on the animation timeline**.

**This means the playback direction is completely unrelated to cycles mirroring.**

This is a major clarification over the confusing semantic of previous versions, which tended
to mix the two concepts and produced often unexpected results.

### Completion events and completed state

_Completion events_ are emitted when an animation complete a cycle.

To prevent any confusion, the reversed playback direction is considerd to be an editing tool,
not typical of normal use in a game or other application, and therefore **completion events are
never emitted in reverse playback**.

Related but distinct, the _completion state_ of the animation determines if the animation
can continue playback or not. The animation is said to be completed when its elapsed time
reaches its total duration, at the end of its timeline. Infinite animations never complete.
