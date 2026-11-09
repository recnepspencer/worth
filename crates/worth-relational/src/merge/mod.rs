mod access;
mod aspect_components;
mod aspect_plan_lookup;
mod aspect_witness_digest;
mod causal;
mod conflicts;
mod correspondence_witness;
pub mod data;
mod execution;
mod execution_diagnostics;
mod execution_mutation_plan;
pub mod facade;
mod identity;
mod identity_digest;
mod identity_records;
mod identity_target_index;
mod lowering;
mod planning;
mod planning_artifact;
mod planning_observation;
mod policy;
mod proof_packet;
mod proof_packet_canonical;
mod request_foundational_lowering;
mod request_normalization;
mod schema_reconciliation_witness;
mod strategy_witness;

pub use access::MergeAccess;
pub(crate) use execution_diagnostics::{
    merge_execution_success_artifact, merge_execution_summary_entry,
};
pub(crate) use lowering::{
    blocked_reason_for_deletion_class, blocked_reason_for_topology_resolution_class,
};
pub(crate) use planning_artifact::lowered_artifact_execution_authority_contract;
pub use planning_observation::RelationalMergePlanningObservation;
pub(crate) use policy::{aggregate_record_resolution, ownership_surface_for_policies};
