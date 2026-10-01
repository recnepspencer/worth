mod copy_publication;
mod live_segment_inventory;
mod publication_roster;
mod reopen;
mod reopened_member;
mod retained_maintenance;
mod snapshot;
pub(super) use copy_publication::observe_bound as observe_bound_copy_publication;
pub(super) use retained_maintenance::RetainedMaintenanceIntent;

use worth_store_physical_backend::ArtifactTreeFile;
use worth_store_wal::{WalAppendFrontier, WalArtifactStoreDenial, WalTopologyDenialKind};

pub(in crate::physical_runtime::durability) use live_segment_inventory::PhysicalWalSegmentInventoryEntry;
pub(super) use live_segment_inventory::{
    PhysicalWalSegmentInventory, PhysicalWalSegmentInventoryUpdateDenial,
};
pub(in crate::physical_runtime) use publication_roster::ReopenedWalPublicationGroup;
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
    ReopenAllocationRejected,
    ReopenAllocationLimitExceeded { admitted: u64, required: u64 },
    SegmentInspection(WalArtifactStoreDenial),
    MemberPayloadRejected,
    IncompletePublicationGroup,
    CheckpointCutoffOutsideRetainedWal,
    Topology(WalTopologyDenialKind),
    CounterOverflow,
}

pub(in crate::physical_runtime) struct ReopenedPhysicalWalInventory {
    pub(super) record_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    pub(super) copy_obligations: Vec<super::RetainedExtentCopyObligation>,
    pub(super) checkpoint_cutoff: u64,
    pub(super) frontier: WalAppendFrontier,
    pub(super) active_artifact: ArtifactTreeFile,
    pub(super) segment_count: u32,
    pub(super) frame_count: u64,
    pub(super) publication_groups: Vec<ReopenedWalPublicationGroup>,
    /// Candidate root, exact charge, and the retained intent's WAL segment/generation.
    pub(super) release_metadata: Vec<(u64, u64, u64, u64)>,
    pub(super) retained_maintenance: Vec<RetainedMaintenanceIntent>,
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
