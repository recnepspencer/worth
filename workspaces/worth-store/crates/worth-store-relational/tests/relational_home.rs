//! A Relational home on Worth Store: journeys and the crash-consistency
//! oracle. The reference for every claim is a memory-only Relational runtime
//! executing the same script.

#[path = "relational_home/batches.rs"]
mod batches;
#[path = "relational_home/branch_lifecycle.rs"]
mod branch_lifecycle;
#[path = "relational_home/crash_oracle.rs"]
mod crash_oracle;
#[path = "relational_home/durable_run.rs"]
mod durable_run;
#[path = "relational_home/identity_safety.rs"]
mod identity_safety;
#[path = "relational_home/journeys.rs"]
mod journeys;
#[path = "relational_home/killed_child.rs"]
mod killed_child;
#[path = "relational_home/media_case.rs"]
mod media_case;
#[path = "relational_home/media_faults.rs"]
mod media_faults;
#[path = "relational_home/media_sweep.rs"]
mod media_sweep;
#[path = "relational_home/open_refusals.rs"]
mod open_refusals;
#[path = "relational_home/prepared_home.rs"]
mod prepared_home;
#[path = "relational_home/process_kill.rs"]
mod process_kill;
#[path = "relational_home/publication_refusals.rs"]
mod publication_refusals;
#[path = "relational_home/recovery_interruption.rs"]
mod recovery_interruption;
#[path = "relational_home/schema_recovery.rs"]
mod schema_recovery;
#[path = "relational_home/schema_transition.rs"]
mod schema_transition;
#[path = "relational_home/selected_checkpoint.rs"]
mod selected_checkpoint;
#[path = "relational_home/transition_oracle.rs"]
mod transition_oracle;
#[path = "relational_home/workload.rs"]
mod workload;
