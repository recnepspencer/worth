//! A Relational home on Worth Store: journeys and the crash-consistency
//! oracle. The reference for every claim is a memory-only Relational runtime
//! executing the same script.

#[path = "relational_home/batches.rs"]
mod batches;
#[path = "relational_home/durable_run.rs"]
mod durable_run;
#[path = "relational_home/journeys.rs"]
mod journeys;
#[path = "relational_home/schema_recovery.rs"]
mod schema_recovery;
#[path = "relational_home/schema_transition.rs"]
mod schema_transition;
#[path = "relational_home/workload.rs"]
mod workload;
