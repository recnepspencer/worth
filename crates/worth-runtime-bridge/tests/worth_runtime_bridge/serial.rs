//! Serial integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "serial/execution_request_denials.rs"]
// The bridge_contracts integration suite: one test binary for the crate's
// scenario files, which stay in tests/ with their own module trees.
// Support modules several scenarios share are compiled once here.
pub(crate) mod execution_request_denials;
