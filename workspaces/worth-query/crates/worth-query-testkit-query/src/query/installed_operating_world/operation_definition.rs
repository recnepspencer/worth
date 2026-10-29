pub use worth_query::facade::domain;

pub use super::installed_operation_fixture::{
    canonical_bundle, conflicting_workspace, lowering_mismatch_workspace,
    mismatched_cost_workspace, mismatched_determinism_workspace, mismatched_read_plan_workspace,
    operation_identity_contract, semantic_drift_workspace, unsupported_direct_effect_workspace,
    workspace, GeometryDomain, ReadFamily, ReadVertex, ReadVertexLookalike,
};

pub struct DriftTrigger;
impl domain::WorthQueryOnDemandTriggerFamily for DriftTrigger {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.triggers.operation-drift";
}

#[path = "operation_definition/every_downstream_semantic_role_participates_in_atomic_installati.rs"]
pub mod every_downstream_semantic_role_participates_in_atomic_installati;
