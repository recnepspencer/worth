//! Exhaustive integration courts.
#[allow(unused_imports)]
use crate::*;

#[path = "exhaustive/certification_suite.rs"]
// The server_contracts integration suite: one test binary for the crate's
// scenario files, which stay in tests/ with their own module trees.
// Support modules several scenarios share are compiled once here.
pub(crate) mod certification_suite;

#[path = "exhaustive/durable_product_mutation_crash_recovery.rs"]
pub(crate) mod durable_product_mutation_crash_recovery;
