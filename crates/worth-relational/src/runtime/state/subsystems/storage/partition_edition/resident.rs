//! Whole-edition Full custody, issued only after every declared demand succeeds.
use super::{PartitionEdition, PartitionId, PartitionState};

/// The retained edition owns every counted Full lease that backs these reads.
pub(crate) struct ResidentPartitionEdition {
    edition: PartitionEdition,
}

impl ResidentPartitionEdition {
    pub(super) fn new(edition: PartitionEdition) -> Self {
        Self { edition }
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (PartitionId, &PartitionState)> {
        self.edition.entries()
    }

    pub(crate) fn partitions(&self) -> impl Iterator<Item = &PartitionState> {
        self.edition.partitions()
    }
}
