use super::super::super::{
    WorthQueryAuthorityLane, WorthQueryEffectPolicy, WorthQueryPreviewHandleBindingEvidence,
    WorthQueryPreviewHandleBindingFamily,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::session_label::WorthQuerySessionLabel;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreviewBindingInspection {
    label: WorthQuerySessionLabel,
    handle_name: String,
    family: WorthQueryPreviewHandleBindingFamily,
    source_lane: WorthQueryAuthorityLane,
    preview_lane: WorthQueryAuthorityLane,
    effect_policy: WorthQueryEffectPolicy,
    effect_disposition: Option<String>,
    basis_evidence: Vec<String>,
    basis_digest: String,
    effect_delivery_admitted: bool,
    pending_write_intent_admitted: bool,
    authoritative_side_effect_admitted: bool,
    admission_digest: String,
    inspection_digest: String,
}

impl WorthQueryPreviewBindingInspection {
    pub(in crate::runtime) fn from_binding(
        binding: &WorthQueryPreviewHandleBindingEvidence,
    ) -> Result<Self, IdentityDerivationDenial> {
        let basis_evidence = binding.basis_evidence().to_vec();
        let basis_digest = worth_query_evidence_identity(
            WorthQueryEvidenceScope::PreviewBindingInspectionArtifact,
        )
        .field_shape(WorthQueryEvidenceTag::new("artifact_kind"), "basis")
        .field_value(
            WorthQueryEvidenceTag::new("session_label_identity"),
            binding.label_identity().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("binding_family"),
            binding.family().as_str(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("basis_evidence"),
            basis_evidence.iter().map(String::as_str),
        )
        .seal()?
        .as_str()
        .to_string();
        let effect_disposition = binding
            .effect_disposition()
            .map(|disposition| disposition.as_str().to_string());
        let admission_digest = worth_query_evidence_identity(
            WorthQueryEvidenceScope::PreviewBindingInspectionArtifact,
        )
        .field_shape(WorthQueryEvidenceTag::new("artifact_kind"), "admission")
        .field_value(
            WorthQueryEvidenceTag::new("session_label_identity"),
            binding.label_identity().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("handle_name"),
            binding.handle_name(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("binding_family"),
            binding.family().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("source_lane"),
            binding.source_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("preview_lane"),
            binding.preview_lane().as_str(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("effect_policy"),
            binding.effect_policy().as_str(),
        )
        .optional_shape(
            WorthQueryEvidenceTag::new("effect_disposition"),
            effect_disposition.as_deref(),
        )
        .field_value(WorthQueryEvidenceTag::new("basis_digest"), &basis_digest)
        .field_bool(
            WorthQueryEvidenceTag::new("effect_delivery_admitted"),
            binding.effect_delivery_admitted(),
        )
        .field_bool(
            WorthQueryEvidenceTag::new("pending_write_intent_admitted"),
            binding.pending_write_intent_admitted(),
        )
        .field_bool(
            WorthQueryEvidenceTag::new("authoritative_side_effect_admitted"),
            binding.authoritative_side_effect_admitted(),
        )
        .seal()?
        .as_str()
        .to_string();
        let inspection_digest = worth_query_evidence_identity(
            WorthQueryEvidenceScope::PreviewBindingInspectionArtifact,
        )
        .field_shape(WorthQueryEvidenceTag::new("artifact_kind"), "inspection")
        .field_value(
            WorthQueryEvidenceTag::new("admission_digest"),
            &admission_digest,
        )
        .seal()?
        .as_str()
        .to_string();

        Ok(Self {
            label: binding.session_label().clone(),
            handle_name: binding.handle_name().to_string(),
            family: binding.family(),
            source_lane: binding.source_lane(),
            preview_lane: binding.preview_lane(),
            effect_policy: binding.effect_policy(),
            effect_disposition,
            basis_evidence,
            basis_digest,
            effect_delivery_admitted: binding.effect_delivery_admitted(),
            pending_write_intent_admitted: binding.pending_write_intent_admitted(),
            authoritative_side_effect_admitted: binding.authoritative_side_effect_admitted(),
            admission_digest,
            inspection_digest,
        })
    }

    pub fn label(&self) -> &str {
        self.label.display()
    }

    pub fn session_label(&self) -> &WorthQuerySessionLabel {
        &self.label
    }
    pub fn handle_name(&self) -> &str {
        &self.handle_name
    }
    pub fn family(&self) -> WorthQueryPreviewHandleBindingFamily {
        self.family
    }
    pub fn source_lane(&self) -> WorthQueryAuthorityLane {
        self.source_lane
    }
    pub fn preview_lane(&self) -> WorthQueryAuthorityLane {
        self.preview_lane
    }
    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }
    pub fn effect_disposition(&self) -> Option<&str> {
        self.effect_disposition.as_deref()
    }
    pub fn basis_evidence(&self) -> &[String] {
        &self.basis_evidence
    }
    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }
    pub fn effect_delivery_admitted(&self) -> bool {
        self.effect_delivery_admitted
    }
    pub fn pending_write_intent_admitted(&self) -> bool {
        self.pending_write_intent_admitted
    }
    pub fn authoritative_side_effect_admitted(&self) -> bool {
        self.authoritative_side_effect_admitted
    }
    pub fn admission_digest(&self) -> &str {
        &self.admission_digest
    }
    pub fn inspection_digest(&self) -> &str {
        &self.inspection_digest
    }
}
