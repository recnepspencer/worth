#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Integration courts grouped by execution cost.

#[path = "worth_signal/fast.rs"]
mod fast;

#[path = "worth_signal/exhaustive.rs"]
mod exhaustive;

#[path = "worth_signal/compile_fail.rs"]
mod compile_fail;

#[path = "worth_signal/serial.rs"]
mod serial;

#[path = "../examples/branch_bases.rs"]
// The signal_contracts integration suite: one test binary for the crate's
// scenario files, which stay in tests/ with their own module trees.
// Support modules several scenarios share are compiled once here.
pub(crate) mod branch_bases_example;

#[path = "worth_signal/test_process.rs"]
mod test_process;
