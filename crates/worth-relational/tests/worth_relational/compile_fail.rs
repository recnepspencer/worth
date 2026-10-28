//! Compile fail integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "compile_fail/branch_reference_compile_time.rs"]
// The relational_contracts integration suite: one test binary for the crate's
// scenario files, which stay in tests/ with their own module trees.
// Support modules several scenarios share are compiled once here.
pub(crate) mod branch_reference_compile_time;

#[path = "compile_fail/phase_boundaries_compile_fail.rs"]
pub(crate) mod phase_boundaries_compile_fail;

#[path = "compile_fail/ui.rs"]
pub(crate) mod ui;
