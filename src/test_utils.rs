use bevy::{
    math::{Quat, Vec3, Vec4},
    transform::components::Transform,
};

pub(crate) trait AbsDiffEq: std::fmt::Debug {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool;
    fn delta(a: Self, b: Self) -> Self;
}

impl AbsDiffEq for f32 {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool {
        (a - b).abs() < tol
    }
    fn delta(a: Self, b: Self) -> Self {
        (a - b).abs()
    }
}

impl AbsDiffEq for Vec3 {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool {
        Vec3::abs_diff_eq(a, b, tol)
    }
    fn delta(a: Self, b: Self) -> Self {
        (a - b).abs()
    }
}

impl AbsDiffEq for Quat {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool {
        Quat::abs_diff_eq(a, b, tol)
    }
    fn delta(a: Self, b: Self) -> Self {
        Quat::from_vec4((Vec4::from(a) - Vec4::from(b)).abs())
    }
}

impl AbsDiffEq for Transform {
    fn abs_diff_eq(a: Self, b: Self, tol: f32) -> bool {
        a.translation.abs_diff_eq(b.translation, tol)
            && a.rotation.abs_diff_eq(b.rotation, tol)
            && a.scale.abs_diff_eq(b.scale, tol)
    }
    fn delta(a: Self, b: Self) -> Self {
        Transform {
            translation: AbsDiffEq::delta(a.translation, b.translation),
            rotation: AbsDiffEq::delta(a.rotation, b.rotation),
            scale: AbsDiffEq::delta(a.scale, b.scale),
        }
    }
}

/// Assert that two floating-point quantities are approximately equal.
///
/// This macro asserts that the absolute difference between the two first
/// arguments is strictly less than a tolerance factor, which can be explicitly
/// passed as third argument or implicitly defaults to `1e-5`.
///
/// # Usage
///
/// ```
/// let x = 3.500009;
/// assert_approx_eq!(x, 3.5); // default tolerance 1e-5
///
/// let x = 3.509;
/// assert_approx_eq!(x, 3.5, 0.01); // explicit tolerance
/// ```
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr $(,)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                assert!(
                    crate::test_utils::AbsDiffEq::abs_diff_eq(*left_val, *right_val, 1e-5),
                    "assertion failed: expected={:?} actual={:?} delta={:?} tol=1e-5(default)",
                    left_val,
                    right_val,
                    crate::test_utils::AbsDiffEq::delta(*left_val, *right_val),
                );
            }
        }
    };
    ($left:expr, $right:expr, $tol:expr $(,)?) => {
        match (&$left, &$right, &$tol) {
            (left_val, right_val, tol_val) => {
                assert!(
                    crate::test_utils::AbsDiffEq::abs_diff_eq(*left_val, *right_val, *tol_val),
                    "assertion failed: expected={:?} actual={:?} delta={:?} tol={}",
                    left_val,
                    right_val,
                    crate::test_utils::AbsDiffEq::delta(*left_val, *right_val),
                    tol_val
                );
            }
        }
    };
}

pub(crate) use assert_approx_eq;
