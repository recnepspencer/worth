use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PhysicalExtentCopyIntent, PhysicalExtentCopyRecord, PhysicalExtentCopyResolution,
};
#[cfg(test)]
mod tests;

/// An intent observed in a verified WAL frame, or settled by the live WAL barrier.
/// Its original interval survives resolution until a real checkpoint covers it.
#[derive(Debug, Clone, Copy)]
pub(in crate::physical_runtime) struct RetainedExtentCopyObligation {
    intent: PhysicalExtentCopyIntent,
    digest: [u8; 32],
    segment: u64,
    generation: u64,
    start: u64,
    end: u64,
    resolution: Option<(PhysicalExtentCopyResolution, u64)>,
    pub(super) publication: Option<(u64, u64)>,
}

impl RetainedExtentCopyObligation {
    pub(in crate::physical_runtime) fn intent(self) -> PhysicalExtentCopyIntent {
        self.intent
    }
    pub(in crate::physical_runtime) fn intent_digest(self) -> [u8; 32] {
        self.digest
    }
    pub(in crate::physical_runtime) fn intent_lsn(self) -> u64 {
        self.start
    }
    #[cfg(test)]
    pub(in crate::physical_runtime) fn wal_interval(self) -> (u64, u64, u64, u64) {
        (self.segment, self.generation, self.start, self.end)
    }
    pub(in crate::physical_runtime) fn resolution(
        self,
    ) -> Option<(PhysicalExtentCopyResolution, u64)> {
        self.resolution
    }
    pub(in crate::physical_runtime) fn publication(self) -> Option<(u64, u64)> {
        self.publication
    }
    pub(super) fn holds_at(self, cutoff: u64) -> bool {
        self.resolution.is_none_or(|(_, end)| end > cutoff)
    }
    pub(super) fn overlaps(self, start: u64, end: u64) -> bool {
        start < self.end && self.start < end
    }
}

pub(super) fn observe_copy_record(
    obligations: &mut Vec<RetainedExtentCopyObligation>,
    record: PhysicalExtentCopyRecord,
    segment: u64,
    generation: u64,
    start: u64,
    end: u64,
    checkpoint_cutoff: u64,
) -> Result<(), ()> {
    match record {
        PhysicalExtentCopyRecord::Intent(intent) => {
            // A stable operation owns precisely one original durable intent.
            if obligations
                .iter()
                .any(|entry| entry.intent.operation() == intent.operation())
            {
                return Err(());
            }
            obligations.push(RetainedExtentCopyObligation {
                intent,
                digest: Sha256::digest(record.encode()).into(),
                segment,
                generation,
                start,
                end,
                resolution: None,
                publication: None,
            });
        }
        PhysicalExtentCopyRecord::Resolved(resolution) => {
            let Some(entry) = obligations
                .iter_mut()
                .find(|entry| entry.intent.operation() == resolution.operation())
            else {
                // A checkpointed resolution may outlive its reclaimed intent segment.
                return if end <= checkpoint_cutoff {
                    Ok(())
                } else {
                    Err(())
                };
            };
            if entry.start != resolution.intent_lsn()
                || entry.digest != resolution.intent_digest()
                || start < entry.end
                || entry.resolution.is_some()
            {
                return Err(());
            }
            match resolution.kind() {
                worth_store_physical_format::PhysicalExtentCopyResolutionKind::Cancelled
                    if entry.publication.is_some() =>
                {
                    return Err(())
                }
                worth_store_physical_format::PhysicalExtentCopyResolutionKind::Published {
                    root_generation,
                    publication_lsn,
                } if entry.publication != Some((root_generation, publication_lsn)) => {
                    return Err(())
                }
                _ => {}
            }
            entry.resolution = Some((resolution, end));
        }
    }
    Ok(())
}

/// The caller holds completed reclamation admitted through the checkpoint gate.
/// Keep unresolved entries even if a caller presents an inconsistent identity.
pub(super) fn prune_reclaimed(
    obligations: &mut Vec<RetainedExtentCopyObligation>,
    segment: u64,
    generation: u64,
) {
    obligations.retain(|entry| {
        entry.segment != segment || entry.generation != generation || entry.resolution.is_none()
    });
}
