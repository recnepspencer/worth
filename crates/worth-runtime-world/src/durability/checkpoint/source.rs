//! The trusted Home adapter retains one real protected World read cut.
use super::{WorldCheckpointSelection, WorldCheckpointSubject};
use crate::durability::WorldDurableMediumDenial;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::WorldDurableJournal;

/// Opaque owner bytes and fixed stream bounds from the same protected root.
/// These are untrusted descriptions until World admits their entire meaning.
pub struct WorldCheckpointReadView {
    pub(super) selected: Option<(ExecutionImmutableBytes, std::ops::Range<usize>)>,
    heads: [u64; 3],
}
impl WorldCheckpointReadView {
    /// The restricted Home adapter supplies its selected World section and
    /// Branches/Custody/Occurrences head positions from ONE actual reader.
    pub fn from_protected_home_source(
        selected: Option<(ExecutionImmutableBytes, std::ops::Range<usize>)>,
        heads: [u64; 3],
    ) -> Result<Self, WorldDurableMediumDenial> {
        if let Some((bytes, region)) = &selected {
            bytes
                .get(region.clone())
                .ok_or(crate::durability::WorldDurableFrameDenial::Malformed)?;
        }
        Ok(Self { selected, heads })
    }
    pub(super) fn head(&self, journal: WorldDurableJournal) -> u64 {
        self.heads[match journal {
            WorldDurableJournal::Branches => 0,
            WorldDurableJournal::Custody => 1,
            WorldDurableJournal::Occurrences => 2,
        }]
    }
}
/// Original journal bytes retain their actual request-owned allocation.
pub struct WorldCheckpointJournalRecord {
    pub(super) position: u64,
    pub(super) digest: [u8; 32],
    pub(super) bytes: ExecutionImmutableBytes,
}
impl WorldCheckpointJournalRecord {
    pub fn from_protected_world_journal(
        position: u64,
        digest: [u8; 32],
        bytes: ExecutionImmutableBytes,
    ) -> Self {
        Self {
            position,
            digest,
            bytes,
        }
    }
}
/// Only the actual attached Home supplies this source. The implementation
/// retains its protected physical reader and original selected membership.
/// No method may select a newer root, infer missing state, or replace the
/// original request. Selection consults that SAME Home's pending attempt.
pub trait WorldCheckpointReadSource: std::fmt::Debug + Send + Sync {
    fn read(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorldCheckpointReadView, WorldDurableMediumDenial>;
    fn read_journal_record(
        &self,
        request: &ExecutionRequest<'_, '_>,
        journal: WorldDurableJournal,
        position: u64,
    ) -> Result<WorldCheckpointJournalRecord, WorldDurableMediumDenial>;
    fn observe_selection(
        &self,
        subject: &WorldCheckpointSubject,
    ) -> Result<Option<WorldCheckpointSelection>, WorldDurableMediumDenial>;
}
