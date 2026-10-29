//! Integration courts grouped by execution cost.

#[path = "worth_relational/fast.rs"]
mod fast;

#[path = "worth_relational/exhaustive.rs"]
mod exhaustive;

#[path = "worth_relational/scale.rs"]
mod scale;

#[path = "worth_relational/compile_fail.rs"]
mod compile_fail;

#[path = "worth_relational/serial.rs"]
mod serial;

#[path = "worth_relational/fixtures/relational_certification/test_execution_policy.rs"]
#[allow(dead_code, unused_imports)]
pub(crate) mod shared_relational_certification_test_execution_policy;

#[path = "worth_relational/test_process.rs"]
mod test_process;
