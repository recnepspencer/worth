use worth_store_physical_format::{
    PersistedRecordIdentity, ReleasedDropPredecessorV1, ReleasedGenerationReclaimBasisV1,
};

use super::{valid_drop_set, PhysicalReclaimAttempt};
use crate::physical_runtime::{durability::DisplacedArtifact, PhysicalRecordReader};

/// A different semantic source shares the same single Store root/drop fence.
/// The proof is consumed before this capability exists; encoded basis alone
/// cannot construct it or authorize another publication.
pub(in crate::physical_runtime) struct AdmittedReleasedGenerationDrop {
    _reader: PhysicalRecordReader,
    attempt: PhysicalReclaimAttempt,
    basis: ReleasedGenerationReclaimBasisV1,
    dropped: Vec<PersistedRecordIdentity>,
    displaced: Vec<DisplacedArtifact>,
    remaining: u64,
    predecessor: Option<ReleasedDropPredecessorV1>,
    cumulative_dropped: u64,
    terminal: bool,
}

impl AdmittedReleasedGenerationDrop {
    pub(in crate::physical_runtime) fn protected_reader(&self) -> &PhysicalRecordReader {
        &self._reader
    }
    #[allow(clippy::too_many_arguments)]
    pub(in crate::physical_runtime) fn new(
        reader: PhysicalRecordReader,
        attempt: PhysicalReclaimAttempt,
        basis: ReleasedGenerationReclaimBasisV1,
        dropped: Vec<PersistedRecordIdentity>,
        displaced: Vec<DisplacedArtifact>,
        remaining: u64,
        predecessor: Option<ReleasedDropPredecessorV1>,
        cumulative_dropped: u64,
        terminal: bool,
    ) -> Option<Self> {
        if !valid_drop_set(&dropped, &displaced)
            || (predecessor.is_none()
                && dropped.binary_search(&basis.publication_record()).is_err())
            || (predecessor.is_some() && dropped.contains(&basis.publication_record()))
            || cumulative_dropped < dropped.len() as u64
            || (predecessor.is_none() && cumulative_dropped != dropped.len() as u64)
        {
            return None;
        }
        Some(Self {
            _reader: reader,
            attempt,
            basis,
            dropped,
            displaced,
            remaining,
            predecessor,
            cumulative_dropped,
            terminal,
        })
    }

    pub(in crate::physical_runtime) fn attempt(&self) -> &PhysicalReclaimAttempt {
        &self.attempt
    }
    pub(in crate::physical_runtime) const fn basis(&self) -> ReleasedGenerationReclaimBasisV1 {
        self.basis
    }
    pub(in crate::physical_runtime) fn dropped(&self) -> &[PersistedRecordIdentity] {
        &self.dropped
    }
    pub(in crate::physical_runtime) fn displaced(&self) -> &[DisplacedArtifact] {
        &self.displaced
    }
    pub(in crate::physical_runtime) const fn remaining(&self) -> u64 {
        self.remaining
    }
    pub(in crate::physical_runtime) const fn predecessor(
        &self,
    ) -> Option<ReleasedDropPredecessorV1> {
        self.predecessor
    }
    pub(in crate::physical_runtime) const fn cumulative_dropped(&self) -> u64 {
        self.cumulative_dropped
    }
    pub(in crate::physical_runtime) const fn terminal(&self) -> bool {
        self.terminal
    }
    pub(in crate::physical_runtime) fn complete(self) -> bool {
        self.attempt.complete()
    }
}
