use super::{
    speculative_index, PhysicalResidencyAllocationActualization,
    PhysicalResidencyAllocationEventRecorder, PhysicalResidencyCounters,
};
use crate::{
    PhysicalOperationAllocationScope, PhysicalResidencyDimension, PhysicalSpeculativeWorkKind,
};

#[path = "resource_accounting/frame.rs"]
mod frame;
pub(crate) use frame::PhysicalFrameRemoval;
#[path = "resource_accounting/lifecycle.rs"]
mod lifecycle;
#[path = "resource_accounting/retirement.rs"]
mod retirement;
#[path = "resource_accounting/speculative.rs"]
mod speculative;

#[derive(Debug)]
pub(crate) struct PhysicalResidencyAccounting {
    counters: PhysicalResidencyCounters,
    events: PhysicalResidencyAllocationEventRecorder,
}

impl PhysicalResidencyAccounting {
    pub(crate) fn new(
        requested_metadata_bytes: u64,
        metadata_bytes: u64,
        events: PhysicalResidencyAllocationEventRecorder,
    ) -> Self {
        events.admit(PhysicalResidencyDimension::MetadataBytes, metadata_bytes);
        events.admit(PhysicalResidencyDimension::TotalBytes, metadata_bytes);
        events.actualize_unscoped(
            PhysicalResidencyDimension::MetadataBytes,
            requested_metadata_bytes,
            metadata_bytes,
        );
        Self {
            counters: PhysicalResidencyCounters {
                metadata_bytes,
                peak_metadata_bytes: metadata_bytes,
                peak_admitted_bytes: metadata_bytes,
                ..PhysicalResidencyCounters::default()
            },
            events,
        }
    }

    pub(crate) const fn snapshot(&self) -> PhysicalResidencyCounters {
        self.counters
    }

    pub(crate) const fn resident_bytes(&self) -> u64 {
        self.counters.resident_bytes
    }

    pub(crate) const fn frame_entries(&self) -> u32 {
        self.counters.frame_entries
    }

    pub(crate) const fn pinned_frames(&self) -> u32 {
        self.counters.pinned_frames
    }

    pub(crate) const fn pin_leases(&self) -> u32 {
        self.counters.pin_leases
    }

    pub(crate) const fn dirty_frames(&self) -> u32 {
        self.counters.dirty_frames
    }

    pub(crate) const fn dirty_replacement_bytes(&self) -> u64 {
        self.counters.dirty_replacement_bytes
    }

    pub(crate) const fn active_operation_bytes(&self) -> u64 {
        self.counters.active_operation_bytes
    }

    pub(crate) const fn operation_scope_bytes(
        &self,
        scope: PhysicalOperationAllocationScope,
    ) -> u64 {
        self.counters.operation_scope_bytes[scope.index()]
    }

    pub(crate) const fn active_speculative_frames(&self, kind: PhysicalSpeculativeWorkKind) -> u32 {
        self.counters.active_speculative_frames[speculative_index(kind)]
    }

    pub(crate) const fn admitted_bytes(&self) -> u64 {
        self.counters.admitted_bytes()
    }

    pub(crate) fn deny(&mut self) {
        self.counters.denials += 1;
    }

    pub(crate) fn deny_dimension(
        &mut self,
        dimension: PhysicalResidencyDimension,
        scope: PhysicalOperationAllocationScope,
        requested: u64,
    ) {
        self.deny();
        self.events.deny_scoped(dimension, scope, requested);
    }

    pub(crate) fn admit_operation(&mut self, scope: PhysicalOperationAllocationScope, bytes: u64) {
        self.counters.active_operation_bytes += bytes;
        self.counters.peak_operation_bytes = self
            .counters
            .peak_operation_bytes
            .max(self.counters.active_operation_bytes);
        self.counters.operation_scope_bytes[scope.index()] += bytes;
        self.counters.peak_operation_scope_bytes[scope.index()] =
            self.counters.peak_operation_scope_bytes[scope.index()]
                .max(self.counters.operation_scope_bytes[scope.index()]);
        self.events
            .admit_scoped(PhysicalResidencyDimension::OperationBytes, scope, bytes);
        self.events.admit_scoped(
            PhysicalResidencyDimension::OperationScope(scope),
            scope,
            bytes,
        );
        self.events
            .admit_scoped(PhysicalResidencyDimension::TotalBytes, scope, bytes);
        self.observe_admitted_peak();
    }

    pub(crate) fn release_operation(
        &mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
    ) {
        self.release_operation_counters(scope, bytes);
        self.events
            .release_scoped(PhysicalResidencyDimension::OperationBytes, scope, bytes);
        self.events.release_scoped(
            PhysicalResidencyDimension::OperationScope(scope),
            scope,
            bytes,
        );
        self.events
            .release_scoped(PhysicalResidencyDimension::TotalBytes, scope, bytes);
    }

    pub(crate) fn reserve_dirty_replacement(
        &mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
    ) {
        self.counters.dirty_replacement_bytes += bytes;
        self.counters.peak_dirty_replacement_bytes = self
            .counters
            .peak_dirty_replacement_bytes
            .max(self.counters.dirty_replacement_bytes);
        self.events.admit_scoped(
            PhysicalResidencyDimension::DirtyReplacementBytes,
            scope,
            bytes,
        );
        self.events
            .admit_scoped(PhysicalResidencyDimension::TotalBytes, scope, bytes);
        self.observe_admitted_peak();
    }

    pub(crate) fn release_dirty_replacement(
        &mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
    ) {
        self.counters.dirty_replacement_bytes -= bytes;
        self.events.release_scoped(
            PhysicalResidencyDimension::DirtyReplacementBytes,
            scope,
            bytes,
        );
        self.events
            .release_scoped(PhysicalResidencyDimension::TotalBytes, scope, bytes);
    }

    pub(crate) fn dirty_replacement_allocator_failed(
        &mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
    ) {
        self.counters.dirty_replacement_bytes -= bytes;
        self.counters.denials += 1;
        self.events.allocator_failure_scoped(
            PhysicalResidencyDimension::DirtyReplacementBytes,
            scope,
            bytes,
        );
        self.events
            .allocator_failure_scoped(PhysicalResidencyDimension::TotalBytes, scope, bytes);
    }

    pub(crate) fn record_source_load(&mut self) {
        self.counters.source_loads += 1;
    }

    pub(crate) fn actualize_allocation(
        &self,
        actualization: PhysicalResidencyAllocationActualization,
    ) {
        self.events.actualize(actualization);
    }

    pub(crate) fn record_copy(&mut self, bytes: u64) {
        self.counters.copy_operations += 1;
        self.counters.copied_bytes = self.counters.copied_bytes.saturating_add(bytes);
        self.counters.maximum_copy_width = self.counters.maximum_copy_width.max(bytes);
    }

    pub(crate) fn inspect_eviction_candidate(&mut self) {
        self.counters.eviction_candidate_inspections += 1;
    }

    pub(crate) fn record_eviction(&mut self) {
        self.counters.evictions += 1;
    }

    pub(crate) fn record_administrative_drain(&mut self) {
        self.counters.administrative_drains += 1;
    }

    pub(crate) fn record_identity_transition(&mut self) {
        self.counters.identity_transitions += 1;
    }

    fn observe_admitted_peak(&mut self) {
        self.counters.peak_admitted_bytes = self
            .counters
            .peak_admitted_bytes
            .max(self.counters.admitted_bytes());
    }
}
