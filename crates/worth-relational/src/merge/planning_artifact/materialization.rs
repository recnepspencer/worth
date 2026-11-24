use crate::merge::data::{LoweredMergePlan, MergePlanningArtifactCore};

use super::super::schema_reconciliation_witness::retained_schema_reconciliation_witness;
use super::super::strategy_witness::retained_merge_strategy_witness;
use super::digest_basis::merge_artifact_digest_basis;
use super::lowered_artifact_execution_authority_contract;
use super::summaries::{merge_ancestry_summary, merge_request_summary};

pub(super) fn materialize_planning_observation(
    plan: LoweredMergePlan,
    schema_snapshot: crate::merge::data::MergeSchemaSnapshotDigestBasis,
) -> super::super::planning_observation::RelationalMergePlanningObservation {
    let execution_authority_contract = lowered_artifact_execution_authority_contract();
    let digest_basis = merge_artifact_digest_basis(
        &plan,
        schema_snapshot.clone(),
        execution_authority_contract.clone(),
    );
    let schema_reconciliation_witness =
        retained_schema_reconciliation_witness(&plan, &schema_snapshot);
    let strategy_witness = retained_merge_strategy_witness(&plan, &execution_authority_contract);
    let request_summary = merge_request_summary(&plan);
    let ancestry_summary = merge_ancestry_summary(&plan);

    let artifact = MergePlanningArtifactCore {
        request: plan.request.clone(),
        branch_basis: plan.basis.clone(),
        schema_snapshot,
        schema_reconciliation_witness,
        strategy_witness,
        execution_authority_contract,
        ancestry: plan.ancestry.clone(),
        identity_discovery: plan.identity_summary.clone(),
        conflict_classification: plan.conflict_summary,
        causal_annotation: plan.causal_summary,
        policy_resolution: plan.policy_summary.clone(),
        lowered_plan: plan.lowered_summary,
        decision_log: plan.decision_log.clone(),
        digest_basis,
        decision_log_digest_basis: plan.decision_log_digest_basis.clone(),
        summary: crate::merge::data::MergePlanningSummary {
            request_summary,
            ancestry_summary,
        },
    };
    super::super::planning_observation::RelationalMergePlanningObservation::new(
        artifact,
        plan.target_delta,
        plan.source_delta,
    )
}
