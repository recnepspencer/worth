//! Integration courts grouped by execution cost.

#[path = "worth_foundational/fast.rs"]
mod fast;

#[path = "worth_foundational/exhaustive.rs"]
mod exhaustive;

#[path = "worth_foundational/compile_fail.rs"]
mod compile_fail;

#[path = "worth_foundational/serial.rs"]
mod serial;
