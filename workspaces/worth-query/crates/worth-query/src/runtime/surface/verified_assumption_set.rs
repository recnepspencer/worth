use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::WorthQueryAspectTouch;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryVerificationReadSetBreadth {
    target_binding_count: usize,
    asserted_aspect_count: usize,
    distinct_asserted_aspect_touch_count: usize,
    cleared_assertion_count: usize,
    counter_snapshot: String,
}

impl WorthQueryVerificationReadSetBreadth {
    pub(in crate::runtime) fn new(
        target_binding_count: usize,
        asserted_aspect_count: usize,
        asserted_aspects: &[WorthQueryAspectTouch],
        cleared_assertion_count: usize,
    ) -> Self {
        let distinct_asserted_aspect_touch_count = asserted_aspects
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .len();
        let counter_snapshot = diagnostic_counter_snapshot(&[
            ("target_bindings", target_binding_count),
            ("asserted_aspects", asserted_aspect_count),
            (
                "distinct_asserted_aspect_touches",
                distinct_asserted_aspect_touch_count,
            ),
            ("cleared_assertions", cleared_assertion_count),
        ]);
        Self {
            target_binding_count,
            asserted_aspect_count,
            distinct_asserted_aspect_touch_count,
            cleared_assertion_count,
            counter_snapshot,
        }
    }

    pub fn target_binding_count(&self) -> usize {
        self.target_binding_count
    }

    pub fn asserted_aspect_count(&self) -> usize {
        self.asserted_aspect_count
    }

    pub fn distinct_asserted_aspect_touch_count(&self) -> usize {
        self.distinct_asserted_aspect_touch_count
    }

    pub fn cleared_assertion_count(&self) -> usize {
        self.cleared_assertion_count
    }

    pub fn counter_snapshot(&self) -> &str {
        &self.counter_snapshot
    }
}

fn diagnostic_counter_snapshot(fields: &[(&str, usize)]) -> String {
    let mut snapshot = String::new();
    for (index, (label, value)) in fields.iter().enumerate() {
        if index > 0 {
            snapshot.push(';');
        }
        snapshot.push_str(label);
        snapshot.push('=');
        snapshot.push_str(&value.to_string());
    }
    snapshot
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryVerifiedAssumptionSet {
    binding_digest: WorthQueryEvidenceIdentity,
    asserted_aspects: Vec<WorthQueryAspectTouch>,
    assumption_snapshot_identity: WorthQuerySnapshotIdentity,
    assumption_snapshot_evidence_identity: crate::evidence_identity::WorthQueryEvidenceIdentity,
    assumption_snapshot_digest: WorthQueryEvidenceIdentity,
    verified_precondition_digest: WorthQueryEvidenceIdentity,
    verification_read_set_breadth: WorthQueryVerificationReadSetBreadth,
    verified_assumption_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryVerifiedAssumptionSet {
    pub(in crate::runtime) fn new(
        binding_digest: WorthQueryEvidenceIdentity,
        asserted_aspects: Vec<WorthQueryAspectTouch>,
        assumed_value_fragments: Vec<WorthQueryEvidenceIdentity>,
        cleared_assertion_count: usize,
        snapshot_identity: WorthQuerySnapshotIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let assumption_snapshot_evidence_identity = snapshot_identity.evidence_identity()?;
        let verification_read_set_breadth = WorthQueryVerificationReadSetBreadth::new(
            1,
            asserted_aspects.len(),
            &asserted_aspects,
            cleared_assertion_count,
        );
        let assumption_snapshot_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "existing-truth-assumption-snapshot",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), &binding_digest)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("snapshot"),
                    &assumption_snapshot_evidence_identity,
                )
                .seal()?;
        let verified_precondition_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "existing-truth-verified-precondition",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("assumption_snapshot"),
                    &assumption_snapshot_digest,
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("assumed_value"),
                    assumed_value_fragments.iter(),
                )
                .seal()?;
        let verified_assumption_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "existing-truth-verified-assumption-set",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("binding"), &binding_digest)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("assumption_snapshot"),
                    &assumption_snapshot_digest,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("precondition"),
                    &verified_precondition_digest,
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("asserted_aspect_touch"),
                    asserted_aspects
                        .iter()
                        .map(|touch| touch.admitted_touch_digest_part()),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("target_binding_count"),
                    verification_read_set_breadth.target_binding_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("asserted_aspect_count"),
                    verification_read_set_breadth.asserted_aspect_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("distinct_asserted_aspect_touch_count"),
                    verification_read_set_breadth.distinct_asserted_aspect_touch_count(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("cleared_assertion_count"),
                    verification_read_set_breadth.cleared_assertion_count(),
                )
                .seal()?;
        Ok(Self {
            binding_digest,
            asserted_aspects,
            assumption_snapshot_identity: snapshot_identity,
            assumption_snapshot_evidence_identity,
            assumption_snapshot_digest,
            verified_precondition_digest,
            verification_read_set_breadth,
            verified_assumption_digest,
        })
    }

    pub fn binding_digest(&self) -> &str {
        self.binding_digest.as_str()
    }

    pub fn binding_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_digest
    }

    pub fn asserted_aspects(&self) -> &[WorthQueryAspectTouch] {
        &self.asserted_aspects
    }

    pub fn assumption_snapshot_token(&self) -> &str {
        self.assumption_snapshot_evidence_identity
            .reporting_projection()
    }

    pub fn assumption_snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.assumption_snapshot_identity
    }

    pub fn assumption_snapshot_digest(&self) -> &str {
        self.assumption_snapshot_digest.as_str()
    }

    pub fn assumption_snapshot_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.assumption_snapshot_digest
    }

    pub fn verified_precondition_digest(&self) -> &str {
        self.verified_precondition_digest.as_str()
    }

    pub fn verified_precondition_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.verified_precondition_digest
    }

    pub fn verification_read_set_breadth(&self) -> &WorthQueryVerificationReadSetBreadth {
        &self.verification_read_set_breadth
    }

    pub fn verified_assumption_digest(&self) -> &str {
        self.verified_assumption_digest.as_str()
    }

    pub fn verified_assumption_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.verified_assumption_digest
    }
}
