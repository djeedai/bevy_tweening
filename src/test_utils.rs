/// Utility to compare floating-point values with a tolerance.
pub(crate) fn abs_diff_eq(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() < tol
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
/// assert_approx_eq!(x, 3.5);       // default tolerance 1e-5
///
/// let x = 3.509;
/// assert_approx_eq!(x, 3.5, 0.01); // explicit tolerance
/// ```
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr $(,)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                assert!(
                    abs_diff_eq(*left_val, *right_val, 1e-5),
                    "assertion failed: expected={} actual={} delta={} tol=1e-5(default)",
                    left_val,
                    right_val,
                    (left_val - right_val).abs(),
                );
            }
        }
    };
    ($left:expr, $right:expr, $tol:expr $(,)?) => {
        match (&$left, &$right, &$tol) {
            (left_val, right_val, tol_val) => {
                assert!(
                    abs_diff_eq(*left_val, *right_val, *tol_val),
                    "assertion failed: expected={} actual={} delta={} tol={}",
                    left_val,
                    right_val,
                    (left_val - right_val).abs(),
                    tol_val
                );
            }
        }
    };
}

pub(crate) use assert_approx_eq;
