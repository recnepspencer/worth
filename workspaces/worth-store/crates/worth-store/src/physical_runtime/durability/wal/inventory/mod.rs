mod live_segment_inventory;
mod reopen;
mod reopened_member;
mod snapshot;

use worth_store_physical_backend::ArtifactTreeFile;
use worth_store_wal::{WalAppendFrontier, WalArtifactStoreDenial, WalTopologyDenialKind};

pub(in crate::physical_runtime::durability) use live_segment_inventory::PhysicalWalSegmentInventoryEntry;
pub(super) use live_segment_inventory::{
    PhysicalWalSegmentInventory, PhysicalWalSegmentInventoryUpdateDenial,
};
pub(in crate::physical_runtime) use reopen::reopen_wal_inventory;
pub(in crate::physical_runtime) use reopened_member::{
    PhysicalWalBindingReopenCutoff, ReopenedPhysicalWalMember,
};
pub(in crate::physical_runtime::durability) use snapshot::PhysicalWalInventorySnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalWalOpenFailure {
    Media(worth_store_physical_backend::ArtifactTreeFailure),
    InventoryLimitExceeded,
    NonCanonicalArtifact,
    EmptySegment,
    SegmentByteLimitExceeded { admitted: u64, observed: u64 },
    SegmentAllocationRejected,
    SegmentInspection(WalArtifactStoreDenial),
    MemberPayloadRejected,
    CheckpointCutoffOutsideRetainedWal,
    Topology(WalTopologyDenialKind),
    CounterOverflow,
}

pub(in crate::physical_runtime) struct ReopenedPhysicalWalInventory {
    pub(super) frontier: WalAppendFrontier,
    pub(super) active_artifact: ArtifactTreeFile,
    pub(super) segment_count: u32,
    pub(super) frame_count: u64,
    pub(super) byte_count: u64,
    pub(super) peak_buffer_bytes: u64,
    pub(super) requires_inspection: bool,
    pub(super) segments: PhysicalWalSegmentInventory,
    pub(in crate::physical_runtime::durability) members: Vec<ReopenedPhysicalWalMember>,
    pub(in crate::physical_runtime::durability) retirement_spans: Vec<(u64, u64)>,
    pub(in crate::physical_runtime::durability) retirement_records:
        Vec<crate::physical_runtime::durability::RetirementRecord>,
    pub(in crate::physical_runtime::durability) retirement_locations: Vec<(u64, u64)>,
}

impl ReopenedPhysicalWalInventory {
    pub(in crate::physical_runtime) fn take_members(&mut self) -> Vec<ReopenedPhysicalWalMember> {
        std::mem::take(&mut self.members)
    }

    pub(in crate::physical_runtime) fn take_retirement_spans(&mut self) -> Vec<(u64, u64)> {
        std::mem::take(&mut self.retirement_spans)
    }

    pub(in crate::physical_runtime) fn take_retirement_records(
        &mut self,
    ) -> Vec<crate::physical_runtime::durability::RetirementRecord> {
        std::mem::take(&mut self.retirement_records)
    }

    pub(in crate::physical_runtime) fn take_retirement_locations(&mut self) -> Vec<(u64, u64)> {
        std::mem::take(&mut self.retirement_locations)
    }
}
