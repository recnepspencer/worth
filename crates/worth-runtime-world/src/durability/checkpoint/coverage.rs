use crate::durability::WorldReadmissionDenial;
use worth_foundational::facade::WorldDurableJournal;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldJournalCoverage {
    pub(crate) position: u64,
    pub(crate) digest: [u8; 32],
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorldCheckpointCoverage {
    pub(crate) branches: WorldJournalCoverage,
    pub(crate) custody: WorldJournalCoverage,
    pub(crate) occurrences: WorldJournalCoverage,
}
impl WorldCheckpointCoverage {
    pub(crate) fn journal(&self, journal: WorldDurableJournal) -> &WorldJournalCoverage {
        match journal {
            WorldDurableJournal::Branches => &self.branches,
            WorldDurableJournal::Custody => &self.custody,
            WorldDurableJournal::Occurrences => &self.occurrences,
        }
    }
    pub(crate) fn include(
        &mut self,
        journal: WorldDurableJournal,
        position: u64,
        digest: [u8; 32],
    ) -> Result<(), WorldReadmissionDenial> {
        let current = match journal {
            WorldDurableJournal::Branches => &mut self.branches,
            WorldDurableJournal::Custody => &mut self.custody,
            WorldDurableJournal::Occurrences => &mut self.occurrences,
        };
        if current.position.checked_add(1) != Some(position) {
            return Err(WorldReadmissionDenial::JournalSourceMismatch);
        }
        *current = WorldJournalCoverage { position, digest };
        Ok(())
    }
}
