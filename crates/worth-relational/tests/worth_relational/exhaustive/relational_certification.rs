#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::*;

#[path = "relational_certification/basis/cost.rs"]
pub(crate) mod basis_cost;

#[path = "relational_certification/delta_contracts.rs"]
pub(crate) mod delta_contracts;

#[path = "relational_certification/invariants/uniqueness/global.rs"]
pub(crate) mod invariant_global_uniqueness;

#[path = "relational_certification/invariants/selected_state/structural_selection.rs"]
pub(crate) mod invariant_structural_selection;

#[path = "relational_certification/mvcc/model_sequences.rs"]
pub(crate) mod mvcc_model_sequences;

#[path = "relational_certification/mvcc/seeded_model.rs"]
pub(crate) mod mvcc_seeded_model;

#[path = "relational_certification/observation_contracts.rs"]
pub(crate) mod observation_contracts;

#[path = "relational_certification/oracle_ancestry.rs"]
pub(crate) mod oracle_ancestry;

#[path = "relational_certification/oracle_application.rs"]
pub(crate) mod oracle_application;

#[path = "relational_certification/owner_service_completion.rs"]
pub(crate) mod owner_service_completion;

#[path = "relational_certification/preservation/recovery.rs"]
pub(crate) mod preservation_recovery;

#[path = "relational_certification/production_failures.rs"]
pub(crate) mod production_failures;

#[path = "relational_certification/production_world.rs"]
pub(crate) mod production_world;

#[path = "relational_certification/profile_contracts.rs"]
pub(crate) mod profile_contracts;

#[path = "relational_certification/read_contracts.rs"]
pub(crate) mod read_contracts;

#[path = "relational_certification/root/accounting/authoritative.rs"]
pub(crate) mod root_authoritative_accounting;

#[path = "relational_certification/root/selection/branch_isolation.rs"]
pub(crate) mod root_branch_isolation;

#[path = "relational_certification/root/cost/scale_axes.rs"]
pub(crate) mod root_cost_scale_axes;

#[path = "relational_certification/root/cost/scopes.rs"]
pub(crate) mod root_cost_scopes;

#[path = "relational_certification/root/delta/lowering_contracts.rs"]
pub(crate) mod root_delta_lowering_contracts;

#[path = "relational_certification/root/accounting/persistent_path.rs"]
pub(crate) mod root_persistent_path_accounting;

#[path = "relational_certification/schema_transition_replay.rs"]
pub(crate) mod schema_transition_replay;

#[path = "relational_certification/schema_transition_target.rs"]
pub(crate) mod schema_transition_target;

#[path = "relational_certification/trace_replay.rs"]
pub(crate) mod trace_replay;

#[path = "relational_certification/baseline_layers.rs"]
pub(crate) mod baseline_layers;

#[path = "relational_certification/basis/observation.rs"]
pub(crate) mod basis_observation;

#[path = "relational_certification/basis/read_cutover.rs"]
pub(crate) mod basis_read_cutover;

#[path = "relational_certification/basis/readmission.rs"]
pub(crate) mod basis_readmission;

#[path = "relational_certification/basis/retention.rs"]
pub(crate) mod basis_retention;

#[path = "relational_certification/comparison_contracts.rs"]
pub(crate) mod comparison_contracts;

#[path = "relational_certification/mvcc/archive_retention.rs"]
pub(crate) mod mvcc_archive_retention;

#[path = "relational_certification/mvcc/cancellation.rs"]
pub(crate) mod mvcc_cancellation;

#[path = "relational_certification/mvcc/cancellation_publication_boundaries.rs"]
pub(crate) mod mvcc_cancellation_publication_boundaries;

#[path = "relational_certification/mvcc/owner_phase_locality.rs"]
pub(crate) mod mvcc_owner_phase_locality;

#[path = "relational_certification/mvcc/retention.rs"]
pub(crate) mod mvcc_retention;

#[path = "relational_certification/reference/compatibility.rs"]
pub(crate) mod reference_compatibility;

#[path = "relational_certification/reference/cost.rs"]
pub(crate) mod reference_cost;

#[path = "relational_certification/reference/fork.rs"]
pub(crate) mod reference_fork;

#[path = "relational_certification/reference/owner_binding.rs"]
pub(crate) mod reference_owner_binding;

#[path = "relational_certification/reference/retention.rs"]
pub(crate) mod reference_retention;

#[path = "relational_certification/root/copy_on_write/branch.rs"]
pub(crate) mod root_branch_copy_on_write;

#[path = "relational_certification/root/selection/traversal.rs"]
pub(crate) mod root_branch_traversal_isolation;

#[path = "relational_certification/root/accounting/derived_cache.rs"]
pub(crate) mod root_derived_cache_accounting;

#[path = "relational_certification/root/sharing/fork.rs"]
pub(crate) mod root_fork_sharing;

#[path = "relational_certification/root/inspection/boundaries.rs"]
pub(crate) mod root_inspection_boundaries;

#[path = "relational_certification/root/copy_on_write/named_delta.rs"]
pub(crate) mod root_named_delta_cow;

#[path = "relational_certification/root/copy_on_write/region_reuse.rs"]
pub(crate) mod root_region_reuse;

#[path = "relational_certification/root/sharing/metric_truth_lanes.rs"]
pub(crate) mod root_sharing_metric_truth_lanes;

#[path = "relational_certification/schema_transition_cancellation.rs"]
pub(crate) mod schema_transition_cancellation;
