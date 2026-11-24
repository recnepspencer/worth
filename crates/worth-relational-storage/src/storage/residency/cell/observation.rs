//! Account resident backing without admitting record payloads or reading Store.
use super::{RelationalRegionResidency, ResidencyState, ResidentRegion};

impl RelationalRegionResidency {
    pub fn observe_reclaimable_resident_payload(
        self: &std::sync::Arc<Self>,
        reclaimable: &mut crate::storage::overlay::ReclaimableAllocations,
    ) {
        if let Ok(Some(lease)) = self.lease_checkpoint_contents() {
            lease
                .checkpoint_contents()
                .visit_authoritative_allocation_edges(reclaimable);
        }
    }

    pub(crate) fn reclaimable_unique_resident_payload_bytes(
        self: &std::sync::Arc<Self>,
    ) -> Result<u64, crate::storage::substrate::StorageAllocationOverflow> {
        let mut reclaimable = crate::storage::overlay::ReclaimableAllocations::new();
        self.observe_reclaimable_resident_payload(&mut reclaimable);
        reclaimable.finish()
    }
    /// Slot directory cardinalities are eager manifest facts, including while
    /// member payloads are cold or damaged. Counting slots never reads storage.
    pub(crate) fn slot_counts(&self) -> (usize, usize) {
        if let Some(cold) = &self.backing {
            return (cold.entity_slots, cold.relation_slots);
        }
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match &**state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region) => (
                region.partition.entity_arena.slot_count(),
                region.partition.relation_arena.slot_count(),
            ),
            ResidencyState::Damaged { slot_counts } => *slot_counts,
            _ => unreachable!(
                /* Proof: Cells without cold backing are constructed resident; unload is permitted only when backing exists, so their observation cannot become Cold. */
                "a cell without cold backing is born resident"
            ),
        }
    }
    pub fn resident_inventory(
        &self,
    ) -> Result<
        crate::storage::overlay::RelationalPartitionAllocationInventory,
        crate::storage::substrate::StorageAllocationOverflow,
    > {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &**state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region) => {
                region.partition.allocation_inventory()
            }
            _ => Ok(Default::default()),
        }
    }

    pub fn residency_allocation_bytes(&self) -> u64 {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let resident = matches!(
            &**state,
            ResidencyState::Head { .. } | ResidencyState::Full(_)
        );
        (std::mem::size_of::<Self>()
            + 2 * std::mem::size_of::<usize>()
            + if resident {
                std::mem::size_of::<ResidentRegion>() + 2 * std::mem::size_of::<usize>()
            } else {
                0
            }) as u64
    }

    pub fn resident_commitment_bytes(&self) -> u64 {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &**state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region) => {
                region
                    .commitment
                    .as_ref()
                    .map_or(0, |commitment| commitment.allocation_bytes())
                    + region
                        .kind_indexes
                        .as_ref()
                        .map_or(0, |index| index.allocation_bytes())
            }
            _ => 0,
        }
    }

    pub fn visit_resident_commitment_allocations(
        &self,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let ResidencyState::Head { region, .. } | ResidencyState::Full(region) = &**state
            && let Some(commitment) = &region.commitment
        {
            commitment.visit_allocations(visitor);
            if let Some(indexes) = &region.kind_indexes {
                indexes.visit_allocations(visitor);
            }
        }
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait RelationalRegionResidencyFixturePort: Sized {
    fn reclaimable_unique_resident_payload_bytes_for_test(
        self: &std::sync::Arc<Self>,
    ) -> Result<u64, crate::storage::substrate::StorageAllocationOverflow>;
    fn slot_counts_for_test(&self) -> (usize, usize);
}

impl RelationalRegionResidencyFixturePort for RelationalRegionResidency {
    fn reclaimable_unique_resident_payload_bytes_for_test(
        self: &std::sync::Arc<Self>,
    ) -> Result<u64, crate::storage::substrate::StorageAllocationOverflow> {
        self.reclaimable_unique_resident_payload_bytes()
    }

    fn slot_counts_for_test(&self) -> (usize, usize) {
        self.slot_counts()
    }
}
