//! Account resident backing without admitting record payloads or reading Store.
use super::{RelationalRegionResidency, ResidencyState, ResidentRegion};

impl RelationalRegionResidency {
    /// Slot directory cardinalities are eager manifest facts, including while
    /// member payloads are cold or damaged. Counting slots never reads storage.
    pub(crate) fn slot_counts(&self) -> (usize, usize) {
        if let Some(cold) = &self.backing {
            return (cold.entity_slots, cold.relation_slots);
        }
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        match &*state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region) => (
                region.partition.entity_arena.slot_count(),
                region.partition.relation_arena.slot_count(),
            ),
            ResidencyState::Damaged { slot_counts } => *slot_counts,
            _ => unreachable!("a cell without cold backing is born resident"),
        }
    }
    pub(crate) fn resident_inventory(
        &self,
    ) -> crate::storage::overlay::RelationalPartitionAllocationInventory {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &*state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region) => {
                region.partition.allocation_inventory()
            }
            _ => Default::default(),
        }
    }

    pub(crate) fn residency_allocation_bytes(&self) -> u64 {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let resident = matches!(
            &*state,
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

    pub(crate) fn resident_commitment_bytes(&self) -> u64 {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match &*state {
            ResidencyState::Head { region, .. } | ResidencyState::Full(region) => region
                .commitment
                .as_ref()
                .map_or(0, |commitment| commitment.allocation_bytes()),
            _ => 0,
        }
    }

    pub(crate) fn visit_resident_commitment_allocations(
        &self,
        visitor: &mut dyn crate::storage::substrate::StorageAllocationVisitor,
    ) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let ResidencyState::Head { region, .. } | ResidencyState::Full(region) = &*state
            && let Some(commitment) = &region.commitment
        {
            commitment.visit_allocations(visitor);
        }
    }
}
