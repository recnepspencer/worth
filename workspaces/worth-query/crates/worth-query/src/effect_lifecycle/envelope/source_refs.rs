use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::receipt::EffectExecutionReceipt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectEnvelopeSourceRefs {
    receipt_identity: WorthQueryEvidenceIdentity,
    lowered_identity: WorthQueryEvidenceIdentity,
    authority_artifact_identity: WorthQueryEvidenceIdentity,
    counter_snapshot_identity: WorthQueryEvidenceIdentity,
    sources_identity: WorthQueryEvidenceIdentity,
}

impl EffectEnvelopeSourceRefs {
    pub(super) fn from_receipt(receipt: &EffectExecutionReceipt) -> Self {
        let receipt_identity = receipt.receipt_identity().clone();
        let lowered_identity = receipt.decision_trace().lowered_identity().clone();
        let authority_artifact_identity = receipt
            .integrity_markers()
            .authority_artifact_identity()
            .clone();
        let counter_snapshot_identity = receipt
            .integrity_markers()
            .counter_snapshot_identity()
            .clone();
        let sources_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_envelope_source_refs_v1",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("receipt"), &receipt_identity)
                .field_evidence_identity(WorthQueryEvidenceTag::new("lowered"), &lowered_identity)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("authority_artifact"),
                    &authority_artifact_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("counters"),
                    &counter_snapshot_identity,
                )
                .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens");
        Self {
            receipt_identity,
            lowered_identity,
            authority_artifact_identity,
            counter_snapshot_identity,
            sources_identity,
        }
    }

    pub fn receipt_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }

    pub fn receipt_for_reporting(&self) -> &str {
        self.receipt_identity.as_str()
    }

    pub fn lowered_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.lowered_identity
    }

    pub fn lowered_for_reporting(&self) -> &str {
        self.lowered_identity.as_str()
    }

    pub fn authority_artifact_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.authority_artifact_identity
    }

    pub fn authority_artifact_for_reporting(&self) -> &str {
        self.authority_artifact_identity.as_str()
    }

    pub fn counter_snapshot_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.counter_snapshot_identity
    }

    pub fn counter_snapshot_for_reporting(&self) -> &str {
        self.counter_snapshot_identity.as_str()
    }

    pub fn sources_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.sources_identity
    }

    pub fn sources_for_reporting(&self) -> &str {
        self.sources_identity.as_str()
    }
}
