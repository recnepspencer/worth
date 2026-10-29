//! Compile fail integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "compile_fail/compile_time_boundaries.rs"]
pub(crate) mod compile_time_boundaries;

#[path = "compile_fail/value_shape.rs"]
pub(crate) mod value_shape;
