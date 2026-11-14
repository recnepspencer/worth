//! Shared Query history records at the public data-contract boundary.
#[path = "query_obligations/conditional_decision.rs"]
mod conditional_decision;
#[path = "query_obligations/managed_cleanup.rs"]
mod managed_cleanup;
#[path = "query_obligations/support.rs"]
mod support;
#[path = "query_obligations/support_retirement.rs"]
mod support_retirement;

#[path = "query_obligations/managed_run.rs"]
mod managed_run;

#[path = "query_obligations/owned_async.rs"]
mod owned_async;

#[path = "query_obligations/yield_recovery.rs"]
mod yield_recovery;

#[path = "query_obligations/restore_recovery.rs"]
mod restore_recovery;
