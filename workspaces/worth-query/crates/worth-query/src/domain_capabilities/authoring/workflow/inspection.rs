use super::WorthQueryWorkflowContributionAuthoring;
use crate::basis::ExecutionPreflightBundle;
use crate::domain_capabilities::payloads::{
    WorthQueryWorkflowContributionPosture, WorthQueryWorkflowRuntimeBindingSemantics,
    WorthQueryWorkflowRuntimeSemantics,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::workflow::{
    WorkflowAuthorityTargetFamily, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowFreshnessPolicy, WorkflowPreviewEvaluationClass,
};
use worth_runtime_bridge::facade::BridgePreviewSessionIdentity;

impl WorthQueryWorkflowContributionAuthoring {
    pub fn preview_only_query_inspection(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        preview_session_identity: BridgePreviewSessionIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_semantics(
            WorthQueryWorkflowContributionPosture::PreviewOnly,
            semantic_code,
            detail,
            inspection_runtime_semantics(
                WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                    preview_session_identity,
                    WorkflowPreviewEvaluationClass::ReadOnly,
                ),
            ),
        )?)
    }

    pub fn confirmation_required_query_inspection(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_semantics(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
            inspection_runtime_semantics(
                WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_snapshot_identity(
                    runtime_snapshot_identity,
                ),
            ),
        )?)
    }

    pub fn confirmation_required_query_inspection_from_preflight(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        preflight: ExecutionPreflightBundle,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_semantics(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
            inspection_runtime_semantics(
                WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_bundle(preflight),
            ),
        )?)
    }

    pub fn discard_required_query_inspection(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        preview_session_identity: BridgePreviewSessionIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_runtime_semantics(
            WorthQueryWorkflowContributionPosture::DiscardRequired,
            semantic_code,
            detail,
            inspection_runtime_semantics(
                WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                    preview_session_identity,
                    WorkflowPreviewEvaluationClass::PromotionEligible,
                ),
            ),
        )?)
    }
}

fn inspection_runtime_semantics(
    binding: WorthQueryWorkflowRuntimeBindingSemantics,
) -> WorthQueryWorkflowRuntimeSemantics {
    WorthQueryWorkflowRuntimeSemantics::new(
        binding,
        WorkflowDeclarationFamily::ConflictInspectionNarrow,
        WorkflowAuthorityTargetFamily::QueryInspection,
        WorkflowCostClass::InspectionNarrow,
        WorkflowBudgetClass::InspectionBounded,
        WorkflowFreshnessPolicy::ExactBasis,
    )
}
