use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_relational::facade::merge::RelationalMergeInspectionArtifact;

use crate::domain_capabilities::payloads::{
    WorthQueryWorkflowContributionPayload, WorthQueryWorkflowContributionPosture,
    WorthQueryWorkflowInspectionSemantics, WorthQueryWorkflowRuntimeBindingSemantics,
    WorthQueryWorkflowRuntimeSemantics,
};
use crate::workflow::{
    LoweredMergeWorkflowDeclaration, QueryWritebackDeclaration, WorkflowAuthorityTargetFamily,
    WorkflowBudgetClass, WorkflowCostClass, WorkflowDeclarationFamily, WorkflowFreshnessPolicy,
    WorkflowPreviewEvaluationClass,
};

use super::workflow::WorthQueryWorkflowContributionAuthoring;

impl WorthQueryWorkflowContributionAuthoring {
    pub fn promotion_eligible_merge_conflict_inspection(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        lowered_merge: LoweredMergeWorkflowDeclaration,
        relational_inspection: RelationalMergeInspectionArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        let preview_session_identity = lowered_merge
            .declaration()
            .binding()
            .preview_session_identity()
            .expect(
                "promotion-eligible merge conflict inspection requires preview foundation binding",
            )
            .clone();
        Ok(Self::with_runtime_and_inspection_semantics(
            WorthQueryWorkflowContributionPosture::PromotionEligible,
            semantic_code,
            detail,
            WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::preview_foundation(
                    preview_session_identity,
                    WorkflowPreviewEvaluationClass::PromotionEligible,
                ),
                WorkflowDeclarationFamily::ConflictInspectionNarrow,
                WorkflowAuthorityTargetFamily::QueryInspection,
                WorkflowCostClass::InspectionNarrow,
                WorkflowBudgetClass::InspectionBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            WorthQueryWorkflowInspectionSemantics::merge_conflict(
                lowered_merge,
                relational_inspection,
            ),
        )?)
    }

    pub fn confirmation_required_merge_conflict_inspection(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        lowered_merge: LoweredMergeWorkflowDeclaration,
        relational_inspection: RelationalMergeInspectionArtifact,
    ) -> Result<Self, IdentityDerivationDenial> {
        let runtime_snapshot_identity = lowered_merge
            .declaration()
            .binding()
            .runtime_snapshot_identity()
            .expect(
                "confirmation-required merge conflict inspection requires runtime basis binding",
            )
            .clone();
        Ok(Self::with_runtime_and_inspection_semantics(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
            WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_snapshot_identity(
                    runtime_snapshot_identity,
                ),
                WorkflowDeclarationFamily::ConflictInspectionNarrow,
                WorkflowAuthorityTargetFamily::QueryInspection,
                WorkflowCostClass::InspectionNarrow,
                WorkflowBudgetClass::InspectionBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            WorthQueryWorkflowInspectionSemantics::merge_conflict(
                lowered_merge,
                relational_inspection,
            ),
        )?)
    }

    pub fn confirmation_required_post_merge_merge_outcome_inspection(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        lowered_merge: LoweredMergeWorkflowDeclaration,
    ) -> Result<Self, IdentityDerivationDenial> {
        let runtime_snapshot_identity = lowered_merge
            .declaration()
            .binding()
            .runtime_snapshot_identity()
            .expect("post-merge merge outcome inspection requires runtime basis binding")
            .clone();
        Ok(Self::with_runtime_and_inspection_semantics(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
            WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_snapshot_identity(
                    runtime_snapshot_identity,
                ),
                WorkflowDeclarationFamily::PostMergeInspectionNarrow,
                WorkflowAuthorityTargetFamily::QueryInspection,
                WorkflowCostClass::InspectionNarrow,
                WorkflowBudgetClass::InspectionBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            WorthQueryWorkflowInspectionSemantics::post_merge_from_merge(lowered_merge),
        )?)
    }

    pub fn confirmation_required_post_merge_writeback_outcome_inspection(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        lowered_writeback: QueryWritebackDeclaration,
    ) -> Result<Self, IdentityDerivationDenial> {
        let runtime_snapshot_identity = lowered_writeback
            .declaration()
            .binding()
            .runtime_snapshot_identity()
            .expect("post-merge writeback inspection requires runtime basis binding")
            .clone();
        Ok(Self::with_runtime_and_inspection_semantics(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
            WorthQueryWorkflowRuntimeSemantics::new(
                WorthQueryWorkflowRuntimeBindingSemantics::runtime_preflight_snapshot_identity(
                    runtime_snapshot_identity,
                ),
                WorkflowDeclarationFamily::PostMergeInspectionNarrow,
                WorkflowAuthorityTargetFamily::QueryInspection,
                WorkflowCostClass::InspectionNarrow,
                WorkflowBudgetClass::InspectionBounded,
                WorkflowFreshnessPolicy::ExactBasis,
            ),
            WorthQueryWorkflowInspectionSemantics::post_merge_from_writeback(lowered_writeback),
        )?)
    }

    fn with_runtime_and_inspection_semantics(
        posture: WorthQueryWorkflowContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: WorthQueryWorkflowRuntimeSemantics,
        inspection_semantics: WorthQueryWorkflowInspectionSemantics,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryWorkflowContributionPayload::with_runtime_and_inspection_semantics(
                posture,
                semantic_code,
                detail,
                Some(runtime_semantics),
                Some(inspection_semantics),
            )?,
        })
    }
}
