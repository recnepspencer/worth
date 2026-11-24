use crate::merge::data::MergePlanningArtifactCore;
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_relational_contracts::internal_port::merge::data::artifacts::correspondence_witness::RelationalMergeCorrespondenceWitnessOwnerPort as _;

use crate::merge::data::{
    IdentityMatchCandidate, IdentityResolutionReason, RelationalMergeCorrespondenceWitness,
};

pub(super) fn retain_merge_correspondence_witness_from_planning_artifact(
    artifact: &MergePlanningArtifactCore,
) -> RelationalMergeCorrespondenceWitness {
    retained_merge_correspondence_witness(
        artifact.request.request_digest().to_string(),
        artifact.branch_basis.basis_digest(),
        &artifact.identity_discovery.candidates,
    )
}

fn retained_merge_correspondence_witness(
    request_digest: String,
    branch_basis_digest: String,
    candidates: &[IdentityMatchCandidate],
) -> RelationalMergeCorrespondenceWitness {
    let schema_counts = schema_declared_correspondence_counts(candidates);
    let rows = candidates
        .iter()
        .map(|candidate| {
            let posture =
                if candidate.reason == IdentityResolutionReason::SchemaDeclaredCorrespondence {
                    let source_count = schema_counts
                        .source_counts
                        .get(&candidate.source_record)
                        .copied()
                        .unwrap_or(0);
                    let target_count = candidate
                        .target_record
                        .as_ref()
                        .and_then(|target| schema_counts.target_counts.get(target).copied())
                        .unwrap_or(0);
                    crate::merge::data::schema_declared_correspondence_posture(
                        source_count,
                        target_count,
                    )
                } else {
                    crate::merge::data::correspondence_posture_for_candidate(candidate)
                };
            crate::merge::data::row_for_candidate(candidate, posture)
        })
        .collect::<Vec<_>>();

    RelationalMergeCorrespondenceWitness::retained(
        request_digest,
        branch_basis_digest,
        Arc::from(rows),
    )
}

struct SchemaDeclaredCorrespondenceCounts {
    source_counts: BTreeMap<crate::transactions::data::primitives::RecordRef, usize>,
    target_counts: BTreeMap<crate::transactions::data::primitives::RecordRef, usize>,
}

fn schema_declared_correspondence_counts(
    candidates: &[IdentityMatchCandidate],
) -> SchemaDeclaredCorrespondenceCounts {
    let mut source_counts = BTreeMap::new();
    let mut target_counts = BTreeMap::new();
    for candidate in candidates.iter().filter(|candidate| {
        candidate.reason == IdentityResolutionReason::SchemaDeclaredCorrespondence
    }) {
        *source_counts
            .entry(candidate.source_record.clone())
            .or_insert(0) += 1;
        if let Some(target_record) = &candidate.target_record {
            *target_counts.entry(target_record.clone()).or_insert(0) += 1;
        }
    }
    SchemaDeclaredCorrespondenceCounts {
        source_counts,
        target_counts,
    }
}
