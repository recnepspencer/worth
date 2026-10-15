//! The installation_contracts integration suite: one test binary for the crate's
//! scenario files, which stay in tests/ with their own module trees.
//! Support modules several scenarios share are compiled once here.

#[path = "application_contribution_installation.rs"]
mod application_contribution_installation;
#[path = "application_value_binding_installation.rs"]
mod application_value_binding_installation;
#[path = "temporal_intent_idempotency_relation.rs"]
mod temporal_intent_idempotency_relation;
#[path = "untrusted_application_capability_construction.rs"]
mod untrusted_application_capability_construction;
#[path = "untrusted_record_construction.rs"]
mod untrusted_record_construction;
