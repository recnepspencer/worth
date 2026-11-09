use super::WorthQueryWorkflowContributionAuthoring;
use crate::domain_capabilities::payloads::{
    WorthQueryWorkflowContributionPosture, WorthQueryWorkflowLoweringSemantics,
    WorthQueryWorkflowRuntimeBindingSemantics, WorthQueryWorkflowRuntimeSemantics,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::workflow::{
    MergeLoweringInput, WorkflowAuthorityTargetFamily, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowFreshnessPolicy, WorkflowPreviewEvaluationClass,
};
use worth_relational::facade::history::BranchId;
use worth_runtime_bridge::facade::BridgePreviewSessionIdentity;

impl WorthQueryWorkflowContributionAuthoring {
    pub fn promotion_eligible_merge_reconciliation(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        preview_session_identity: BridgePreviewSessionIdentity,
        target_branch: BranchId,
        source_branch: BranchId,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_and_lowering_semantics(
            WorthQueryWorkflowContributionPosture::PromotionEligible,
            semantic_code,
            detail,
            WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                    preview_session_identity,
                    WorkflowPreviewEvaluationClass::PromotionEligible,
                ),
                WorkflowDeclarationFamily::MergeLoweringNarrow,
                WorkflowAuthorityTargetFamily::RelationalMerge,
                WorkflowCostClass::MergeLoweringNarrow,
                WorkflowBudgetClass::AuthorityTargetBounded,
                WorkflowFreshnessPolicy::AllowExplicitRebind,
            ),
            WorthQueryWorkflowLoweringSemantics::merge(MergeLoweringInput::reconcile_into_target(
                target_branch,
                source_branch,
            )),
        )?)
    }
}
