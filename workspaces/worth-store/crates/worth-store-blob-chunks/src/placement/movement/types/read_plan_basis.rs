use worth_store_physical_isolation::{
    ChunkMigrationReadInterlockPlan, PhysicalReadPlanCompletionReceipt,
};

/// Lower-level read-plan bookkeeping for movement eligibility only.
///
/// A completed read plan is not evidence that Store bytes were read and this
/// value cannot execute or publish a physical movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobPlacementMovementReadPlanBasis {
    completion: PhysicalReadPlanCompletionReceipt,
    movement_interlock: ChunkMigrationReadInterlockPlan,
}

impl BlobPlacementMovementReadPlanBasis {
    pub const fn from_completed_plan_and_interlock(
        completion: PhysicalReadPlanCompletionReceipt,
        movement_interlock: ChunkMigrationReadInterlockPlan,
    ) -> Self {
        Self {
            completion,
            movement_interlock,
        }
    }

    pub const fn completion(self) -> PhysicalReadPlanCompletionReceipt {
        self.completion
    }

    pub const fn movement_interlock(self) -> ChunkMigrationReadInterlockPlan {
        self.movement_interlock
    }
}
