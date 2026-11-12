use super::super::*;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryAspectTouch;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreviewExecutionKind {
    LivePatch,
    ComputedPatch,
    EffectDelivery,
    PendingWriteIntent,
    MutedEffect,
}

impl WorthQueryPreviewExecutionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LivePatch => "live-patch",
            Self::ComputedPatch => "computed-patch",
            Self::EffectDelivery => "effect-delivery",
            Self::PendingWriteIntent => "pending-write-intent",
            Self::MutedEffect => "muted-effect",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreviewExecutionEvidence {
    session_label: WorthQuerySessionLabel,
    pub(in crate::runtime::preview) kind: WorthQueryPreviewExecutionKind,
    handle_name: String,
    source_lane: WorthQueryAuthorityLane,
    preview_lane: WorthQueryAuthorityLane,
    source_evidence_identity: WorthQueryEvidenceIdentity,
    aspect_touches: Vec<WorthQueryAspectTouch>,
    intent_strategy_name: Option<String>,
    execution_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryPreviewExecutionEvidence {
    pub(in crate::runtime::preview) fn for_aspect_touches(
        basis_admission: &WorthQueryPreviewBasisAdmission,
        kind: WorthQueryPreviewExecutionKind,
        handle_name: &str,
        source_lane: WorthQueryAuthorityLane,
        preview_lane: WorthQueryAuthorityLane,
        source_evidence_identity: &WorthQueryEvidenceIdentity,
        aspect_touches: Vec<WorthQueryAspectTouch>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Self::new(
            basis_admission,
            kind,
            handle_name,
            source_lane,
            preview_lane,
            source_evidence_identity,
            aspect_touches,
            None,
        )
    }

    pub(in crate::runtime::preview) fn for_intent_strategy(
        basis_admission: &WorthQueryPreviewBasisAdmission,
        handle_name: &str,
        source_evidence_identity: &WorthQueryEvidenceIdentity,
        intent_strategy_name: &str,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Self::new(
            basis_admission,
            WorthQueryPreviewExecutionKind::PendingWriteIntent,
            handle_name,
            WorthQueryAuthorityLane::PendingWriteIntent,
            WorthQueryAuthorityLane::PreviewTruth,
            source_evidence_identity,
            Vec::new(),
            Some(intent_strategy_name.to_string()),
        )
    }

    fn new(
        basis_admission: &WorthQueryPreviewBasisAdmission,
        kind: WorthQueryPreviewExecutionKind,
        handle_name: &str,
        source_lane: WorthQueryAuthorityLane,
        preview_lane: WorthQueryAuthorityLane,
        source_evidence_identity: &WorthQueryEvidenceIdentity,
        aspect_touches: Vec<WorthQueryAspectTouch>,
        intent_strategy_name: Option<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let execution_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::PreviewExecutionEvidence)
                .field_value(
                    WorthQueryEvidenceTag::new("session_label_identity"),
                    basis_admission.label_identity().as_str(),
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .field_shape(WorthQueryEvidenceTag::new("handle_name"), handle_name)
                .field_shape(
                    WorthQueryEvidenceTag::new("source_lane"),
                    source_lane.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("preview_lane"),
                    preview_lane.as_str(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("source_evidence_identity"),
                    source_evidence_identity,
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("admitted_aspect_touch"),
                    aspect_touches
                        .iter()
                        .map(|touch| touch.admitted_touch_digest_part()),
                )
                .optional_shape(
                    WorthQueryEvidenceTag::new("intent_strategy_name"),
                    intent_strategy_name.as_deref(),
                )
                .seal()?;
        Ok(Self {
            session_label: basis_admission.session_label().clone(),
            kind,
            handle_name: handle_name.to_string(),
            source_lane,
            preview_lane,
            source_evidence_identity: source_evidence_identity.clone(),
            aspect_touches,
            intent_strategy_name,
            execution_identity,
        })
    }

    pub fn label(&self) -> &str {
        self.session_label.display()
    }

    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.session_label
    }

    pub fn label_identity(&self) -> &crate::WorthQueryEvidenceIdentity {
        self.session_label.identity_digest()
    }

    pub fn kind(&self) -> WorthQueryPreviewExecutionKind {
        self.kind
    }

    pub fn handle_name(&self) -> &str {
        &self.handle_name
    }

    pub fn source_lane(&self) -> WorthQueryAuthorityLane {
        self.source_lane
    }

    pub fn preview_lane(&self) -> WorthQueryAuthorityLane {
        self.preview_lane
    }

    pub fn source_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_evidence_identity
    }

    pub fn aspect_touches(&self) -> &[WorthQueryAspectTouch] {
        &self.aspect_touches
    }

    pub fn intent_strategy_name(&self) -> Option<&str> {
        self.intent_strategy_name.as_deref()
    }

    pub fn execution_digest(&self) -> &str {
        self.execution_identity.as_str()
    }

    pub fn execution_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.execution_identity
    }
}
