#[path = "observation/allocation_events.rs"]
mod allocation_events;
#[path = "observation/resource_accounting.rs"]
mod resource_accounting;

pub(crate) use allocation_events::{
    PhysicalResidencyActualAllocationUnits, PhysicalResidencyAllocationActualization,
    PhysicalResidencyAllocationEventRecorder, PhysicalResidencyRequestedAllocationUnits,
};
pub use allocation_events::{
    PhysicalResidencyAllocationBoundaryEvent, PhysicalResidencyAllocationBoundaryKind,
    PhysicalResidencyAllocationEventCounters, PhysicalResidencyAllocationEventObserver,
    PhysicalResidencyAllocationEventSnapshot, PhysicalResidencyAllocationOperation,
    PhysicalResidencyAllocationTrace,
};
pub(crate) use resource_accounting::{PhysicalFrameRemoval, PhysicalResidencyAccounting};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalResidencyCounters {
    metadata_bytes: u64,
    peak_metadata_bytes: u64,
    resident_bytes: u64,
    peak_resident_bytes: u64,
    dirty_replacement_bytes: u64,
    peak_dirty_replacement_bytes: u64,
    frame_entries: u32,
    peak_frame_entries: u32,
    pinned_frames: u32,
    peak_pinned_frames: u32,
    pin_leases: u32,
    peak_pin_leases: u32,
    dirty_frames: u32,
    peak_dirty_frames: u32,
    candidate_frames: u32,
    peak_candidate_frames: u32,
    active_loading_frames: u32,
    active_writeback_claims: u32,
    peak_writeback_claims: u32,
    active_operation_bytes: u64,
    peak_operation_bytes: u64,
    peak_admitted_bytes: u64,
    hits: u64,
    faults: u64,
    coalesced_waiters: u64,
    source_loads: u64,
    evictions: u64,
    eviction_candidate_inspections: u64,
    administrative_drains: u64,
    writebacks: u64,
    candidate_publications: u64,
    copied_bytes: u64,
    copy_operations: u64,
    maximum_copy_width: u64,
    dirty_transitions: u64,
    denials: u64,
    identity_transitions: u64,
    speculative_attempts: [u64; 3],
    speculative_admissions: [u64; 3],
    speculative_completions: [u64; 3],
    speculative_denials: [u64; 3],
    active_speculative_frames: [u32; 3],
    peak_speculative_frames: [u32; 3],
    operation_scope_bytes: [u64; 7],
    peak_operation_scope_bytes: [u64; 7],
}

impl PhysicalResidencyCounters {
    /// Accounted frame-table metadata charge: actual retained vector backing,
    /// fixed pool state, and conservative modeled lookup-index storage.
    /// This is not an exact allocator census of the entire pool.
    pub const fn metadata_bytes(self) -> u64 {
        self.metadata_bytes
    }
    pub const fn peak_metadata_bytes(self) -> u64 {
        self.peak_metadata_bytes
    }
    pub const fn resident_bytes(self) -> u64 {
        self.resident_bytes
    }
    pub const fn peak_resident_bytes(self) -> u64 {
        self.peak_resident_bytes
    }
    pub const fn dirty_replacement_bytes(self) -> u64 {
        self.dirty_replacement_bytes
    }
    pub const fn peak_dirty_replacement_bytes(self) -> u64 {
        self.peak_dirty_replacement_bytes
    }
    pub const fn frame_entries(self) -> u32 {
        self.frame_entries
    }
    pub const fn peak_frame_entries(self) -> u32 {
        self.peak_frame_entries
    }
    pub const fn pinned_frames(self) -> u32 {
        self.pinned_frames
    }
    pub const fn peak_pinned_frames(self) -> u32 {
        self.peak_pinned_frames
    }
    pub const fn pin_leases(self) -> u32 {
        self.pin_leases
    }
    pub const fn peak_pin_leases(self) -> u32 {
        self.peak_pin_leases
    }
    pub const fn dirty_frames(self) -> u32 {
        self.dirty_frames
    }
    pub const fn peak_dirty_frames(self) -> u32 {
        self.peak_dirty_frames
    }
    pub const fn candidate_frames(self) -> u32 {
        self.candidate_frames
    }
    pub const fn peak_candidate_frames(self) -> u32 {
        self.peak_candidate_frames
    }
    pub const fn active_loading_frames(self) -> u32 {
        self.active_loading_frames
    }
    pub const fn active_writeback_claims(self) -> u32 {
        self.active_writeback_claims
    }
    pub const fn peak_writeback_claims(self) -> u32 {
        self.peak_writeback_claims
    }
    pub const fn active_operation_bytes(self) -> u64 {
        self.active_operation_bytes
    }
    pub const fn peak_operation_bytes(self) -> u64 {
        self.peak_operation_bytes
    }
    pub const fn admitted_bytes(self) -> u64 {
        self.metadata_bytes
            .saturating_add(self.resident_bytes)
            .saturating_add(self.dirty_replacement_bytes)
            .saturating_add(self.active_operation_bytes)
    }
    pub const fn peak_admitted_bytes(self) -> u64 {
        self.peak_admitted_bytes
    }
    pub const fn hits(self) -> u64 {
        self.hits
    }
    pub const fn faults(self) -> u64 {
        self.faults
    }
    pub const fn coalesced_waiters(self) -> u64 {
        self.coalesced_waiters
    }
    pub const fn source_loads(self) -> u64 {
        self.source_loads
    }
    pub const fn evictions(self) -> u64 {
        self.evictions
    }
    pub const fn eviction_candidate_inspections(self) -> u64 {
        self.eviction_candidate_inspections
    }
    pub const fn administrative_drains(self) -> u64 {
        self.administrative_drains
    }
    pub const fn writebacks(self) -> u64 {
        self.writebacks
    }
    pub const fn candidate_publications(self) -> u64 {
        self.candidate_publications
    }
    pub const fn copied_bytes(self) -> u64 {
        self.copied_bytes
    }
    pub const fn copy_operations(self) -> u64 {
        self.copy_operations
    }
    pub const fn maximum_copy_width(self) -> u64 {
        self.maximum_copy_width
    }
    pub const fn dirty_transitions(self) -> u64 {
        self.dirty_transitions
    }
    pub const fn denials(self) -> u64 {
        self.denials
    }
    pub const fn identity_transitions(self) -> u64 {
        self.identity_transitions
    }
    pub const fn speculative_attempts(self, kind: crate::PhysicalSpeculativeWorkKind) -> u64 {
        self.speculative_attempts[speculative_index(kind)]
    }
    pub const fn speculative_admissions(self, kind: crate::PhysicalSpeculativeWorkKind) -> u64 {
        self.speculative_admissions[speculative_index(kind)]
    }
    pub const fn speculative_completions(self, kind: crate::PhysicalSpeculativeWorkKind) -> u64 {
        self.speculative_completions[speculative_index(kind)]
    }
    pub const fn speculative_denials(self, kind: crate::PhysicalSpeculativeWorkKind) -> u64 {
        self.speculative_denials[speculative_index(kind)]
    }
    pub const fn active_speculative_frames(self, kind: crate::PhysicalSpeculativeWorkKind) -> u32 {
        self.active_speculative_frames[speculative_index(kind)]
    }
    pub const fn peak_speculative_frames(self, kind: crate::PhysicalSpeculativeWorkKind) -> u32 {
        self.peak_speculative_frames[speculative_index(kind)]
    }
    pub const fn active_operation_bytes_for(
        self,
        scope: super::PhysicalOperationAllocationScope,
    ) -> u64 {
        self.operation_scope_bytes[scope.index()]
    }
    pub const fn peak_operation_bytes_for(
        self,
        scope: super::PhysicalOperationAllocationScope,
    ) -> u64 {
        self.peak_operation_scope_bytes[scope.index()]
    }
}

pub(crate) const fn speculative_index(kind: crate::PhysicalSpeculativeWorkKind) -> usize {
    match kind {
        crate::PhysicalSpeculativeWorkKind::ReadAhead => 0,
        crate::PhysicalSpeculativeWorkKind::Prefetch => 1,
        crate::PhysicalSpeculativeWorkKind::WriteBehind => 2,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalResidencyShutdown {
    counters: PhysicalResidencyCounters,
}

impl PhysicalResidencyShutdown {
    pub(crate) const fn new(counters: PhysicalResidencyCounters) -> Self {
        Self { counters }
    }
    pub const fn counters(self) -> PhysicalResidencyCounters {
        self.counters
    }
    pub const fn requires_inspection(self) -> bool {
        self.counters.pin_leases() > self.counters.active_loading_frames()
            || self.counters.dirty_frames() > 0
            || self.counters.active_writeback_claims() > 0
            || self
                .counters
                .active_speculative_frames(crate::PhysicalSpeculativeWorkKind::WriteBehind)
                != 0
            || self.has_cancellable_work_residue()
    }

    pub const fn has_cancellable_work_residue(self) -> bool {
        self.counters.active_operation_bytes() > 0
            || self.counters.active_loading_frames() > 0
            || self
                .counters
                .active_speculative_frames(crate::PhysicalSpeculativeWorkKind::ReadAhead)
                != 0
            || self
                .counters
                .active_speculative_frames(crate::PhysicalSpeculativeWorkKind::Prefetch)
                != 0
    }
}
