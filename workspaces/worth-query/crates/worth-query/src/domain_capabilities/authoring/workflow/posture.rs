use super::WorthQueryWorkflowContributionAuthoring;
use crate::domain_capabilities::payloads::WorthQueryWorkflowContributionPosture;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl WorthQueryWorkflowContributionAuthoring {
    pub fn preview_only(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryWorkflowContributionPosture::PreviewOnly,
            semantic_code,
            detail,
        )?)
    }

    pub fn promotion_eligible(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryWorkflowContributionPosture::PromotionEligible,
            semantic_code,
            detail,
        )?)
    }

    pub fn confirmation_required(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryWorkflowContributionPosture::ConfirmationRequired,
            semantic_code,
            detail,
        )?)
    }

    pub fn discard_required(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryWorkflowContributionPosture::DiscardRequired,
            semantic_code,
            detail,
        )?)
    }
}
