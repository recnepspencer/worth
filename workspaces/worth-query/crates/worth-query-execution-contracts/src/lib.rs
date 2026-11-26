#![cfg_attr(
    not(test),
    deny(clippy::iter_over_hash_type, clippy::disallowed_methods)
)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Shared internal Query execution contracts, denial vocabularies, runtime
//! time, observation guards, and leaf primary-graph value/identity types.

#![forbid(unsafe_code)]

mod aggregate_identity;
mod attempt_denial_kind;
mod authorization_denial_kind;
mod computation_denial;
mod conditional_owner;
mod dispatch_journal_denial;
mod entity_key;
mod entity_resolution_denial;
mod execution_observation;
mod generated_output_invariant_denial;
mod home_retirement_owner;
mod keyed_entity_refusal;
mod managed_index_denial;
mod operation_admission_identity;
mod operation_home_retirement;
mod output_projection_denial;
mod partitioned_tree_observation;
mod preimage_denial;
mod product_activation_denial;
mod product_branch_denial;
mod product_stale;
mod projection_denial;
mod provider_session_denial_kind;
mod publication_access;
mod read_observation;
mod relational_execution_denial;
mod runtime_time;
mod source_facts_allocation;
mod structural_walk_memory;
mod verification_capacity_stop;

pub mod facade;
