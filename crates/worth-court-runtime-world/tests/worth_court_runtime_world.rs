#![cfg_attr(test, allow(clippy::disallowed_methods))]
#[path = "worth_court_runtime_world/compile_fail/mod.rs"]
pub(crate) mod compile_fail;
#[path = "worth_court_runtime_world/cost/mod.rs"]
pub(crate) mod cost;
#[allow(dead_code, unused_imports)]
#[path = "worth_court_runtime_world/support/court/mod.rs"]
pub(crate) mod court;
#[path = "worth_court_runtime_world/oracle/mod.rs"]
pub(crate) mod oracle;
#[allow(dead_code, unused_imports)]
#[path = "worth_court_runtime_world/support/owned_async/mod.rs"]
pub(crate) mod owned_async;
#[allow(dead_code, unused_imports)]
#[path = "worth_court_runtime_world/support/reference/mod.rs"]
pub(crate) mod reference;
#[path = "worth_court_runtime_world/runtime_world/mod.rs"]
pub(crate) mod runtime_world;
#[allow(dead_code, unused_imports)]
#[path = "worth_court_runtime_world/support/runtime_world_context/mod.rs"]
pub(crate) mod runtime_world_context;
#[allow(dead_code, unused_imports)]
#[path = "worth_court_runtime_world/support/test_execution_policy/mod.rs"]
pub(crate) mod test_execution_policy;

#[path = "worth_court_runtime_world/supported_programs/mod.rs"]
mod supported_programs;
