//! Serial integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "serial/borrowed_canonical_material.rs"]
// `borrowed_canonical_material` keeps its own test process: it constructs process-wide
// state that the shared suite binary cannot host twice.
pub(crate) mod borrowed_canonical_material;
