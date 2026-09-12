use worth_store::physical_runtime::stability::StablePhysicalReadReceipt;
use worth_store_physical_isolation::ChunkMigrationReadInterlockPlan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobPlacementMovementReadHold {
    stable_read: StablePhysicalReadReceipt,
    movement_interlock: ChunkMigrationReadInterlockPlan,
}

impl BlobPlacementMovementReadHold {
    pub const fn from_physical_isolation_stable_read_and_movement_interlock(
        stable_read: StablePhysicalReadReceipt,
        movement_interlock: ChunkMigrationReadInterlockPlan,
    ) -> Self {
        Self {
            stable_read,
            movement_interlock,
        }
    }

    pub const fn stable_read(self) -> StablePhysicalReadReceipt {
        self.stable_read
    }

    pub const fn movement_interlock(self) -> ChunkMigrationReadInterlockPlan {
        self.movement_interlock
    }

    pub const fn guarded_bytes(self) -> u64 {
        self.stable_read.counters().guarded_bytes()
    }
}
