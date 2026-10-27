use std::collections::BTreeSet;

use crate::domain_computation::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::anchor::CausalObservationAnchor;
use super::inventory::CausalEvidenceFamily;
use super::observation_identity::CausalEvidenceReferenceDigest;
use super::reference::{
    CausalEvidenceReference, CausalEvidenceReferenceReceipt, CausalEvidenceReferenceResolution,
    CausalEvidenceReferenceResolutionCounters, CausalEvidenceReferenceResolutionDenial,
    CausalEvidenceReferenceSet,
};
use super::reference_index::{
    causal_evidence_reference_index, causal_evidence_reference_index_record, owner_for_family,
    CausalEvidenceReferenceIndex,
};

pub fn resolve_causal_evidence_references(
    anchor: CausalObservationAnchor,
    requested_families: &[CausalEvidenceFamily],
) -> CausalEvidenceReferenceResolution {
    let index = anchor_derived_reference_index(&anchor);
    resolve_indexed_causal_evidence_references(anchor, requested_families, &index)
}

pub fn resolve_indexed_causal_evidence_references(
    anchor: CausalObservationAnchor,
    requested_families: &[CausalEvidenceFamily],
    index: &CausalEvidenceReferenceIndex,
) -> CausalEvidenceReferenceResolution {
    let requested_families = requested_reference_families(&anchor, requested_families);
    let anchor_reference_width = anchor.observation_receipt().evidence_identities().len();
    let mut resolved_references = Vec::new();
    let mut missing_families = Vec::new();
    let mut missing_indexed_reference_count = 0;
    let mut index_lookup_count = 0;

    for family in requested_families.iter().copied() {
        let anchor_reference_digests = anchor_reference_digests_for_family(&anchor, family);
        if anchor_reference_digests.is_empty() {
            index_lookup_count += 1;
            missing_families.push(family);
            continue;
        }

        for anchor_reference_digest in anchor_reference_digests {
            index_lookup_count += 1;
            match index.record_for_reference(family, anchor_reference_digest) {
                Some(index_record) => {
                    resolved_references.push(CausalEvidenceReference::new(
                        index_record.owner(),
                        family,
                        index_record.reference_digest().clone(),
                    ));
                }
                None => missing_indexed_reference_count += 1,
            }
        }
    }

    let counters = CausalEvidenceReferenceResolutionCounters::new(
        requested_families.len(),
        anchor_reference_width,
        resolved_references.len() + missing_indexed_reference_count,
        index_lookup_count,
        resolved_references.len(),
        missing_families.len() + missing_indexed_reference_count,
    );

    if !missing_families.is_empty() || missing_indexed_reference_count > 0 {
        return CausalEvidenceReferenceResolution::MissingRequiredEvidence {
            denial: CausalEvidenceReferenceResolutionDenial::new(
                anchor.anchor_digest().clone(),
                missing_families,
                missing_indexed_reference_count,
            ),
            counters,
        };
    }

    let reference_set_digest = reference_set_digest(&anchor, &resolved_references);
    let receipt = CausalEvidenceReferenceReceipt::new(
        anchor.anchor_digest().clone(),
        reference_set_digest.clone(),
        resolved_references.len(),
        0,
    );
    CausalEvidenceReferenceResolution::Resolved {
        reference_set: CausalEvidenceReferenceSet::new(
            anchor,
            resolved_references,
            reference_set_digest,
            receipt,
        ),
        counters,
    }
}

fn requested_reference_families(
    anchor: &CausalObservationAnchor,
    requested_families: &[CausalEvidenceFamily],
) -> Vec<CausalEvidenceFamily> {
    if requested_families.is_empty() {
        return anchor
            .observation_receipt()
            .evidence_identities()
            .iter()
            .map(|identity| identity.family())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
    }
    requested_families
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn reference_set_digest(
    anchor: &CausalObservationAnchor,
    references: &[CausalEvidenceReference],
) -> CausalEvidenceReferenceDigest {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::CausalEvidenceReference)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("anchor"),
            anchor.anchor_digest().evidence_identity(),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("references"),
            references
                .iter()
                .map(CausalEvidenceReference::evidence_identity),
        )
        .seal()
        .into()
}

fn anchor_derived_reference_index(
    anchor: &CausalObservationAnchor,
) -> CausalEvidenceReferenceIndex {
    causal_evidence_reference_index(
        anchor
            .observation_receipt()
            .evidence_identities()
            .iter()
            .map(|identity| {
                causal_evidence_reference_index_record(
                    owner_for_family(identity.family()),
                    identity.family(),
                    identity.reference_digest().clone(),
                )
                .expect("Phase 1 anchor validation rejects empty evidence identities")
            }),
    )
}

fn anchor_reference_digests_for_family(
    anchor: &CausalObservationAnchor,
    family: CausalEvidenceFamily,
) -> Vec<&CausalEvidenceReferenceDigest> {
    anchor
        .observation_receipt()
        .evidence_identities()
        .iter()
        .filter(|identity| identity.family() == family)
        .map(|identity| identity.reference_digest())
        .collect()
}
