#![cfg_attr(test, allow(clippy::disallowed_methods))]
//! Query execution proofs grouped by their product frontier.
#![recursion_limit = "256"]
mod compile_fail;
mod cost;
#[path = "worth_query_court_execution/domain_computation/mod.rs"]
pub mod domain_computation;
mod naming_bounded_depth;
mod naming_bridge_continuity;
mod naming_bridge_planning;
mod naming_cascade_retirement;
mod naming_cold_directory_recovery;
mod naming_cold_source_fixture;
mod naming_decision_reader;
mod naming_discriminator_agreement;
mod naming_durable_recovery;
mod naming_execution_birth_wiring;
mod naming_execution_fixture;
mod naming_fork_ancestry;
mod naming_group_durable_recovery;
mod naming_group_soundness;
mod naming_operation_outputs;
mod naming_posture_agreement;
mod naming_postures_e2e;
mod naming_reorder_and_collision;
mod naming_resolution_agreement;
mod naming_same_commit_names;
mod naming_same_commit_refusals;
mod naming_schema_authority_reopen;
mod naming_tracked_birth_kinds;
mod naming_tracked_births;
mod naming_tracked_cold_directory;
mod naming_tracked_cost;
mod naming_tracked_dependency_cost;
mod naming_tracked_fork_settlement;
mod naming_tracked_invalidation;
mod naming_tracked_rebuild;
mod naming_tracked_resolution;
mod naming_tracked_world;

mod query_obligations;
