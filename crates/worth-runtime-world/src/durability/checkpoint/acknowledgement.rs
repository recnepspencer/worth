//! Coverage advances only for the SAME prepared subject's actual Home selection.
use super::{WorldCheckpointCoverage, WorldCheckpointSelection, WorldCheckpointSubject};
use crate::durability::WorldDurableMediumDenial as Denial;

#[derive(Debug, Default)]
pub(in crate::durability) struct WorldCheckpointAcknowledgement {
    pub(in crate::durability) coverage: WorldCheckpointCoverage,
    selected: Option<(u64, [u8; 32])>,
}
impl WorldCheckpointAcknowledgement {
    pub(in crate::durability) fn admit_recovered(
        &mut self,
        coverage: WorldCheckpointCoverage,
    ) -> Result<(), Denial> {
        if self.selected.is_some() || self.coverage != WorldCheckpointCoverage::default() {
            return Err(Denial::StaleCheckpoint);
        }
        self.coverage = coverage;
        Ok(())
    }
    pub(super) fn validate(&self, candidate: &WorldCheckpointCoverage) -> Result<(), Denial> {
        validate_successor(&self.coverage, candidate)
    }
}
pub(super) fn validate_successor(
    before: &WorldCheckpointCoverage,
    candidate: &WorldCheckpointCoverage,
) -> Result<(), Denial> {
    use worth_foundational::facade::WorldDurableJournal::*;
    for journal in [Branches, Custody, Occurrences] {
        let before = before.journal(journal);
        let after = candidate.journal(journal);
        if after.position < before.position
            || (after.position == before.position && after.digest != before.digest)
        {
            return Err(Denial::StaleCheckpoint);
        }
    }
    Ok(())
}
impl WorldCheckpointAcknowledgement {
    pub(super) fn acknowledge(
        &mut self,
        subject: &WorldCheckpointSubject,
        selected: &WorldCheckpointSelection,
    ) -> Result<(), Denial> {
        if !selected.admits(subject) {
            return Err(Denial::CheckpointSelectionRequired);
        }
        if let Some((generation, digest)) = self.selected
            && (selected.generation() < generation
                || (selected.generation() == generation && selected.manifest_sha256() != digest))
        {
            return Err(Denial::StaleCheckpoint);
        }
        self.validate(subject.coverage())?;
        self.coverage = subject.coverage().clone();
        self.selected = Some((selected.generation(), selected.manifest_sha256()));
        Ok(())
    }
}
