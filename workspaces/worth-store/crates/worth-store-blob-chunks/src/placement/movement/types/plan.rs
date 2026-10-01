use crate::BlobPlacementClass;

use super::{
    basis::BlobPlacementMovementBasis, cold_outcome::BlobPlacementMovementColdOutcome,
    read_plan_basis::BlobPlacementMovementReadPlanBasis,
};
use crate::placement::movement::counters::BlobPlacementMovementCounterSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBlobPlacementMovementPlan {
    pub(crate) basis: BlobPlacementMovementBasis,
    pub(crate) source_class: BlobPlacementClass,
    pub(crate) target_class: BlobPlacementClass,
    pub(crate) read_plan: BlobPlacementMovementReadPlanBasis,
    pub(crate) cold_outcome: BlobPlacementMovementColdOutcome,
    pub(crate) counters: BlobPlacementMovementCounterSnapshot,
}

impl AdmittedBlobPlacementMovementPlan {
    pub const fn counters(&self) -> BlobPlacementMovementCounterSnapshot {
        self.counters
    }

    pub const fn source_class(&self) -> BlobPlacementClass {
        self.source_class
    }

    pub const fn target_class(&self) -> BlobPlacementClass {
        self.target_class
    }

    pub const fn read_plan(&self) -> BlobPlacementMovementReadPlanBasis {
        self.read_plan
    }

    pub const fn cold_outcome(&self) -> BlobPlacementMovementColdOutcome {
        self.cold_outcome
    }

    pub(crate) const fn basis(&self) -> &BlobPlacementMovementBasis {
        &self.basis
    }
}
