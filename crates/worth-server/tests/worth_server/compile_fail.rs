//! Compile fail integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "compile_fail/certification_suite.rs"]
// The server_contracts integration suite: one test binary for the crate's
// scenario files, which stay in tests/ with their own module trees.
// Support modules several scenarios share are compiled once here.
pub(crate) mod certification_suite;

#[path = "compile_fail/compile_fail.rs"]
pub(crate) mod compile_fail;
