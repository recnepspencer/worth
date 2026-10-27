use std::collections::HashSet;
use std::sync::Arc;

use super::super::authority::{BridgeCausalEvidenceFamily, BridgeCausalEvidenceOwner};
use super::super::counters::BridgeCausalEnvelopeCounters;
use super::super::denial::{BridgeCausalEnvelopeDenial, BridgeCausalEnvelopeDenialKind};
use super::super::evidence_reference::BridgeCausalEvidenceReference;
use super::super::{
    compose_bridge_causal_envelope_evidence_identity,
    digest_basis::BridgeCausalEnvelopeDigestArtifact, evidence_part, shape_part,
};
use crate::identity::BridgeIdentityEvidence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeCausalInspectionAdmissionSummaryKind {
    Admitted,
    Advisory,
}

impl BridgeCausalInspectionAdmissionSummaryKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Advisory => "advisory",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalInspectionAdmissionSummary {
    kind: BridgeCausalInspectionAdmissionSummaryKind,
    query_admission_identity: BridgeIdentityEvidence,
    causal_observation_anchor_identity: BridgeIdentityEvidence,
    summary_identity: BridgeIdentityEvidence,
    expected_query_observation: Option<BridgeIdentityEvidence>,
}

impl BridgeCausalInspectionAdmissionSummary {
    pub fn admitted(
        query_admission_identity: BridgeIdentityEvidence,
        causal_observation_anchor_identity: BridgeIdentityEvidence,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        Self::new(
            BridgeCausalInspectionAdmissionSummaryKind::Admitted,
            query_admission_identity,
            causal_observation_anchor_identity,
        )
    }

    pub fn advisory(
        query_admission_identity: BridgeIdentityEvidence,
        causal_observation_anchor_identity: BridgeIdentityEvidence,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        Self::new(
            BridgeCausalInspectionAdmissionSummaryKind::Advisory,
            query_admission_identity,
            causal_observation_anchor_identity,
        )
    }

    fn new(
        kind: BridgeCausalInspectionAdmissionSummaryKind,
        query_admission_identity: BridgeIdentityEvidence,
        causal_observation_anchor_identity: BridgeIdentityEvidence,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        validate_admission_summary_inputs(
            query_admission_identity.as_str(),
            causal_observation_anchor_identity.as_str(),
        )?;
        let summary_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::AdmissionSummary,
            &[
                shape_part(kind.as_str()),
                evidence_part(&query_admission_identity),
                evidence_part(&causal_observation_anchor_identity),
            ],
        );
        Ok(Self {
            kind,
            query_admission_identity,
            causal_observation_anchor_identity,
            summary_identity,
            expected_query_observation: None,
        })
    }

    pub fn kind(&self) -> BridgeCausalInspectionAdmissionSummaryKind {
        self.kind
    }

    /// Reporting association only; this descriptor grants no Query permission.
    pub fn for_native_observation(
        kind: BridgeCausalInspectionAdmissionSummaryKind,
        query_admission_identity: BridgeIdentityEvidence,
        causal_observation_anchor_identity: BridgeIdentityEvidence,
        expected_query_observation: BridgeIdentityEvidence,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        validate_admission_summary_inputs(
            expected_query_observation.as_str(),
            causal_observation_anchor_identity.as_str(),
        )?;
        let mut summary = Self::new(
            kind,
            query_admission_identity,
            causal_observation_anchor_identity,
        )?;
        summary.summary_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::NativeAdmissionSummary,
            &[
                shape_part(kind.as_str()),
                evidence_part(&summary.query_admission_identity),
                evidence_part(&summary.causal_observation_anchor_identity),
                evidence_part(&expected_query_observation),
            ],
        );
        summary.expected_query_observation = Some(expected_query_observation);
        Ok(summary)
    }

    pub fn expected_query_observation_identity(&self) -> Option<&BridgeIdentityEvidence> {
        self.expected_query_observation.as_ref()
    }

    pub fn query_admission_for_reporting(&self) -> &str {
        self.query_admission_identity.as_str()
    }

    pub fn causal_observation_anchor_for_reporting(&self) -> &str {
        self.causal_observation_anchor_identity.as_str()
    }

    pub fn query_admission_identity(&self) -> &BridgeIdentityEvidence {
        &self.query_admission_identity
    }

    pub fn causal_observation_anchor_identity(&self) -> &BridgeIdentityEvidence {
        &self.causal_observation_anchor_identity
    }

    pub fn summary_for_reporting(&self) -> &str {
        self.summary_identity.as_str()
    }

    pub fn summary_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.summary_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeCausalEnvelopeAssemblyRequest {
    admission_summary: BridgeCausalInspectionAdmissionSummary,
    references: Arc<[BridgeCausalEvidenceReference]>,
    request_identity: BridgeIdentityEvidence,
}

impl BridgeCausalEnvelopeAssemblyRequest {
    pub fn from_query_admission(
        admission_summary: BridgeCausalInspectionAdmissionSummary,
        references: Vec<BridgeCausalEvidenceReference>,
    ) -> Result<Self, BridgeCausalEnvelopeDenial> {
        validate_request_inputs(&references)?;
        if let Some(expected) = admission_summary.expected_query_observation_identity() {
            let actual = references
                .iter()
                .find(|reference| {
                    reference.family() == BridgeCausalEvidenceFamily::QueryObservation
                })
                .unwrap();
            if actual.reference_evidence_identity() != expected {
                return Err(BridgeCausalEnvelopeDenial::new(
                    BridgeCausalEnvelopeDenialKind::QueryObservationAssociationMismatch,
                    actual.family(),
                    actual.owner(),
                    actual.family().expected_owner(),
                    actual.reference_evidence_identity().clone(),
                    BridgeCausalEnvelopeCounters::empty(),
                ));
            }
        }
        let mut request_parts = Vec::with_capacity(references.len() + 1);
        request_parts.push(evidence_part(admission_summary.summary_evidence_identity()));
        request_parts.extend(
            references
                .iter()
                .map(BridgeCausalEvidenceReference::reference_digest_evidence_identity)
                .map(evidence_part),
        );
        let request_identity = compose_bridge_causal_envelope_evidence_identity(
            BridgeCausalEnvelopeDigestArtifact::AssemblyRequest,
            &request_parts,
        );
        Ok(Self {
            admission_summary,
            references: Arc::from(references),
            request_identity,
        })
    }

    pub fn admission_summary(&self) -> &BridgeCausalInspectionAdmissionSummary {
        &self.admission_summary
    }

    pub fn query_admission_for_reporting(&self) -> &str {
        self.admission_summary.query_admission_for_reporting()
    }

    pub fn causal_observation_anchor_for_reporting(&self) -> &str {
        self.admission_summary
            .causal_observation_anchor_for_reporting()
    }

    pub fn references(&self) -> &[BridgeCausalEvidenceReference] {
        &self.references
    }

    pub fn request_for_reporting(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn request_evidence_identity(&self) -> &BridgeIdentityEvidence {
        &self.request_identity
    }
}

fn validate_admission_summary_inputs(
    query_admission_digest: &str,
    causal_observation_anchor_digest: &str,
) -> Result<(), BridgeCausalEnvelopeDenial> {
    if query_admission_digest.is_empty() || causal_observation_anchor_digest.is_empty() {
        return Err(BridgeCausalEnvelopeDenial::new(
            BridgeCausalEnvelopeDenialKind::EmptyAssemblyRequestDigest,
            BridgeCausalEvidenceFamily::QueryObservation,
            BridgeCausalEvidenceOwner::Query,
            BridgeCausalEvidenceOwner::Query,
            denial_sentinel_identity(
                BridgeCausalEnvelopeDenialKind::EmptyAssemblyRequestDigest,
                "empty-query-admission-or-anchor",
            ),
            BridgeCausalEnvelopeCounters::empty(),
        ));
    }
    Ok(())
}

fn validate_request_inputs(
    references: &[BridgeCausalEvidenceReference],
) -> Result<(), BridgeCausalEnvelopeDenial> {
    if references.is_empty() {
        return Err(BridgeCausalEnvelopeDenial::new(
            BridgeCausalEnvelopeDenialKind::MissingEvidenceReference,
            BridgeCausalEvidenceFamily::BridgeRoute,
            BridgeCausalEvidenceOwner::RuntimeBridge,
            BridgeCausalEvidenceOwner::RuntimeBridge,
            denial_sentinel_identity(
                BridgeCausalEnvelopeDenialKind::MissingEvidenceReference,
                "missing-evidence-reference",
            ),
            BridgeCausalEnvelopeCounters::empty(),
        ));
    }
    validate_unique_references(references)?;
    validate_query_observation_anchor(references)?;
    Ok(())
}

fn validate_unique_references(
    references: &[BridgeCausalEvidenceReference],
) -> Result<(), BridgeCausalEnvelopeDenial> {
    let mut seen_references = HashSet::new();
    for reference in references {
        let reference_key = (
            reference.owner(),
            reference.family(),
            reference.reference_evidence_identity(),
        );
        if !seen_references.insert(reference_key) {
            return Err(BridgeCausalEnvelopeDenial::new(
                BridgeCausalEnvelopeDenialKind::DuplicateEvidenceReference,
                reference.family(),
                reference.owner(),
                reference.family().expected_owner(),
                reference.reference_evidence_identity().clone(),
                BridgeCausalEnvelopeCounters::empty(),
            ));
        }
    }
    Ok(())
}

fn validate_query_observation_anchor(
    references: &[BridgeCausalEvidenceReference],
) -> Result<(), BridgeCausalEnvelopeDenial> {
    let query_observation_anchor_count = references
        .iter()
        .filter(|reference| {
            reference.owner() == BridgeCausalEvidenceOwner::Query
                && reference.family() == BridgeCausalEvidenceFamily::QueryObservation
        })
        .count();
    match query_observation_anchor_count {
        1 => Ok(()),
        0 => Err(BridgeCausalEnvelopeDenial::new(
            BridgeCausalEnvelopeDenialKind::MissingQueryObservationAnchor,
            BridgeCausalEvidenceFamily::QueryObservation,
            BridgeCausalEvidenceOwner::Query,
            BridgeCausalEvidenceOwner::Query,
            denial_sentinel_identity(
                BridgeCausalEnvelopeDenialKind::MissingQueryObservationAnchor,
                "missing-query-observation-anchor",
            ),
            BridgeCausalEnvelopeCounters::empty(),
        )),
        count => Err(BridgeCausalEnvelopeDenial::new(
            BridgeCausalEnvelopeDenialKind::QueryObservationAnchorOverclaim,
            BridgeCausalEvidenceFamily::QueryObservation,
            BridgeCausalEvidenceOwner::Query,
            BridgeCausalEvidenceOwner::Query,
            overclaimed_query_observation_anchor_identity(count),
            BridgeCausalEnvelopeCounters::empty(),
        )),
    }
}

fn denial_sentinel_identity(
    kind: BridgeCausalEnvelopeDenialKind,
    label: &'static str,
) -> BridgeIdentityEvidence {
    compose_bridge_causal_envelope_evidence_identity(
        BridgeCausalEnvelopeDigestArtifact::Denial,
        &[shape_part(kind.as_str()), shape_part(label)],
    )
}

fn overclaimed_query_observation_anchor_identity(count: usize) -> BridgeIdentityEvidence {
    let count_text = count.to_string();
    compose_bridge_causal_envelope_evidence_identity(
        BridgeCausalEnvelopeDigestArtifact::Denial,
        &[
            shape_part(BridgeCausalEnvelopeDenialKind::QueryObservationAnchorOverclaim.as_str()),
            shape_part(BridgeCausalEvidenceFamily::QueryObservation.as_str()),
            shape_part("query-observation-anchor-count"),
            shape_part(count_text.as_str()),
        ],
    )
}
