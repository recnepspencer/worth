use std::path::Path;

use worth_foundational::{
    PhysicalArtifactFamily as Family, PhysicalArtifactGeneration, PhysicalArtifactIdentity,
};

use super::super::OfflineBlobReclaimSourceKind;
use super::super::{
    BoundedMediaWalk, OfflineArtifactFamily, OfflineArtifactObservation,
    OfflineIndeterminatePhysicalReason, OfflinePhysicalDamageCause as Cause,
    OfflineUnknownPhysicalReason,
};
use super::{damage, logical_digest, quarantine, BlobRecordWalk, Outcome, Pending, Selected};

impl BlobRecordWalk {
    pub(crate) fn finish(
        mut self,
        root: &Path,
        walk: &mut BoundedMediaWalk,
        arena_routes_intact: bool,
    ) -> Vec<OfflineArtifactObservation> {
        if let Some(pending) = self.pending.take() {
            self.selected.push(incomplete_at_finish(
                pending,
                walk.exhausted_reason(),
                arena_routes_intact,
            ));
        }
        self.validate_selected_claim_graph();
        if !arena_routes_intact {
            for row in &mut self.selected {
                if row.outcome == Outcome::Intact {
                    row.outcome =
                        Outcome::Unknown(OfflineUnknownPhysicalReason::ParentScopeUnavailable);
                }
            }
        }
        logical_digest::verify_publications(&mut self.selected, root, walk);
        quarantine::validate(&mut self.selected, root, walk);
        self.selected
            .into_iter()
            .map(|row| {
                let identity = format!("blob-record:{}", hex_record(&row.record));
                let source_kind = match row.fact.as_ref() {
                    Some(
                        super::BlobFact::DropSetManifest { .. }
                        | super::BlobFact::ReclaimDescriptor { .. },
                    ) => Some(OfflineBlobReclaimSourceKind::FailedIngest),
                    Some(
                        super::BlobFact::ReleasedDropSetManifest { .. }
                        | super::BlobFact::ReleasedReclaimDescriptor { .. },
                    ) => Some(OfflineBlobReclaimSourceKind::ReleasedGeneration),
                    _ => None,
                };
                let observation = OfflineArtifactObservation::new(
                    row.path,
                    row.family,
                    PhysicalArtifactIdentity::new(identity).expect("short identity"),
                    PhysicalArtifactGeneration::encoded(row.generation)
                        .unwrap_or(PhysicalArtifactGeneration::NotEncoded),
                    // C.5 extent chunks interleave logical payload bytes on disk.
                    None,
                    row.outcome,
                );
                if let Some(kind) = source_kind {
                    observation.with_blob_reclaim_source_kind(kind)
                } else {
                    observation
                }
            })
            .collect()
    }
}

pub(super) fn incomplete_at_finish(
    pending: Pending,
    exhausted: Option<OfflineIndeterminatePhysicalReason>,
    arena_routes_intact: bool,
) -> Selected {
    let fallback = exhausted.map(Outcome::Indeterminate).or_else(|| {
        (!arena_routes_intact).then_some(Outcome::Unknown(
            OfflineUnknownPhysicalReason::ParentScopeUnavailable,
        ))
    });
    incomplete(pending, fallback)
}

pub(super) fn incomplete(pending: Pending, fallback: Option<Outcome>) -> Selected {
    Selected {
        record: pending.record,
        path: pending.path,
        generation: pending.generation,
        family: family_from_prefix(&pending.bytes),
        fact: None,
        outcome: pending
            .interruption
            .or(fallback)
            .unwrap_or_else(|| damage(Cause::Truncation)),
        route: Some(pending.route),
    }
}

pub(super) fn family_from_prefix(bytes: &[u8]) -> OfflineArtifactFamily {
    match bytes.get(8) {
        Some(1 | 5 | 6 | 11) => Family::BlobResumeSession.into(),
        Some(12) => OfflineArtifactFamily::DedupeQuarantine,
        Some(2) => Family::BlobChunkFrame.into(),
        Some(3) => Family::BlobTreeNode.into(),
        Some(4) => Family::BlobGenerationPublication.into(),
        Some(7) => Family::BlobDropSetManifest.into(),
        Some(9) => Family::BlobDropSetManifest.into(),
        Some(13) => Family::BlobDropSetManifest.into(),
        Some(8) => Family::BlobReclaimDescriptor.into(),
        Some(14) => Family::BlobReclaimDescriptor.into(),
        Some(10) => OfflineArtifactFamily::OriginalDropReservation,
        _ => OfflineArtifactFamily::Unrecognized,
    }
}

pub(super) fn hex_record(record: &[u8; 24]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(48);
    for byte in record {
        value.push(HEX[(byte >> 4) as usize] as char);
        value.push(HEX[(byte & 15) as usize] as char);
    }
    value
}
