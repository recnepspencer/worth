use serde::{Deserialize, Serialize};

use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

/// A slot in the node arena that may be occupied or vacant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Slot {
    /// Generation counter (bumped on each reuse).
    pub(crate) generation: u32,
    /// Whether this slot has been permanently retired after generation wrap.
    pub(crate) retired: bool,
    /// Whether this slot currently owns a live node lane payload.
    pub(crate) occupied: bool,
}

impl RetainedStorageMeasurement for Slot {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            generation: _,
            retired: _,
            occupied: _,
        } = self;
        Ok(Charge::ZERO)
    }
}

impl Slot {
    /// Create a new vacant slot at generation 0.
    pub(crate) fn vacant() -> Self {
        Self {
            generation: 0,
            retired: false,
            occupied: false,
        }
    }

    /// Create a permanently retired vacant slot to reserve allocator space.
    pub(crate) fn retired_placeholder() -> Self {
        Self {
            generation: 0,
            retired: true,
            occupied: false,
        }
    }

    /// Occupy this slot, returning the current generation.
    pub(crate) fn occupy(&mut self) -> u32 {
        self.occupied = true;
        self.generation
    }

    /// Vacate this slot, bumping the generation and retiring on wrap.
    pub(crate) fn vacate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.retired = true;
        }
        self.occupied = false;
    }

    /// Whether this slot is occupied.
    pub(crate) fn is_occupied(&self) -> bool {
        self.occupied
    }

    pub(crate) fn is_retired(&self) -> bool {
        self.retired
    }
}
