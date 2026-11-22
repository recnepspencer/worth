//! Runtime forwarding shims colocated with `application_attempt`.

mod capability_and_elevation_refusal;
mod compare_and_commit;
mod elevation_commit_and_readmission;
mod idempotency_and_replay;
mod operation_body;
mod original_operation;
mod provider;
mod workflow;
mod workflow_transitions;
