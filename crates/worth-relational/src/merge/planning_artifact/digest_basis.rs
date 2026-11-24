use std::sync::Arc;
use worth_relational_contracts::internal_port::history::data::merge_branch_basis::ResolvedMergeBaseOwnerPort as _;

use crate::merge::data::{
    LoweredMergePlan, MergeArtifactDigestBasis, MergeBaseDigestBasis, MergeCausalDigestBasis,
    MergeExecutionAuthorityContract, MergeIdentityDigestBasis, MergeRequestDigestBasis,
    MergeSchemaSnapshotDigestBasis,
};

use super::conflict_digest_basis::merge_conflict_digest_basis;
use super::lowered_plan_digest_basis::merge_lowered_plan_digest_basis;
use super::policy_digest_basis::merge_policy_digest_basis;

pub(super) fn merge_artifact_digest_basis(
    plan: &LoweredMergePlan,
    schema_snapshot: MergeSchemaSnapshotDigestBasis,
    execution_authority_contract: MergeExecutionAuthorityContract,
) -> MergeArtifactDigestBasis {
    MergeArtifactDigestBasis {
        request: merge_request_digest_basis(plan),
        schema: schema_snapshot,
        execution_contract: execution_authority_contract,
        merge_base: merge_base_digest_basis(plan),
        identity: merge_identity_digest_basis(plan),
        causal: merge_causal_digest_basis(plan),
        conflict: merge_conflict_digest_basis(plan),
        policy: merge_policy_digest_basis(plan),
        lowered_plan: merge_lowered_plan_digest_basis(plan),
        decision_log: plan.decision_log_digest_basis.clone(),
    }
}

fn merge_request_digest_basis(plan: &LoweredMergePlan) -> MergeRequestDigestBasis {
    MergeRequestDigestBasis {
        target_branch: plan.request.target_branch().clone(),
        source_branch: plan.request.source_branch().clone(),
        merge_intent: plan.request.merge_intent(),
        family: plan.request.family(),
        correspondence_posture: plan.request.correspondence_posture(),
        schema_reconciliation_posture: plan.request.schema_reconciliation_posture(),
        topology_intent: plan.request.topology_intent(),
    }
}

fn merge_base_digest_basis(plan: &LoweredMergePlan) -> MergeBaseDigestBasis {
    MergeBaseDigestBasis {
        rule: plan.basis.merge_base().rule(),
        commit_id: plan.basis.merge_base().commit().commit_id,
        supporting_left_ancestors: plan.basis.merge_base().left_ancestors().clone(),
        supporting_right_ancestors: plan.basis.merge_base().right_ancestors().clone(),
    }
}

fn merge_identity_digest_basis(plan: &LoweredMergePlan) -> MergeIdentityDigestBasis {
    MergeIdentityDigestBasis {
        effective_declarations: plan.effective_identity_declarations.clone(),
        candidate_scopes: Arc::from(
            plan.candidates
                .iter()
                .map(|candidate| candidate.scope.clone())
                .collect::<Vec<_>>(),
        ),
        candidate_sources: Arc::from(
            plan.candidates
                .iter()
                .map(|candidate| candidate.source_record.clone())
                .collect::<Vec<_>>(),
        ),
        candidate_targets: Arc::from(
            plan.candidates
                .iter()
                .map(|candidate| candidate.target_record.clone())
                .collect::<Vec<_>>(),
        ),
        candidate_bases: Arc::from(
            plan.candidates
                .iter()
                .map(|candidate| candidate.basis.clone())
                .collect::<Vec<_>>(),
        ),
        candidate_match_classes: Arc::from(
            plan.candidates
                .iter()
                .map(|candidate| candidate.match_class.clone())
                .collect::<Vec<_>>(),
        ),
        candidate_reasons: Arc::from(
            plan.candidates
                .iter()
                .map(|candidate| candidate.reason.clone())
                .collect::<Vec<_>>(),
        ),
    }
}

fn merge_causal_digest_basis(plan: &LoweredMergePlan) -> MergeCausalDigestBasis {
    MergeCausalDigestBasis {
        records: Arc::from(
            plan.causal_annotations
                .iter()
                .map(|annotation| annotation.record.clone())
                .collect::<Vec<_>>(),
        ),
        dispositions: Arc::from(
            plan.causal_annotations
                .iter()
                .map(|annotation| annotation.disposition)
                .collect::<Vec<_>>(),
        ),
    }
}
