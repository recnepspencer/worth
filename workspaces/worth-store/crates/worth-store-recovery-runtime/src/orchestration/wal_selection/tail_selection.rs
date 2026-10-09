//! Tail partitioning keeps native custody across all input/output overlap.

use super::{
    candidate_storage::ResidentWalCandidates, resident_allocation::WalSelectionAllocation,
    source_selection::ResidentSourceSelection,
};
use worth_store::physical_runtime::{PhysicalRecoveryCoordination, RecoveryWalAllocationDenial};
use worth_store_recovery_physics::{
    admit_physical_wal_tail, select_physical_recovery_sources, PhysicalCheckpointBase,
    PhysicalRecoveryResidue, PhysicalSourceSelectionDenial, SelectedCompactionProduct,
    SelectedPhysicalPageFacts, SelectedPhysicalRoot, SelectedPhysicalWalTail,
    SelectedPhysicalWalTailDenial,
};

#[derive(Debug)]
pub(crate) struct ResidentWalTail {
    tail: SelectedPhysicalWalTail,
    allocation: WalSelectionAllocation,
}

pub(crate) enum WalTailSelectionDenial {
    Allocation(RecoveryWalAllocationDenial),
    Selection(SelectedPhysicalWalTailDenial),
}

impl ResidentWalTail {
    #[cfg(test)]
    pub(crate) fn facts(&self) -> &SelectedPhysicalWalTail {
        &self.tail
    }

    #[cfg(test)]
    pub(crate) fn charged_bytes(&self) -> u64 {
        self.allocation.charged_bytes()
    }

    pub(super) fn from_candidates(
        candidates: ResidentWalCandidates,
        owner: &PhysicalRecoveryCoordination,
        frontier: u64,
        cutoff: Option<u64>,
    ) -> Result<Self, WalTailSelectionDenial> {
        let mut input = candidates;
        let count = input.candidates.len();
        let covered = input
            .allocation
            .prepare_vector(owner, count)
            .map_err(WalTailSelectionDenial::Allocation)?;
        let retained = input
            .allocation
            .prepare_vector(owner, count)
            .map_err(WalTailSelectionDenial::Allocation)?;
        let candidates = std::mem::take(&mut input.candidates);
        let tail = admit_physical_wal_tail(frontier, cutoff, candidates, covered, retained)
            .map_err(WalTailSelectionDenial::Selection)?;
        let bytes = tail
            .owned_heap_bytes()
            .ok_or(WalTailSelectionDenial::Allocation(
                RecoveryWalAllocationDenial::SizeOverflow,
            ))?;
        input
            .allocation
            .settle_after_disposal(bytes)
            .map_err(WalTailSelectionDenial::Allocation)?;
        Ok(Self {
            tail,
            allocation: input.allocation,
        })
    }

    pub(crate) fn select_sources(
        self,
        root: SelectedPhysicalRoot,
        page_facts: SelectedPhysicalPageFacts,
        retained_previous: Option<SelectedPhysicalPageFacts>,
        checkpoint: Option<PhysicalCheckpointBase>,
        compaction: Option<SelectedCompactionProduct>,
        residue: Vec<PhysicalRecoveryResidue>,
    ) -> Result<ResidentSourceSelection, PhysicalSourceSelectionDenial> {
        let Self { tail, allocation } = self;
        let selection = select_physical_recovery_sources(
            root,
            page_facts,
            retained_previous,
            checkpoint,
            tail,
            compaction,
            residue,
        )?;
        Ok(ResidentSourceSelection {
            selection,
            allocation,
        })
    }
}
