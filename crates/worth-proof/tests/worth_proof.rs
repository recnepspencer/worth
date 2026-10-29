//! Integration courts grouped by execution cost.

#[path = "worth_proof/fast.rs"]
mod fast;

#[path = "worth_proof/process.rs"]
mod process;

#[path = "worth_proof/compile_fail.rs"]
mod compile_fail;

#[path = "worth_proof/fixtures/support/mod.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_support;
