#![cfg_attr(test, allow(clippy::disallowed_methods))]
// A Signal-backed Home carries Signal's persistent (NodeId, NodeId) chunks; their
// deep Sync proof needs the same compiler depth as the oracle testkit.
#![recursion_limit = "256"]
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/batches/mod.rs"]
pub(crate) mod batches;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/branch_lifecycle/mod.rs"]
pub(crate) mod branch_lifecycle;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/crash_oracle/mod.rs"]
pub(crate) mod crash_oracle;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/durable_run/mod.rs"]
pub(crate) mod durable_run;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/identity_safety/mod.rs"]
pub(crate) mod identity_safety;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/incarnation/mod.rs"]
pub(crate) mod incarnation;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/journeys/mod.rs"]
pub(crate) mod journeys;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/killed_child/mod.rs"]
pub(crate) mod killed_child;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/media_case/mod.rs"]
pub(crate) mod media_case;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/media_faults/mod.rs"]
pub(crate) mod media_faults;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/media_sweep/mod.rs"]
pub(crate) mod media_sweep;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/open_refusals/mod.rs"]
pub(crate) mod open_refusals;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/prepared_home/mod.rs"]
pub(crate) mod prepared_home;
#[path = "worth_store_court_application/process/mod.rs"]
pub(crate) mod process;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/process_kill/mod.rs"]
pub(crate) mod process_kill;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/publication_refusals/mod.rs"]
pub(crate) mod publication_refusals;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/recovery_interruption/mod.rs"]
pub(crate) mod recovery_interruption;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/residency_agreement/mod.rs"]
pub(crate) mod residency_agreement;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/residency_capture/mod.rs"]
pub(crate) mod residency_capture;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/residency_digest/mod.rs"]
pub(crate) mod residency_digest;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/residency_fixture/mod.rs"]
pub(crate) mod residency_fixture;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/residency_media/mod.rs"]
pub(crate) mod residency_media;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/residency_process/mod.rs"]
pub(crate) mod residency_process;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/schema_recovery/mod.rs"]
pub(crate) mod schema_recovery;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/schema_transition/mod.rs"]
pub(crate) mod schema_transition;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/selected_checkpoint/mod.rs"]
pub(crate) mod selected_checkpoint;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/transition_oracle/mod.rs"]
pub(crate) mod transition_oracle;
#[allow(dead_code, unused_imports)]
#[path = "worth_store_court_application/support/workload/mod.rs"]
pub(crate) mod workload;
