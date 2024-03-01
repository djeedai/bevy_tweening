# 🍃 Bevy Tweening

[![License: MIT/Apache](https://img.shields.io/badge/License-MIT%20or%20Apache2-blue.svg)](https://opensource.org/licenses/MIT)
[![Doc](https://docs.rs/bevy_tweening/badge.svg)](https://docs.rs/bevy_tweening)
[![Crate](https://img.shields.io/crates/v/bevy_tweening.svg)](https://crates.io/crates/bevy_tweening)
[![Build Status](https://github.com/djeedai/bevy_tweening/actions/workflows/ci.yaml/badge.svg)](https://github.com/djeedai/bevy_tweening/actions/workflows/ci.yaml)
[![Coverage Status](https://coveralls.io/repos/github/djeedai/bevy_tweening/badge.svg?branch=main&kill_cache=1)](https://coveralls.io/github/djeedai/bevy_tweening?branch=main)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-v0.13-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

Tweening animation plugin for the Bevy game engine.

## Features

- [x] Animate any field of any component or asset, including custom ones.
- [x] Run multiple tweens (animations) per component/asset in parallel.
- [x] Chain multiple tweens (animations) one after the other for complex animations.
- [x] Raise a Bevy event or invoke a callback when an tween completed.

## Usage

### Dependency

Add to `Cargo.toml`:

```toml
[dependencies]
bevy_tweening = "0.10"
```

This crate supports the following features:

| Feature | Default | Description |
|---|---|---|
| `bevy_asset`  | Yes | Enable animating Bevy assets (`Asset`) in addition of components. |
| `bevy_sprite` | Yes | Includes built-in lenses for some `Sprite`-related components. |
| `bevy_ui`     | Yes | Includes built-in lenses for some UI-related components. |
| `bevy_text`   | Yes | Includes built-in lenses for some `Text`-related components. |

### System setup

Add the `TweeningPlugin` to your app:

```rust
App::default()
    .add_plugins(DefaultPlugins)
    .add_plugins(TweeningPlugin)
    .run();
```

This provides the basic setup for using 🍃 Bevy Tweening. However, additional setup is required depending on the components and assets you want to animate:

- To ensure a component `C` is animated, the `component_animator_system::<C>` system must run each frame, in addition of adding an `Animator::<C>` component to the same Entity as `C`.

- To ensure an asset `A` is animated, the `asset_animator_system::<A>` system must run each frame, in addition of adding an `AssetAnimator<A>` component to any Entity. Animating assets also requires the `bevy_asset` feature (enabled by default).

By default, 🍃 Bevy Tweening adopts a minimalist approach, and the `TweeningPlugin` will only add systems to animate components and assets for which a `Lens` is provided by 🍃 Bevy Tweening itself. This means that any other Bevy component or asset (either built-in from Bevy itself, or custom) requires manually scheduling the appropriate system.

| Component or Asset | Animation system added by `TweeningPlugin`? |
|---|---|
| `Transform`          | Yes                           |
| `Sprite`             | Only if `bevy_sprite` feature |
| `ColorMaterial`      | Only if `bevy_sprite` feature |
| `Style`              | Only if `bevy_ui` feature     |
| `Text`               | Only if `bevy_text` feature   |
| All other components | No                            |

To add a system for a component `C`, use:

```rust
app.add_systems(Update, component_animator_system::<C>.in_set(AnimationSystem::AnimationUpdate));
```

Similarly for an asset `A`, use:

```rust
app.add_systems(Update, asset_animator_system::<A>.in_set(AnimationSystem::AnimationUpdate));
```

### Animate a component

Animate the transform position of an entity by creating a `Tween` animation for the transform, and adding an `Animator` component with that tween:

```rust
// Create a single animation (tween) to move an entity.
let tween = Tween::new(
    // Use a quadratic easing on both endpoints.
    EaseFunction::QuadraticInOut,
    // Animation time (one way only; for ping-pong it takes 2 seconds
    // to come back to start).
    Duration::from_secs(1),
    // The lens gives the Animator access to the Transform component,
    // to animate it. It also contains the start and end values associated
    // with the animation ratios 0. and 1.
    TransformPositionLens {
        start: Vec3::ZERO,
        end: Vec3::new(1., 2., -4.),
    },
)
// Repeat twice (one per way)
.with_repeat_count(RepeatCount::Finite(2))
// After each iteration, reverse direction (ping-pong)
.with_repeat_strategy(RepeatStrategy::MirroredRepeat);

commands.spawn((
    // Spawn a Sprite entity to animate the position of.
    SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
        ..default()
    },
    // Add an Animator component to control and execute the animation.
    Animator::new(tween),
));
```

### Chaining animations

Bevy Tweening supports several types of _tweenables_, building blocks that can be combined to form complex animations. A tweenable is a type implementing the `Tweenable<T>` trait.

- **`Tween`** - A simple tween (easing) animation between two values.
- **`Sequence`** - A series of tweenables executing in series, one after the other.
- **`Tracks`** - A collection of tweenables executing in parallel.
- **`Delay`** - A time delay.

Most tweenables can be chained with the `then()` operator:

```rust
// Produce a sequence executing 'tween1' then 'tween2'
let tween1 = Tween { [...] }
let tween2 = Tween { [...] }
let seq = tween1.then(tween2);
```

## Predefined Lenses

A small number of predefined lenses are available for the most common use cases, which also serve as examples. **Users are encouraged to write their own lens to tailor the animation to their use case.**

The naming scheme for predefined lenses is `"<TargetName><FieldName>Lens"`, where `<TargetName>` is the name of the target Bevy component or asset type which is queried by the internal animation system to be modified, and `<FieldName>` is the field which is mutated in place by the lens. All predefined lenses modify a single field. Custom lenses can be written which modify multiple fields at once.

### Bevy Components

| Target Component | Animated Field | Lens | Feature |
|---|---|---|---|
| [`Transform`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.Transform.html) | [`translation`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.Transform.html#structfield.translation) | [`TransformPositionLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.TransformPositionLens.html) | |
| | [`rotation`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.Transform.html#structfield.rotation) (`Quat`)¹ | [`TransformRotationLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.TransformRotationLens.html) | |
| | [`rotation`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.Transform.html#structfield.rotation) (angle)² | [`TransformRotateXLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.TransformRotateXLens.html) | |
| | [`rotation`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.Transform.html#structfield.rotation) (angle)² | [`TransformRotateYLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.TransformRotateYLens.html) | |
| | [`rotation`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.Transform.html#structfield.rotation) (angle)² | [`TransformRotateZLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.TransformRotateZLens.html) | |
| | [`rotation`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.Transform.html#structfield.rotation) (angle)² | [`TransformRotateAxisLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.TransformRotateAxisLens.html) | |
| | [`scale`](https://docs.rs/bevy/0.12.0/bevy/transform/components/struct.Transform.html#structfield.scale) | [`TransformScaleLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.TransformScaleLens.html) | |
| [`Sprite`](https://docs.rs/bevy/0.12.0/bevy/sprite/struct.Sprite.html) | [`color`](https://docs.rs/bevy/0.12.0/bevy/sprite/struct.Sprite.html#structfield.color) | [`SpriteColorLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.SpriteColorLens.html) | `bevy_sprite` |
| [`Style`](https://docs.rs/bevy/0.12.0/bevy/ui/struct.Style.html) | [`position`](https://docs.rs/bevy/0.12.0/bevy/ui/struct.Style.html#structfield.position) | [`UiPositionLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.UiPositionLens.html) | `bevy_ui` |
| [`BackgroundColor`](https://docs.rs/bevy/0.12.0/bevy/ui/struct.BackgroundColor.html)| | [`UiBackgroundColorLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.UiBackgroundColorLens.html) | `bevy_ui` |
| [`Text`](https://docs.rs/bevy/0.12.0/bevy/text/struct.Text.html) | [`TextStyle::color`](https://docs.rs/bevy/0.12.0/bevy/text/struct.TextStyle.html#structfield.color) | [`TextColorLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.TextColorLens.html) | `bevy_text` |

¹ Shortest-path interpolation between two rotations, using `Quat::slerp()`.

² Angle-based interpolation, valid for rotations over ½ turn.

See the [comparison of rotation lenses](https://docs.rs/bevy_tweening/0.7.0/bevy_tweening/lens/index.html#rotations) for details.

### Bevy Assets

Asset animation always requires the `bevy_asset` feature.

| Target Asset | Animated Field | Lens | Feature |
|---|---|---|---|
| [`ColorMaterial`](https://docs.rs/bevy/0.12.0/bevy/sprite/struct.ColorMaterial.html) | [`color`](https://docs.rs/bevy/0.12.0/bevy/sprite/struct.ColorMaterial.html#structfield.color) | [`ColorMaterialColorLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/lens/struct.ColorMaterialColorLens.html) | `bevy_asset` + `bevy_sprite` |

## Custom lens

A custom lens allows animating any field or group of fields of a Bevy component or asset. A custom lens is a type implementing the `Lens` trait, which is generic over the type of component or asset.

```rust
struct MyXAxisLens {
    start: f32,
    end: f32,
}

impl Lens<Tranform> for MyXAxisLens {
    fn lerp(&self, target: &mut Tranform, ratio: f32) -> f32 {
        let start = Vec3::new(self.start, 0., 0.);
        let end = Vec3::new(self.end, 0., 0.);
        target.translation = start + (end - start) * ratio;
    }
}
```

Note that the lens always **linearly** interpolates the field(s) of the component or asset. The type of easing applied modifies the rate at which the `ratio` parameter evolves, and is applied before the `lerp()` function is invoked.

The basic formula for lerp (linear interpolation) is either of:

- `start + (end - start) * scalar`
- `start * (1.0 - scalar) + end * scalar`

The two formulations are mathematically equivalent, but one may be more suited than the other depending on the type interpolated and the operations available, and the potential floating-point precision errors.

## Custom component support

Custom components are animated via a lens like the ones described in [Bevy Components](#bevy-components).

```rust
#[derive(Component)]
struct MyCustomComponent(f32);

struct MyCustomLens {
    start: f32,
    end: f32,
}

impl Lens<MyCustomComponent> for MyCustomLens {
    fn lerp(&self, target: &mut MyCustomComponent, ratio: f32) -> f32 {
        target.0 = self.start + (self.end - self.start) * ratio;
    }
}
```

Then, in addition, the system `component_animator_system::<CustomComponent>` needs to be added to the application, as described in [System Setup](#system-setup). This system will extract each frame all `CustomComponent` instances with an `Animator<CustomComponent>` on the same entity, and animate the component via its animator.

## Custom asset support

The process is similar to custom components, creating a custom lens for the custom asset. The system to add is `asset_animator_system::<CustomAsset>`, as described in [System Setup](#system-setup). This requires the `bevy_asset` feature (enabled by default).

## Examples

See the [`examples/`](https://github.com/djeedai/bevy_tweening/tree/77b89d9df5a28f66ae6b153e6d24cf0d58042353/examples) folder.

### [`menu`](examples/menu.rs)

```rust
cargo run --example menu --features="bevy/bevy_winit"
```

![menu](https://raw.githubusercontent.com/djeedai/bevy_tweening/77b89d9df5a28f66ae6b153e6d24cf0d58042353/examples/menu.gif)

### [`sprite_color`](examples/sprite_color.rs)

```rust
cargo run --example sprite_color --features="bevy/bevy_winit"
```

![sprite_color](https://raw.githubusercontent.com/djeedai/bevy_tweening/77b89d9df5a28f66ae6b153e6d24cf0d58042353/examples/sprite_color.gif)

### [`transform_rotation`](examples/transform_rotation.rs)

```rust
cargo run --example transform_rotation --features="bevy/bevy_winit"
```

![sprite_color](https://raw.githubusercontent.com/djeedai/bevy_tweening/77b89d9df5a28f66ae6b153e6d24cf0d58042353/examples/transform_rotation.gif)

### [`transform_translation`](examples/transform_translation.rs)

```rust
cargo run --example transform_translation --features="bevy/bevy_winit"
```

![sprite_color](https://raw.githubusercontent.com/djeedai/bevy_tweening/77b89d9df5a28f66ae6b153e6d24cf0d58042353/examples/transform_translation.gif)

### [`colormaterial_color`](examples/colormaterial_color.rs)

```rust
cargo run --example colormaterial_color --features="bevy/bevy_winit"
```

![colormaterial_color](https://raw.githubusercontent.com/djeedai/bevy_tweening/77b89d9df5a28f66ae6b153e6d24cf0d58042353/examples/colormaterial_color.gif)

### [`ui_position`](examples/ui_position.rs)

```rust
cargo run --example ui_position --features="bevy/bevy_winit"
```

![ui_position](https://raw.githubusercontent.com/djeedai/bevy_tweening/77b89d9df5a28f66ae6b153e6d24cf0d58042353/examples/ui_position.gif)

### [`sequence`](examples/sequence.rs)

```rust
cargo run --example sequence --features="bevy/bevy_winit"
```

![sequence](https://raw.githubusercontent.com/djeedai/bevy_tweening/77b89d9df5a28f66ae6b153e6d24cf0d58042353/examples/sequence.gif)

## Ease Functions

Many [ease functions](https://docs.rs/interpolation/0.2.0/interpolation/enum.EaseFunction.html) are available:

- QuadraticIn
- QuadraticOut
- QuadraticInOut
- CubicIn
- CubicOut
- CubicInOut
- QuarticIn
- QuarticOut
- QuarticInOut
- QuinticIn
- QuinticOut
- QuinticInOut
- SineIn
- SineOut
- SineInOut
- CircularIn
- CircularOut
- CircularInOut
- ExponentialIn
- ExponentialOut
- ExponentialInOut
- ElasticIn
- ElasticOut
- ElasticInOut
- BackIn
- BackOut
- BackInOut
- BounceIn
- BounceOut
- BounceInOut

## Compatible Bevy versions

The `main` branch is compatible with the latest Bevy release.

Compatibility of `bevy_tweening` versions:

| `bevy_tweening` | `bevy` |
| :--             | :--    |
| `0.10`          | `0.13` |
| `0.9`           | `0.12` |
| `0.8`           | `0.11` |
| `0.7`           | `0.10` |
| `0.6`           | `0.9`  |
| `0.5`           | `0.8`  |
| `0.4`           | `0.7`  |
| `0.2`-`0.3`     | `0.6`  |
| `0.1`           | `0.5`  |

Due to the fast-moving nature of Bevy and frequent breaking changes, and the limited resources to maintan 🍃 Bevy Tweening, the `main` (unreleased) Bevy branch is not supported. However the `bevy_tweening` crate is upgraded shortly after each new `bevy` release to support the newly released version.

## Comparison with `bevy_easings`

The `bevy_tweening` library started as a fork of [the `bevy_easings` library by François Mocker](https://github.com/vleue/bevy_easings), with the goals to:

- explore an alternative design based on lenses instead of generic types for each easer/animator. This reduces both the number of generic types needed, and hopefully the code size, as well as the number of systems needed to perform the interpolation.
- improve the interpolation of assets to avoid creating many copies like `bevy_easings` does, and instead mutate the assets (and, by similarity, the components too) in-place without making a copy. The in-place mutation also allows a more optimal interpolation limited to modifying the fields of interest only, instead of creating a new copy of the entire component each tick.
