use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::envelope::SelfDescribingEffectEnvelope;
use super::receipt::EffectExecutionReceipt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectDiagnosticsRequest {
    include_lowered_digest: bool,
    include_counter_snapshot: bool,
    include_integrity_markers: bool,
    include_transition_rules: bool,
    include_source_refs: bool,
}

impl EffectDiagnosticsRequest {
    pub fn forensic() -> Self {
        Self {
            include_lowered_digest: true,
            include_counter_snapshot: true,
            include_integrity_markers: true,
            include_transition_rules: true,
            include_source_refs: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDiagnosticsMaterialization {
    receipt_identity: WorthQueryEvidenceIdentity,
    envelope_identity: WorthQueryEvidenceIdentity,
    detail_sections: Vec<String>,
    diagnostics_identity: WorthQueryEvidenceIdentity,
}

impl EffectDiagnosticsMaterialization {
    pub(super) fn from_receipt(
        receipt: &EffectExecutionReceipt,
        envelope: &SelfDescribingEffectEnvelope,
        request: EffectDiagnosticsRequest,
    ) -> Result<Self, IdentityDerivationDenial> {
        let receipt_identity = receipt.receipt_identity().clone();
        let envelope_identity = envelope.envelope_identity().clone();
        let mut detail_sections = vec![
            format!("family:{}", receipt.declared_effect_family().as_str()),
            format!("authority_lane:{}", receipt.authority_lane().as_str()),
            format!("basis_lane:{}", receipt.basis_lane().as_str()),
            format!(
                "trace:{}",
                receipt.decision_trace().decision_trace_for_reporting()
            ),
        ];
        if request.include_lowered_digest {
            detail_sections.push(format!(
                "lowered:{}",
                receipt.decision_trace().lowered_for_reporting()
            ));
        }
        if request.include_counter_snapshot {
            detail_sections.push(format!(
                "counters:{}",
                receipt.delivery_counters().counter_for_reporting()
            ));
        }
        if request.include_integrity_markers {
            detail_sections.push(format!(
                "integrity:{}",
                receipt.integrity_markers().integrity_for_reporting()
            ));
        }
        if request.include_transition_rules {
            detail_sections.push(format!(
                "transitions:{}",
                receipt.transition_rules()?.rules_for_reporting()
            ));
        }
        if request.include_source_refs {
            detail_sections.push(format!(
                "sources:{}",
                envelope.sources().sources_for_reporting()
            ));
        }
        let diagnostics_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_diagnostics_materialization_v1",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("receipt"), &receipt_identity)
                .field_evidence_identity(WorthQueryEvidenceTag::new("envelope"), &envelope_identity)
                .seal()?;
        Ok(Self {
            receipt_identity,
            envelope_identity,
            detail_sections,
            diagnostics_identity,
        })
    }

    pub fn receipt_for_reporting(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn envelope_for_reporting(&self) -> &str {
        self.envelope_identity.as_str()
    }

    pub fn detail_sections(&self) -> &[String] {
        &self.detail_sections
    }

    pub fn diagnostics_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.diagnostics_identity
    }

    pub fn diagnostics_for_reporting(&self) -> &str {
        self.diagnostics_identity.as_str()
    }
}
