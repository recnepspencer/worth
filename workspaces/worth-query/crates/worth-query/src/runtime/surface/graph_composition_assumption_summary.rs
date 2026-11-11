use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{WorthQueryVerificationReadSetBreadth, WorthQueryWriteReceipt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionAssumptionSummary {
    assumption_snapshot_digests: Vec<WorthQueryEvidenceIdentity>,
    verified_precondition_digests: Vec<WorthQueryEvidenceIdentity>,
    verified_step_count: usize,
    verification_read_set_breadth: WorthQueryVerificationReadSetBreadth,
    counter_snapshot: String,
    aggregate_assumption_snapshot_digest: WorthQueryEvidenceIdentity,
    aggregate_verified_precondition_digest: WorthQueryEvidenceIdentity,
    assumption_summary_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryGraphCompositionAssumptionSummary {
    pub(in crate::runtime) fn derive(
        write_receipts: &[WorthQueryWriteReceipt],
    ) -> Result<Option<Self>, IdentityDerivationDenial> {
        let assumption_sets = write_receipts
            .iter()
            .filter_map(|receipt| {
                receipt
                    .existing_truth_assertion_evidence()
                    .and_then(|evidence| evidence.verified_assumption_set())
            })
            .collect::<Vec<_>>();
        if assumption_sets.is_empty() {
            return Ok(None);
        }

        let assumption_snapshot_digests = assumption_sets
            .iter()
            .map(|set| set.assumption_snapshot_evidence_digest().clone())
            .collect::<Vec<_>>();
        let verified_precondition_digests = assumption_sets
            .iter()
            .map(|set| set.verified_precondition_evidence_digest().clone())
            .collect::<Vec<_>>();

        let target_binding_count = assumption_sets
            .iter()
            .map(|set| set.verification_read_set_breadth().target_binding_count())
            .sum();
        let asserted_aspect_count = assumption_sets
            .iter()
            .map(|set| set.verification_read_set_breadth().asserted_aspect_count())
            .sum();
        let asserted_aspects = assumption_sets
            .iter()
            .flat_map(|set| set.asserted_aspects().iter().cloned())
            .collect::<Vec<_>>();
        let cleared_assertion_count = assumption_sets
            .iter()
            .map(|set| {
                set.verification_read_set_breadth()
                    .cleared_assertion_count()
            })
            .sum();
        let verification_read_set_breadth = WorthQueryVerificationReadSetBreadth::new(
            target_binding_count,
            asserted_aspect_count,
            &asserted_aspects,
            cleared_assertion_count,
        );
        let verified_step_count = assumption_sets.len();
        let counter_snapshot = diagnostic_counter_snapshot_with_tail(
            &[("verified_steps", verified_step_count)],
            verification_read_set_breadth.counter_snapshot(),
        );
        let aggregate_assumption_snapshot_digest = aggregate_digest(
            "worth_query_graph_composition_assumption_snapshot_digest_v1",
            &assumption_snapshot_digests,
        )?;
        let aggregate_verified_precondition_digest = aggregate_digest(
            "worth_query_graph_composition_verified_precondition_digest_v1",
            &verified_precondition_digests,
        )?;
        let assumption_summary_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-assumption-summary",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("assumption_snapshots"),
                    &aggregate_assumption_snapshot_digest,
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("verified_preconditions"),
                    &aggregate_verified_precondition_digest,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("verified_step_count"),
                    verified_step_count,
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
                    WorthQueryEvidenceTag::new("cleared_assertion_count"),
                    verification_read_set_breadth.cleared_assertion_count(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("asserted_aspect_touch"),
                    asserted_aspects
                        .iter()
                        .map(|touch| touch.admitted_touch_digest_part()),
                )
                .seal()?;

        Ok(Some(Self {
            assumption_snapshot_digests,
            verified_precondition_digests,
            verified_step_count,
            verification_read_set_breadth,
            counter_snapshot,
            aggregate_assumption_snapshot_digest,
            aggregate_verified_precondition_digest,
            assumption_summary_digest,
        }))
    }

    pub fn assumption_snapshot_digests(&self) -> Vec<&str> {
        self.assumption_snapshot_digests
            .iter()
            .map(WorthQueryEvidenceIdentity::as_str)
            .collect()
    }

    pub fn assumption_snapshot_evidence_digests(&self) -> &[WorthQueryEvidenceIdentity] {
        &self.assumption_snapshot_digests
    }

    pub fn verified_precondition_digests(&self) -> Vec<&str> {
        self.verified_precondition_digests
            .iter()
            .map(WorthQueryEvidenceIdentity::as_str)
            .collect()
    }

    pub fn verified_precondition_evidence_digests(&self) -> &[WorthQueryEvidenceIdentity] {
        &self.verified_precondition_digests
    }

    pub fn verified_step_count(&self) -> usize {
        self.verified_step_count
    }

    pub fn verification_read_set_breadth(&self) -> &WorthQueryVerificationReadSetBreadth {
        &self.verification_read_set_breadth
    }

    pub fn counter_snapshot(&self) -> &str {
        &self.counter_snapshot
    }

    pub fn aggregate_assumption_snapshot_digest(&self) -> &str {
        self.aggregate_assumption_snapshot_digest.as_str()
    }

    pub fn aggregate_verified_precondition_digest(&self) -> &str {
        self.aggregate_verified_precondition_digest.as_str()
    }

    pub fn assumption_summary_digest(&self) -> &str {
        self.assumption_summary_digest.as_str()
    }

    pub fn assumption_summary_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.assumption_summary_digest
    }
}

fn diagnostic_counter_snapshot_with_tail(fields: &[(&str, usize)], tail: &str) -> String {
    let mut snapshot = diagnostic_counter_snapshot(fields);
    if !snapshot.is_empty() && !tail.is_empty() {
        snapshot.push(';');
    }
    snapshot.push_str(tail);
    snapshot
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

fn aggregate_digest(
    label: &str,
    digests: &[WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
        .field_shape(WorthQueryEvidenceTag::new("role"), label)
        .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("digest"), digests.iter())
        .seal()
}
