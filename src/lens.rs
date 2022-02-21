//! Collection of predefined lenses for common Bevy components and assets.

use bevy::prelude::*;

/// A lens over a subset of a component.
///
/// The lens takes a `target` component or asset from a query, as a mutable reference,
/// and animates (tweens) a subet of the fields of the component/asset based on the
/// linear ratio `ratio`, already sampled from the easing curve.
///
/// # Example
///
/// Implement `Lens` for a custom type:
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_tweening::*;
/// struct MyLens {
///   start: f32,
///   end: f32,
/// }
///
/// #[derive(Component)]
/// struct MyStruct(f32);
///
/// impl Lens<MyStruct> for MyLens {
///   fn lerp(&mut self, target: &mut MyStruct, ratio: f32) {
///     target.0 = self.start + (self.end - self.start) * ratio;
///   }
/// }
/// ```
///
pub trait Lens<T> {
    /// Perform a linear interpolation (lerp) over the subset of fields of a component
    /// or asset the lens focuses on, based on the linear ratio `ratio`. The `target`
    /// component or asset is mutated in place. The implementation decides which fields
    /// are interpolated, and performs the animation in-place, overwriting the target.
    fn lerp(&mut self, target: &mut T, ratio: f32);
}

/// A lens to manipulate the [`color`] field of a section of a [`Text`] component.
///
/// [`color`]: https://docs.rs/bevy/0.6.0/bevy/text/struct.TextStyle.html#structfield.color
/// [`Text`]: https://docs.rs/bevy/0.6.0/bevy/text/struct.Text.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TextColorLens {
    /// Start color.
    pub start: Color,
    /// End color.
    pub end: Color,
    /// Index of the text section in the [`Text`] component.
    pub section: usize,
}

impl Lens<Text> for TextColorLens {
    fn lerp(&mut self, target: &mut Text, ratio: f32) {
        // Note: Add<f32> for Color affects alpha, but not Mul<f32>. So use Vec4 for consistency.
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);
        target.sections[self.section].style.color = value.into();
    }
}

/// A lens to manipulate the [`translation`] field of a [`Transform`] component.
///
/// [`translation`]: https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html#structfield.translation
/// [`Transform`]: https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformPositionLens {
    /// Start value of the translation.
    pub start: Vec3,
    /// End value of the translation.
    pub end: Vec3,
}

impl Lens<Transform> for TransformPositionLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.start + (self.end - self.start) * ratio;
        target.translation = value;
    }
}

/// A lens to manipulate the [`rotation`] field of a [`Transform`] component.
///
/// [`rotation`]: https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html#structfield.rotation
/// [`Transform`]: https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformRotationLens {
    /// Start value of the rotation.
    pub start: Quat,
    /// End value of the rotation.
    pub end: Quat,
}

impl Lens<Transform> for TransformRotationLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        target.rotation = self.start.slerp(self.end, ratio); // FIXME - This slerps the shortest path only! https://docs.rs/bevy/latest/bevy/math/struct.Quat.html#method.slerp
    }
}

/// A lens to manipulate the [`scale`] field of a [`Transform`] component.
///
/// [`scale`]: https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html#structfield.scale
/// [`Transform`]: https://docs.rs/bevy/0.6.0/bevy/transform/components/struct.Transform.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformScaleLens {
    /// Start value of the scale.
    pub start: Vec3,
    /// End value of the scale.
    pub end: Vec3,
}

impl Lens<Transform> for TransformScaleLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.start + (self.end - self.start) * ratio;
        target.scale = value;
    }
}

/// A lens to manipulate the [`position`] field of a UI [`Style`] component.
///
/// [`position`]: https://docs.rs/bevy/0.6.0/bevy/ui/struct.Style.html#structfield.position
/// [`Style`]: https://docs.rs/bevy/0.6.0/bevy/ui/struct.Style.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UiPositionLens {
    /// Start position.
    pub start: Rect<Val>,
    /// End position.
    pub end: Rect<Val>,
}

fn lerp_val(start: &Val, end: &Val, ratio: f32) -> Val {
    match (start, end) {
        (Val::Percent(start), Val::Percent(end)) => Val::Percent(start + (end - start) * ratio),
        (Val::Px(start), Val::Px(end)) => Val::Px(start + (end - start) * ratio),
        _ => *start,
    }
}

impl Lens<Style> for UiPositionLens {
    fn lerp(&mut self, target: &mut Style, ratio: f32) {
        target.position = Rect {
            left: lerp_val(&self.start.left, &self.end.left, ratio),
            right: lerp_val(&self.start.right, &self.end.right, ratio),
            top: lerp_val(&self.start.top, &self.end.top, ratio),
            bottom: lerp_val(&self.start.bottom, &self.end.bottom, ratio),
        };
    }
}

/// A lens to manipulate the [`color`] field of a [`ColorMaterial`] asset.
///
/// [`color`]: https://docs.rs/bevy/0.6.0/bevy/sprite/struct.ColorMaterial.html#structfield.color
/// [`ColorMaterial`]: https://docs.rs/bevy/0.6.0/bevy/sprite/struct.ColorMaterial.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ColorMaterialColorLens {
    /// Start color.
    pub start: Color,
    /// End color.
    pub end: Color,
}

impl Lens<ColorMaterial> for ColorMaterialColorLens {
    fn lerp(&mut self, target: &mut ColorMaterial, ratio: f32) {
        // Note: Add<f32> for Color affects alpha, but not Mul<f32>. So use Vec4 for consistency.
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);
        target.color = value.into();
    }
}

/// A lens to manipulate the [`color`] field of a [`Sprite`] asset.
///
/// [`color`]: https://docs.rs/bevy/0.6.0/bevy/sprite/struct.Sprite.html#structfield.color
/// [`Sprite`]: https://docs.rs/bevy/0.6.0/bevy/sprite/struct.Sprite.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpriteColorLens {
    /// Start color.
    pub start: Color,
    /// End color.
    pub end: Color,
}

impl Lens<Sprite> for SpriteColorLens {
    fn lerp(&mut self, target: &mut Sprite, ratio: f32) {
        // Note: Add<f32> for Color affects alpha, but not Mul<f32>. So use Vec4 for consistency.
        let start: Vec4 = self.start.into();
        let end: Vec4 = self.end.into();
        let value = start.lerp(end, ratio);
        target.color = value.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_color() {
        let mut lens = TextColorLens {
            start: Color::RED,
            end: Color::BLUE,
            section: 0,
        };
        let mut text = Text::with_section("", Default::default(), Default::default());

        lens.lerp(&mut text, 0.);
        assert_eq!(text.sections[0].style.color, Color::RED);

        lens.lerp(&mut text, 1.);
        assert_eq!(text.sections[0].style.color, Color::BLUE);

        lens.lerp(&mut text, 0.3);
        assert_eq!(text.sections[0].style.color, Color::rgba(0.7, 0., 0.3, 1.0));
    }

    #[test]
    fn transform_position() {
        let mut lens = TransformPositionLens {
            start: Vec3::ZERO,
            end: Vec3::new(1., 2., -4.),
        };
        let mut transform = Transform::default();

        lens.lerp(&mut transform, 0.);
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));
        assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));
        assert!(transform.scale.abs_diff_eq(Vec3::ONE, 1e-5));

        lens.lerp(&mut transform, 1.);
        assert!(transform
            .translation
            .abs_diff_eq(Vec3::new(1., 2., -4.), 1e-5));
        assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));
        assert!(transform.scale.abs_diff_eq(Vec3::ONE, 1e-5));

        lens.lerp(&mut transform, 0.3);
        assert!(transform
            .translation
            .abs_diff_eq(Vec3::new(0.3, 0.6, -1.2), 1e-5));
        assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));
        assert!(transform.scale.abs_diff_eq(Vec3::ONE, 1e-5));
    }

    #[test]
    fn transform_rotation() {
        let mut lens = TransformRotationLens {
            start: Quat::IDENTITY,
            end: Quat::from_rotation_z(100_f32.to_radians()),
        };
        let mut transform = Transform::default();

        lens.lerp(&mut transform, 0.);
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));
        assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));
        assert!(transform.scale.abs_diff_eq(Vec3::ONE, 1e-5));

        lens.lerp(&mut transform, 1.);
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));
        assert!(transform
            .rotation
            .abs_diff_eq(Quat::from_rotation_z(100_f32.to_radians()), 1e-5));
        assert!(transform.scale.abs_diff_eq(Vec3::ONE, 1e-5));

        lens.lerp(&mut transform, 0.3);
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));
        assert!(transform
            .rotation
            .abs_diff_eq(Quat::from_rotation_z(30_f32.to_radians()), 1e-5));
        assert!(transform.scale.abs_diff_eq(Vec3::ONE, 1e-5));
    }

    #[test]
    fn transform_scale() {
        let mut lens = TransformScaleLens {
            start: Vec3::ZERO,
            end: Vec3::new(1., 2., -4.),
        };
        let mut transform = Transform::default();

        lens.lerp(&mut transform, 0.);
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));
        assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));
        assert!(transform.scale.abs_diff_eq(Vec3::ZERO, 1e-5));

        lens.lerp(&mut transform, 1.);
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));
        assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));
        assert!(transform.scale.abs_diff_eq(Vec3::new(1., 2., -4.), 1e-5));

        lens.lerp(&mut transform, 0.3);
        assert!(transform.translation.abs_diff_eq(Vec3::ZERO, 1e-5));
        assert!(transform.rotation.abs_diff_eq(Quat::IDENTITY, 1e-5));
        assert!(transform.scale.abs_diff_eq(Vec3::new(0.3, 0.6, -1.2), 1e-5));
    }

    #[test]
    fn colormaterial_color() {
        let mut lens = ColorMaterialColorLens {
            start: Color::RED,
            end: Color::BLUE,
        };
        let mut mat = ColorMaterial {
            color: Color::WHITE,
            texture: None,
        };

        lens.lerp(&mut mat, 0.);
        assert_eq!(mat.color, Color::RED);

        lens.lerp(&mut mat, 1.);
        assert_eq!(mat.color, Color::BLUE);

        lens.lerp(&mut mat, 0.3);
        assert_eq!(mat.color, Color::rgba(0.7, 0., 0.3, 1.0));
    }

    #[test]
    fn sprite_color() {
        let mut lens = SpriteColorLens {
            start: Color::RED,
            end: Color::BLUE,
        };
        let mut sprite = Sprite {
            color: Color::WHITE,
            flip_x: false,
            flip_y: false,
            custom_size: None,
        };

        lens.lerp(&mut sprite, 0.);
        assert_eq!(sprite.color, Color::RED);

        lens.lerp(&mut sprite, 1.);
        assert_eq!(sprite.color, Color::BLUE);

        lens.lerp(&mut sprite, 0.3);
        assert_eq!(sprite.color, Color::rgba(0.7, 0., 0.3, 1.0));
    }
}
