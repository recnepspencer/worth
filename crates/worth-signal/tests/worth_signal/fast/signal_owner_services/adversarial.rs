#[path = "../../fixtures/signal_owner_services/adversarial/world.rs"]
// Phase 6 adversarial evidence for the public Signal owner-service contract.
//
// This module is intentionally subordinate to `signal_owner_services.rs`.
// Every schedule reaches the public facade and the real sealed owner; the
// operation-control feature only makes an existing owner boundary
// deterministic for the test.
pub(crate) mod world;

#[path = "adversarial/operation_control.rs"]
pub(crate) mod operation_control;

#[path = "adversarial/capacity_cleanup.rs"]
pub(crate) mod capacity_cleanup;

#[path = "adversarial/cost.rs"]
pub(crate) mod cost;
