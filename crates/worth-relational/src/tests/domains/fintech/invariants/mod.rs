//! Fintech domain invariants.
//!
//! This module is intentionally a table of contents. Fixture shape, named truth
//! world guarantees, and read-surface assertions are split by concern.

mod case_workflows;
mod fixture_shape;
mod read_surfaces;
mod truth_world;

pub(super) use case_workflows::{
    assert_correction_case_transition, assert_intraday_risk_case_transition,
    assert_observability_overlap_stable, assert_observability_surfaces_agree,
    assert_recovery_matches_truth, assert_replay_targets_branch,
    assert_settlement_repair_case_transition, assert_snapshot_release_contract,
};
pub(super) use fixture_shape::assert_fixture_shape;
pub(super) use read_surfaces::{assert_cross_context_relations, assert_partitioned_aspect_state};
pub(super) use truth_world::assert_named_truth_world;
