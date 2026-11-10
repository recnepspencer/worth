use super::WorthQueryWorkflowContributionAuthoring;
use crate::basis::ExecutionPreflightBundle;
use crate::domain_capabilities::payloads::{
    WorthQueryWorkflowContributionPosture, WorthQueryWorkflowLoweringSemantics,
    WorthQueryWorkflowRuntimeBindingSemantics, WorthQueryWorkflowRuntimeSemantics,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::workflow::{
    WorkflowAuthorityTargetFamily, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowFreshnessPolicy, WorkflowPreviewEvaluationClass,
    WritebackLoweringInput,
};
use worth_runtime_bridge::facade::BridgePreviewSessionIdentity;

impl WorthQueryWorkflowContributionAuthoring {
    pub fn confirmation_required_writeback_projected_state_diff(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_and_lowering_semantics(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
            WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_snapshot_identity(
                    runtime_snapshot_identity,
                ),
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowCostClass::WritebackLoweringNarrow,
                WorkflowBudgetClass::AuthorityTargetBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            WorthQueryWorkflowLoweringSemantics::writeback(
                WritebackLoweringInput::projected_state_diff(),
            ),
        )
    }

    pub fn confirmation_required_writeback_projected_state_diff_from_preflight(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        preflight: ExecutionPreflightBundle,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_and_lowering_semantics(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
            WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_bundle(preflight),
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowCostClass::WritebackLoweringNarrow,
                WorkflowBudgetClass::AuthorityTargetBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            WorthQueryWorkflowLoweringSemantics::writeback(
                WritebackLoweringInput::projected_state_diff(),
            ),
        )
    }

    pub fn discard_required_writeback_projected_state_diff(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        preview_session_identity: BridgePreviewSessionIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_and_lowering_semantics(
            WorthQueryWorkflowContributionPosture::DiscardRequired,
            semantic_code,
            detail,
            WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                    preview_session_identity,
                    WorkflowPreviewEvaluationClass::PromotionEligible,
                ),
                WorkflowDeclarationFamily::WritebackLoweringNarrow,
                WorkflowAuthorityTargetFamily::BridgeWriteback,
                WorkflowCostClass::WritebackLoweringNarrow,
                WorkflowBudgetClass::AuthorityTargetBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            WorthQueryWorkflowLoweringSemantics::writeback(
                WritebackLoweringInput::projected_state_diff(),
            ),
        )
    }
}
