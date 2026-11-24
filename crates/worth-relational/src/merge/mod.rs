mod access;
pub(crate) use worth_relational_merge::internal_port::merge::{
    branch_delta, causal, conflicts, correspondence_witness, execution_diagnostics,
    execution_mutation_plan, identity, lowering, planning_observation, proof_packet,
    proof_packet_canonical, request_foundational_lowering, request_normalization,
};
mod correspondence_witness_access;
pub use worth_relational_merge::internal_port::merge::data;
mod execution;
pub mod facade;
mod planning;
mod planning_artifact;
mod policy;
mod proof_packet_access;

pub use access::MergeAccess;
pub use planning_observation::RelationalMergePlanningObservation;

mod causal_scope;
mod conflict_scope;
mod identity_scope;
mod lowering_scope;
mod schema_reconciliation_witness_access;
mod strategy_witness_access;
