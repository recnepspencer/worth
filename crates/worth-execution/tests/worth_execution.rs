//! Integration courts grouped by execution cost.

#[path = "worth_execution/fast.rs"]
mod fast;

#[path = "worth_execution/exhaustive.rs"]
mod exhaustive;

#[path = "worth_execution/compile_fail.rs"]
mod compile_fail;

#[path = "worth_execution/serial.rs"]
mod serial;

#[path = "worth_execution/fixtures/support/test_serialization.rs"]
pub(crate) mod test_serialization;

#[path = "worth_execution/fixtures/support/reducer_overlap.rs"]
pub(crate) mod reducer_overlap;

#[path = "worth_execution/test_process.rs"]
mod test_process;
