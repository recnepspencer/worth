use crate::basis_lifecycle::BasisFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::inventory::EffectReceiptArtifactKind;
use super::super::planning::EffectAuthorityOwner;
use super::super::receipt::{EffectExecutionReceipt, EffectReceiptTargetEvidence};
use super::super::support_contract::EffectDeferredNeighborFamily;
use super::super::taxonomy::{EffectAuthorityLane, EffectFamily};
use super::primary_result::EffectEnvelopePrimaryResult;
use super::source_refs::EffectEnvelopeSourceRefs;
use super::structural_delta::structural_delta_identities;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfDescribingEffectEnvelope {
    declared_effect_family: EffectFamily,
    authority_lane: EffectAuthorityLane,
    authority_owner: EffectAuthorityOwner,
    basis_lane: BasisFamily,
    receipt_family: EffectReceiptArtifactKind,
    primary_result: EffectEnvelopePrimaryResult,
    warnings: Vec<String>,
    trace_identity: WorthQueryEvidenceIdentity,
    structural_delta_identities: Vec<WorthQueryEvidenceIdentity>,
    integrity_identity: WorthQueryEvidenceIdentity,
    performance_identity: WorthQueryEvidenceIdentity,
    boundary_identity: WorthQueryEvidenceIdentity,
    transition_rules_identity: WorthQueryEvidenceIdentity,
    deferred_neighbors: Vec<EffectDeferredNeighborFamily>,
    sources: EffectEnvelopeSourceRefs,
    envelope_identity: WorthQueryEvidenceIdentity,
}

impl SelfDescribingEffectEnvelope {
    pub(in crate::effect_lifecycle) fn from_receipt(
        receipt: &EffectExecutionReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let primary_result = match receipt.target_evidence() {
            EffectReceiptTargetEvidence::MutationCommit { .. } => {
                EffectEnvelopePrimaryResult::MutationCommitted
            }
            EffectReceiptTargetEvidence::MergeCommit { .. } => {
                EffectEnvelopePrimaryResult::MergeCommitted
            }
            EffectReceiptTargetEvidence::Writeback { .. } => {
                EffectEnvelopePrimaryResult::WritebackCommitted
            }
            EffectReceiptTargetEvidence::BatchMutation { .. } => {
                EffectEnvelopePrimaryResult::BatchMutationCommitted
            }
        };
        let structural_delta_identities = structural_delta_identities(receipt);
        let trace_identity = receipt.decision_trace().decision_trace_identity().clone();
        let integrity_identity = receipt.integrity_markers().integrity_identity().clone();
        let sources = EffectEnvelopeSourceRefs::from_receipt(receipt)?;
        let transition_rules = receipt.transition_rules()?;
        let performance_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_envelope_performance_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("receipt"),
                    receipt.receipt_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("counters"),
                    &receipt.delivery_counters().evidence_identity()?,
                )
                .seal()?;
        let boundary_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_envelope_boundary_v1",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("authority_lane"),
                    receipt.authority_lane().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("basis_lane"),
                    receipt.basis_lane().as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("family"),
                    receipt.declared_effect_family().as_str(),
                )
                .seal()?;
        let deferred_neighbors = transition_rules
            .rules()
            .iter()
            .filter_map(|rule| rule.deferred_neighbor())
            .collect::<Vec<_>>();
        let transition_rules_identity = transition_rules.rules_identity().clone();
        let envelope_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "self_describing_effect_envelope_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("receipt"),
                    receipt.receipt_identity(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("primary"),
                    primary_result.as_str(),
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("delta"),
                    structural_delta_identities.iter(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("integrity"),
                    &integrity_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("performance"),
                    &performance_identity,
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("boundary"), &boundary_identity)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("transitions"),
                    &transition_rules_identity,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("sources"),
                    sources.sources_identity(),
                )
                .seal()?;
        Ok(Self {
            declared_effect_family: receipt.declared_effect_family(),
            authority_lane: receipt.authority_lane(),
            authority_owner: receipt.authority_owner(),
            basis_lane: receipt.basis_lane(),
            receipt_family: receipt.receipt_family(),
            primary_result,
            warnings: Vec::new(),
            trace_identity,
            structural_delta_identities,
            integrity_identity,
            performance_identity,
            boundary_identity,
            transition_rules_identity,
            deferred_neighbors,
            sources,
            envelope_identity,
        })
    }

    pub fn declared_effect_family(&self) -> EffectFamily {
        self.declared_effect_family
    }

    pub fn authority_lane(&self) -> EffectAuthorityLane {
        self.authority_lane
    }

    pub fn authority_owner(&self) -> EffectAuthorityOwner {
        self.authority_owner
    }

    pub fn basis_lane(&self) -> BasisFamily {
        self.basis_lane
    }

    pub fn receipt_family(&self) -> EffectReceiptArtifactKind {
        self.receipt_family
    }

    pub fn primary_result(&self) -> EffectEnvelopePrimaryResult {
        self.primary_result
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn trace_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.trace_identity
    }

    pub fn trace_for_reporting(&self) -> &str {
        self.trace_identity.as_str()
    }

    pub fn structural_delta_identities(&self) -> &[WorthQueryEvidenceIdentity] {
        &self.structural_delta_identities
    }

    pub fn integrity_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.integrity_identity
    }

    pub fn integrity_for_reporting(&self) -> &str {
        self.integrity_identity.as_str()
    }

    pub fn performance_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.performance_identity
    }

    pub fn performance_for_reporting(&self) -> &str {
        self.performance_identity.as_str()
    }

    pub fn boundary_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.boundary_identity
    }

    pub fn boundary_for_reporting(&self) -> &str {
        self.boundary_identity.as_str()
    }

    pub fn transition_rules_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.transition_rules_identity
    }

    pub fn transition_rules_for_reporting(&self) -> &str {
        self.transition_rules_identity.as_str()
    }

    pub fn deferred_neighbors(&self) -> &[EffectDeferredNeighborFamily] {
        &self.deferred_neighbors
    }

    pub fn sources(&self) -> &EffectEnvelopeSourceRefs {
        &self.sources
    }

    pub fn envelope_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.envelope_identity
    }

    pub fn envelope_for_reporting(&self) -> &str {
        self.envelope_identity.as_str()
    }
}
