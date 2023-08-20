mod controller;

pub use controller::*;

/// A method that will update the internal state of a [`Component`].
pub type UpdateComponentFn = dyn Fn(&[usize]);
