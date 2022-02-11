# 🍃 Bevy Tweening

[![License: MIT/Apache](https://img.shields.io/badge/License-MIT%20or%20Apache2-blue.svg)](https://opensource.org/licenses/MIT)
[![Doc](https://docs.rs/bevy_tweening/badge.svg)](https://docs.rs/bevy_tweening)
[![Crate](https://img.shields.io/crates/v/bevy_tweening.svg)](https://crates.io/crates/bevy_tweening)
[![Build Status](https://github.com/djeedai/bevy_tweening/actions/workflows/ci.yaml/badge.svg)](https://github.com/djeedai/bevy_tweening/actions/workflows/ci.yaml)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-v0.6-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

Tweening animation plugin for the Bevy game engine.

## Features

- [x] Animate any field of any component or asset, including custom ones.
- [x] Run multiple tweens (animations) per component/asset in parallel.
- [x] Chain multiple tweens (animations) one after the other for complex animations.

## Usage

### System setup

Add the tweening plugin to your app:

```rust
App::default()
    .add_plugins(DefaultPlugins)
    .add_plugin(TweeningPlugin)
    .run();
```

### Animate a component

Animate the transform position of an entity:

```rust
commands
    // Spawn a Sprite entity to animate the position of
    .spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            custom_size: Some(Vec2::new(size, size)),
            ..Default::default()
        },
        ..Default::default()
    })
    // Add an Animator component to perform the animation. This is a shortcut to
    // create both an Animator and a Tween, and assign the Tween to the Animator.
    .insert(Animator::new(
        // Use a quadratic easing on both endpoints
        EaseFunction::QuadraticInOut,
        // Loop animation back and forth over 1 second, with a 0.5 second
        // pause after each cycle (start -> end -> start -> pause -> ...).
        TweeningType::PingPong {
            duration: Duration::from_secs(1),
            pause: Some(Duration::from_millis(500)),
        },
        // The lens gives access to the Transform component of the Sprite,
        // for the Animator to animate it. It also contains the start and
        // end values associated with the animation ratios 0. and 1.
        TransformPositionLens {
            start: Vec3::new(0., 0., 0.),
            end: Vec3::new(1., 2., -4.),
        },
    ));
```

## Predefined Lenses

A small number of predefined lenses are available for the most common use cases, which also serve as examples. Users are encouraged to write their own lens to tailor the animation to their use case.

The naming scheme for predefined lenses is `"<TargetName><FieldName>Lens"`, where `<TargetName>` is the name of the target Bevy component or asset type which is queried by the internal animation system to be modified, and `<FieldName>` is the field which is mutated in place by the lens. All predefined lenses modify a single field. Custom lenses can be written which modify multiple fields at once.

### Bevy Components

| Target Component | Animated Field | Lens |
|---|---|---|
| [`Sprite`](https://docs.rs/bevy/0.6.0/bevy/sprite/struct.Sprite.html) | [`color`](https://docs.rs/bevy/0.6.0/bevy/sprite/struct.Sprite.html#structfield.color) | [`SpriteColorLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/struct.SpriteColorLens.html) |
| [`Transform`](https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html) | [`translation`](https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html#structfield.translation) | [`TransformPositionLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/struct.TransformPositionLens.html) |
| | [`rotation`](https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html#structfield.rotation) | [`TransformRotationLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/struct.TransformRotationLens.html) |
| | [`scale`](https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html#structfield.scale) | [`TransformScaleLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/struct.TransformScaleLens.html) |
| [`Style`](https://docs.rs/bevy/0.6.0/bevy/ui/struct.Style.html) | [`position`](https://docs.rs/bevy/0.6.0/bevy/ui/struct.Style.html#structfield.position) | [`UiPositionLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/struct.UiPositionLens.html) |
| [`Text`](https://docs.rs/bevy/0.6.0/bevy/text/struct.Text.html) | [`TextStyle::color`](https://docs.rs/bevy/0.6.0/bevy/text/struct.TextStyle.html#structfield.color) | [`TextColorLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/struct.TextColorLens.html) |

### Bevy Assets

| Target Asset | Animated Field | Lens |
|---|---|---|
| [`ColorMaterial`](https://docs.rs/bevy/0.6.0/bevy/sprite/struct.ColorMaterial.html) | [`color`](https://docs.rs/bevy/0.6.0/bevy/sprite/struct.ColorMaterial.html#structfield.color) | [`ColorMaterialColorLens`](https://docs.rs/bevy_tweening/latest/bevy_tweening/struct.ColorMaterialColorLens.html) |

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

Custom components are animated like built-in Bevy ones, via a lens.

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

Then, in addition, the system `component_animator_system::<CustomComponent>` needs to be added to the application. This system will extract each frame all `CustomComponent` instances with an `Animator<CustomComponent>` on the same entity, and animate the component via its animator.

## Custom asset support

The process is similar to custom components, creating a custom lens for the custom asset. The system to add is `asset_animator_system::<CustomAsset>`.

## Examples

See the [`examples/`](https://github.com/djeedai/bevy_tweening/tree/main/examples) folder.

### [`menu`](examples/menu.rs)

```rust
cargo run --example menu --features="bevy/bevy_winit"
```

![menu](https://raw.githubusercontent.com/djeedai/bevy_tweening/main/examples/menu.gif)

### [`sprite_color`](examples/sprite_color.rs)

```rust
cargo run --example sprite_color --features="bevy/bevy_winit"
```

![sprite_color](https://raw.githubusercontent.com/djeedai/bevy_tweening/main/examples/sprite_color.gif)

### [`transform_rotation`](examples/transform_rotation.rs)

```rust
cargo run --example transform_rotation --features="bevy/bevy_winit"
```

![sprite_color](https://raw.githubusercontent.com/djeedai/bevy_tweening/main/examples/transform_rotation.gif)

### [`transform_translation`](examples/transform_translation.rs)

```rust
cargo run --example transform_translation --features="bevy/bevy_winit"
```

![sprite_color](https://raw.githubusercontent.com/djeedai/bevy_tweening/main/examples/transform_translation.gif)

### [`colormaterial_color`](examples/colormaterial_color.rs)

```rust
cargo run --example colormaterial_color --features="bevy/bevy_winit"
```

![colormaterial_color](https://raw.githubusercontent.com/djeedai/bevy_tweening/main/examples/colormaterial_color.gif)

### [`ui_position`](examples/ui_position.rs)

```rust
cargo run --example ui_position --features="bevy/bevy_winit"
```

![ui_position](https://raw.githubusercontent.com/djeedai/bevy_tweening/main/examples/ui_position.gif)

### [`sequence`](examples/sequence.rs)

```rust
cargo run --example sequence --features="bevy/bevy_winit"
```

![sequence](https://raw.githubusercontent.com/djeedai/bevy_tweening/main/examples/sequence.gif)

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
| `0.2`-`0.3`     | `0.6`  |
| `0.1`           | `0.5`  |

## Comparison with `bevy_easings`

The `bevy_tweening` library started as a fork of [the `bevy_easings` library by François Mocker](https://github.com/vleue/bevy_easings), with the goals to:

- explore an alternative design based on lenses instead of generic types for each easer/animator. This reduces both the number of generic types needed, and hopefully the code size, as well as the number of systems needed to perform the interpolation.
- improve the interpolation of assets to avoid creating many copies like `bevy_easings` does, and instead mutate the assets (and, by similarity, the components too) in-place without making a copy. The in-place mutation also allows a more optimal interpolation limited to modifying the fields of interest only, instead of creating a new copy of the entire component each tick.
