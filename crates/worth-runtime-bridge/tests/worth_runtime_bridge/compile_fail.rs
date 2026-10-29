//! Compile fail integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "compile_fail/phase_boundaries_bridge_truth_identity_compile_fail.rs"]
pub(crate) mod phase_boundaries_bridge_truth_identity_compile_fail;

#[path = "compile_fail/phase_boundaries_compile_fail.rs"]
pub(crate) mod phase_boundaries_compile_fail;
