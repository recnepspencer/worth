use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::{
    WorthQueryOrdinaryAuthorityAdmission, WorthQueryPreviewCloseoutKind, WorthQueryPreviewOutcome,
};
use crate::session_label::WorthQuerySessionLabel;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPromotionEligibility {
    session_label: WorthQuerySessionLabel,
    snapshot_identity: WorthQuerySnapshotIdentity,
    eligibility_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryPromotionEligibility {
    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.session_label
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.eligibility_identity
    }

    pub fn identity_for_reporting(&self) -> &str {
        self.eligibility_identity.as_str()
    }

    pub(crate) fn from_authority(
        authority: &WorthQueryOrdinaryAuthorityAdmission,
    ) -> Result<Self, IdentityDerivationDenial> {
        let session_label = authority
            .session_label()
            .expect("preview authority must carry a session label")
            .clone();
        let snapshot_identity = authority.snapshot_identity().clone();
        let eligibility_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::PreviewPromotionContinuation,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("role"),
            "ordinary-promotion-eligibility",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("authority"),
            authority.admission_identity(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("snapshot"),
            &snapshot_identity.evidence_identity(),
        )
        .seal()?;
        Ok(Self {
            session_label,
            snapshot_identity,
            eligibility_identity,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedWorkflowEffect {
    effect_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryAdmittedWorkflowEffect {
    pub fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.effect_identity
    }

    pub(crate) fn new(
        declaration_identity: &WorthQueryEvidenceIdentity,
        authority: &WorthQueryOrdinaryAuthorityAdmission,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            effect_identity: WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::PreviewExecutionEvidence,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "ordinary-admitted-workflow-effect",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration"),
                declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("authority"),
                authority.admission_identity(),
            )
            .seal()?,
        })
    }

    pub(crate) fn from_effect_lifecycle(effect_identity: WorthQueryEvidenceIdentity) -> Self {
        Self { effect_identity }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLoweredWorkflowPlan {
    request_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLoweredWorkflowPlan {
    pub fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub(crate) fn new(request_identity: WorthQueryEvidenceIdentity) -> Self {
        Self { request_identity }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowExecution {
    admitted_effect: WorthQueryAdmittedWorkflowEffect,
    lowered_plan: WorthQueryLoweredWorkflowPlan,
}

impl WorthQueryWorkflowExecution {
    pub fn admitted_effect(&self) -> &WorthQueryAdmittedWorkflowEffect {
        &self.admitted_effect
    }

    pub fn lowered_plan(&self) -> &WorthQueryLoweredWorkflowPlan {
        &self.lowered_plan
    }

    pub(crate) fn new(
        admitted_effect: WorthQueryAdmittedWorkflowEffect,
        lowered_plan: WorthQueryLoweredWorkflowPlan,
    ) -> Self {
        Self {
            admitted_effect,
            lowered_plan,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowAftermath {
    closeout_kind: WorthQueryPreviewCloseoutKind,
    receipt_identity: WorthQueryEvidenceIdentity,
    aftermath_identity: WorthQueryEvidenceIdentity,
    inspection_identity: Option<WorthQueryEvidenceIdentity>,
}

impl WorthQueryWorkflowAftermath {
    pub fn closeout_kind(&self) -> WorthQueryPreviewCloseoutKind {
        self.closeout_kind
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.aftermath_identity
    }

    pub fn inspection_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.inspection_identity.as_ref()
    }

    pub(crate) fn new(
        outcome: &WorthQueryPreviewOutcome,
        receipt_identity: WorthQueryEvidenceIdentity,
        aftermath_identity: WorthQueryEvidenceIdentity,
        inspection_identity: Option<WorthQueryEvidenceIdentity>,
    ) -> Self {
        Self {
            closeout_kind: outcome.closeout_evidence().kind(),
            receipt_identity,
            aftermath_identity,
            inspection_identity,
        }
    }
}
