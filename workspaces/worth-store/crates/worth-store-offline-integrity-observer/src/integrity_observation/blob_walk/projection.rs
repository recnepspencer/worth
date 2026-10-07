use std::path::Path;

use worth_foundational::{PhysicalArtifactGeneration, PhysicalArtifactIdentity};

use super::super::blob_record::FrameKind;
use super::super::OfflineBlobReclaimSourceKind;
use super::super::{
    BoundedMediaWalk, OfflineArtifactObservation, OfflineIndeterminatePhysicalReason,
    OfflinePhysicalDamageCause as Cause, OfflineUnknownPhysicalReason,
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
        // Arena bytes that no route accounts for hold no selected row, so
        // they excuse no absent one: they only keep a row from being intact.
        if !arena_routes_intact {
            for row in &mut self.selected {
                if row.outcome == Outcome::Intact {
                    row.outcome =
                        Outcome::Unknown(OfflineUnknownPhysicalReason::ParentScopeUnavailable);
                }
            }
        }
        logical_digest::verify_publications(&mut self.selected, root, walk);
        quarantine::validate(&mut self.selected, &self.coverage, root, walk);
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
                let family = row.family();
                let observation = OfflineArtifactObservation::new(
                    row.path,
                    family,
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
        kind: kind_from_prefix(&pending.bytes),
        fact: None,
        outcome: pending
            .interruption
            .or(fallback)
            .unwrap_or_else(|| damage(Cause::Truncation)),
        route: Some(pending.route),
    }
}

/// A frame is of the kind its first bytes declare whether or not its payload
/// could be read.
pub(super) fn kind_from_prefix(bytes: &[u8]) -> Option<FrameKind> {
    bytes.get(8).and_then(|code| FrameKind::declared(*code))
}

#[cfg(test)]
mod tests {
    use worth_foundational::PhysicalArtifactFamily as Family;

    use super::super::ExtentRoute;
    use super::*;
    use crate::integrity_observation::OfflineArtifactFamily;

    fn interrupted(prefix: &[u8]) -> Selected {
        let pending = Pending {
            record: [1; 24],
            logical_bytes: 64,
            path: "arena".into(),
            generation: 1,
            bytes: prefix.to_vec(),
            route: ExtentRoute {
                format: [0; 10],
                arena: 1,
                extent: 1,
                logical_bytes: 64,
                frames: Vec::new(),
            },
            interruption: None,
        };
        incomplete(pending, None)
    }

    #[test]
    fn an_unread_frame_is_reported_under_the_family_of_its_declared_kind() {
        let families = [
            (2, Family::BlobChunkFrame.into()),
            (11, Family::BlobResumeSession.into()),
            (15, Family::BlobResumeSession.into()),
            (14, Family::BlobReclaimDescriptor.into()),
            (16, Family::BlobReclaimDescriptor.into()),
            (0, OfflineArtifactFamily::Unrecognized),
            (17, OfflineArtifactFamily::Unrecognized),
        ];
        for (code, family) in families {
            let mut prefix = b"WRC11BLB".to_vec();
            prefix.push(code);
            let row = interrupted(&prefix);
            assert_eq!(row.family(), family, "kind {code}");
            assert_eq!(row.outcome, damage(Cause::Truncation));
        }
        let before_the_kind_byte = interrupted(b"WRC11BLB");
        assert_eq!(
            before_the_kind_byte.family(),
            OfflineArtifactFamily::Unrecognized
        );
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
