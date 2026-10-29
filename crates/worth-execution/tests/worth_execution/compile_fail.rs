//! Compile fail integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "compile_fail/charged_bytes_derive.rs"]
pub(crate) mod charged_bytes_derive;

#[path = "compile_fail/public_boundaries.rs"]
pub(crate) mod public_boundaries;
