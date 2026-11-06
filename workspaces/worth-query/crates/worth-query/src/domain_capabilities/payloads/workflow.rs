use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

use super::common::{
    SealedPayload, WorthQueryDomainCapabilityCategory, WorthQueryDomainCapabilityPayload,
    WorthQueryDomainCapabilitySemanticPosture,
};
use super::{
    WorthQueryWorkflowInspectionSemantics, WorthQueryWorkflowLoweringSemantics,
    WorthQueryWorkflowRuntimeSemantics,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowContributionPosture {
    PreviewOnly,
    PromotionEligible,
    ConfirmationRequired,
    DiscardRequired,
}

impl WorthQueryWorkflowContributionPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreviewOnly => "preview-only",
            Self::PromotionEligible => "promotion-eligible",
            Self::ConfirmationRequired => "confirmation-required",
            Self::DiscardRequired => "discard-required",
        }
    }

    pub const fn semantic_posture(self) -> WorthQueryDomainCapabilitySemanticPosture {
        match self {
            Self::PreviewOnly => WorthQueryDomainCapabilitySemanticPosture::WorkflowPreviewOnly,
            Self::PromotionEligible => {
                WorthQueryDomainCapabilitySemanticPosture::WorkflowPromotionEligible
            }
            Self::ConfirmationRequired => {
                WorthQueryDomainCapabilitySemanticPosture::WorkflowConfirmationRequired
            }
            Self::DiscardRequired => {
                WorthQueryDomainCapabilitySemanticPosture::WorkflowDiscardRequired
            }
        }
    }
}

fn compose_workflow_payload_identity(
    posture: WorthQueryWorkflowContributionPosture,
    semantic_code: &str,
    detail: &str,
    runtime_semantics: Option<&WorthQueryWorkflowRuntimeSemantics>,
    lowering_semantics: Option<&WorthQueryWorkflowLoweringSemantics>,
    inspection_semantics: Option<&WorthQueryWorkflowInspectionSemantics>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let mut identity = domain_capability_scope_encoder("worth_query_domain_capability_payload_v5")
        .field_shape(
            WorthQueryEvidenceTag::new("category"),
            WorthQueryDomainCapabilityCategory::WorkflowPreview.as_str(),
        )
        .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
        .field_shape(WorthQueryEvidenceTag::new("semantic_code"), semantic_code)
        .field_shape(WorthQueryEvidenceTag::new("detail"), detail);
    identity = match runtime_semantics {
        Some(runtime) => identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("runtime"),
            &runtime.semantics_identity(),
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("runtime"), "none"),
    };
    identity = match lowering_semantics {
        Some(lowering) => identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("lowering"),
            &lowering.semantics_identity(),
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("lowering"), "none"),
    };
    identity = match inspection_semantics {
        Some(inspection) => identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("inspection"),
            &inspection.semantics_identity(),
        ),
        None => identity.field_shape(WorthQueryEvidenceTag::new("inspection"), "none"),
    };
    Ok(identity.seal()?)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowContributionPayload {
    posture: WorthQueryWorkflowContributionPosture,
    semantic_code: String,
    detail: String,
    runtime_semantics: Option<WorthQueryWorkflowRuntimeSemantics>,
    lowering_semantics: Option<WorthQueryWorkflowLoweringSemantics>,
    inspection_semantics: Option<WorthQueryWorkflowInspectionSemantics>,
    payload_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryWorkflowContributionPayload {
    pub fn new(
        posture: WorthQueryWorkflowContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_all_semantics(
            posture,
            semantic_code,
            detail,
            None,
            None,
            None,
        )?)
    }

    pub fn with_runtime_semantics(
        posture: WorthQueryWorkflowContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: Option<WorthQueryWorkflowRuntimeSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_all_semantics(
            posture,
            semantic_code,
            detail,
            runtime_semantics,
            None,
            None,
        )?)
    }

    pub fn with_runtime_and_lowering_semantics(
        posture: WorthQueryWorkflowContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: Option<WorthQueryWorkflowRuntimeSemantics>,
        lowering_semantics: Option<WorthQueryWorkflowLoweringSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_all_semantics(
            posture,
            semantic_code,
            detail,
            runtime_semantics,
            lowering_semantics,
            None,
        )?)
    }

    pub fn with_runtime_and_inspection_semantics(
        posture: WorthQueryWorkflowContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: Option<WorthQueryWorkflowRuntimeSemantics>,
        inspection_semantics: Option<WorthQueryWorkflowInspectionSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_all_semantics(
            posture,
            semantic_code,
            detail,
            runtime_semantics,
            None,
            inspection_semantics,
        )?)
    }

    pub fn with_all_semantics(
        posture: WorthQueryWorkflowContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: Option<WorthQueryWorkflowRuntimeSemantics>,
        lowering_semantics: Option<WorthQueryWorkflowLoweringSemantics>,
        inspection_semantics: Option<WorthQueryWorkflowInspectionSemantics>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let semantic_code = semantic_code.into();
        let detail = detail.into();
        let payload_identity = compose_workflow_payload_identity(
            posture,
            &semantic_code,
            &detail,
            runtime_semantics.as_ref(),
            lowering_semantics.as_ref(),
            inspection_semantics.as_ref(),
        )?;
        Ok(Self {
            posture,
            semantic_code,
            detail,
            runtime_semantics,
            lowering_semantics,
            inspection_semantics,
            payload_identity,
        })
    }

    pub fn category(&self) -> WorthQueryDomainCapabilityCategory {
        WorthQueryDomainCapabilityCategory::WorkflowPreview
    }

    pub fn posture(&self) -> WorthQueryWorkflowContributionPosture {
        self.posture
    }

    pub fn semantic_code(&self) -> &str {
        &self.semantic_code
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn runtime_semantics(&self) -> Option<&WorthQueryWorkflowRuntimeSemantics> {
        self.runtime_semantics.as_ref()
    }

    pub fn lowering_semantics(&self) -> Option<&WorthQueryWorkflowLoweringSemantics> {
        self.lowering_semantics.as_ref()
    }

    pub fn inspection_semantics(&self) -> Option<&WorthQueryWorkflowInspectionSemantics> {
        self.inspection_semantics.as_ref()
    }

    pub fn payload_digest(&self) -> &str {
        self.payload_identity.as_str()
    }

    pub fn payload_for_reporting(&self) -> &str {
        self.payload_identity.as_str()
    }

    pub fn payload_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.payload_identity
    }
}

impl SealedPayload for WorthQueryWorkflowContributionPayload {}

impl WorthQueryDomainCapabilityPayload for WorthQueryWorkflowContributionPayload {
    fn category(&self) -> WorthQueryDomainCapabilityCategory {
        self.category()
    }

    fn posture_label(&self) -> &'static str {
        self.posture().as_str()
    }

    fn semantic_posture(&self) -> WorthQueryDomainCapabilitySemanticPosture {
        self.posture().semantic_posture()
    }

    fn semantic_code(&self) -> &str {
        self.semantic_code()
    }

    fn detail(&self) -> &str {
        self.detail()
    }

    fn payload_digest(&self) -> &str {
        self.payload_digest()
    }

    fn payload_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.payload_identity
    }
}
