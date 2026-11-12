use super::WorthQueryWorkflowContributionAuthoring;
use crate::basis::ExecutionPreflightBundle;
use crate::domain_capabilities::payloads::{
    WorthQueryWorkflowContributionPosture, WorthQueryWorkflowLoweringSemantics,
    WorthQueryWorkflowRuntimeBindingSemantics, WorthQueryWorkflowRuntimeSemantics,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::workflow::{
    MutationLoweringInput, WorkflowAuthorityTargetFamily, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowFreshnessPolicy, WorkflowPreviewEvaluationClass,
};
use crate::WorthQueryEvidenceIdentity;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_runtime_bridge::facade::BridgePreviewSessionIdentity;

impl WorthQueryWorkflowContributionAuthoring {
    pub fn promotion_eligible_mutation_lowering(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        preview_session_identity: BridgePreviewSessionIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(
            WorthQueryWorkflowContributionPosture::PromotionEligible,
            semantic_code,
            detail,
            mutation_runtime_semantics(
                WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                    preview_session_identity,
                    WorkflowPreviewEvaluationClass::PromotionEligible,
                ),
                WorkflowFreshnessPolicy::AllowExplicitRebind,
            ),
        )
    }

    pub fn confirmation_required_mutation_reconciliation(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_snapshot_identity: WorthQuerySnapshotIdentity,
        authority_binding_identity: WorthQueryEvidenceIdentity,
        entity_id: EntityId,
        desired_aspect_fields: AspectFieldPatch,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_and_lowering_semantics(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
            mutation_runtime_semantics(
                WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_snapshot_identity(
                    runtime_snapshot_identity,
                ),
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            WorthQueryWorkflowLoweringSemantics::mutation(
                authority_binding_identity,
                MutationLoweringInput::IntentReconciliation {
                    entity_id,
                    desired_aspect_fields,
                },
            ),
        )
    }

    pub fn confirmation_required_mutation_reconciliation_from_preflight(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        preflight: ExecutionPreflightBundle,
        authority_binding_identity: WorthQueryEvidenceIdentity,
        entity_id: EntityId,
        desired_aspect_fields: AspectFieldPatch,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_and_lowering_semantics(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
            mutation_runtime_semantics(
                WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_bundle(preflight),
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            WorthQueryWorkflowLoweringSemantics::mutation(
                authority_binding_identity,
                MutationLoweringInput::IntentReconciliation {
                    entity_id,
                    desired_aspect_fields,
                },
            ),
        )
    }
}

fn mutation_runtime_semantics(
    binding: WorthQueryWorkflowRuntimeBindingSemantics,
    freshness_policy: WorkflowFreshnessPolicy,
) -> WorthQueryWorkflowRuntimeSemantics {
    WorthQueryWorkflowRuntimeSemantics::new(
        binding,
        WorkflowDeclarationFamily::MutationLoweringNarrow,
        WorkflowAuthorityTargetFamily::RelationalMutation,
        WorkflowCostClass::MutationLoweringNarrow,
        WorkflowBudgetClass::AuthorityTargetBounded,
        freshness_policy,
    )
}
