//! Fast integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "fast/certification.rs"]
// The foundational_contracts integration suite: one test binary for the crate's
// scenario files, which stay in tests/ with their own module trees.
// Support modules several scenarios share are compiled once here.
pub(crate) mod certification;

#[path = "fast/expressions.rs"]
pub(crate) mod expressions;

#[path = "fast/resident_value.rs"]
pub(crate) mod resident_value;

#[path = "fast/value_shape.rs"]
pub(crate) mod value_shape;
