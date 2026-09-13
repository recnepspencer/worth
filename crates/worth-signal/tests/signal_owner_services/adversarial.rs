//! Phase 6 adversarial evidence for the public Signal owner-service contract.
//!
//! This module is intentionally subordinate to `signal_owner_services.rs`.
//! Every schedule reaches the public facade and the real sealed owner; the
//! operation-control feature only makes an existing owner boundary
//! deterministic for the test.

#[path = "adversarial/world.rs"]
mod world;

#[cfg(feature = "test-operation-control")]
#[path = "adversarial/operation_control.rs"]
mod operation_control;

#[path = "adversarial/capacity_cleanup.rs"]
mod capacity_cleanup;
#[path = "adversarial/cost.rs"]
mod cost;
