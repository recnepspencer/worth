//! Integration courts grouped by execution cost.

#[path = "worth_runtime_bridge/fast.rs"]
mod fast;

#[path = "worth_runtime_bridge/compile_fail.rs"]
mod compile_fail;

#[path = "worth_runtime_bridge/serial.rs"]
mod serial;

#[path = "worth_runtime_bridge/test_process.rs"]
mod test_process;
