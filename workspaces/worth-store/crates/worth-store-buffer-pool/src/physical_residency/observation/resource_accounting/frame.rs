use crate::{PhysicalOperationAllocationScope, PhysicalResidencyDimension};

use super::PhysicalResidencyAccounting;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PhysicalFrameRemoval {
    scope: PhysicalOperationAllocationScope,
    bytes: u64,
    pins: u32,
    dirty: bool,
    candidate: bool,
}

impl PhysicalFrameRemoval {
    pub(crate) const fn new(
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
        pins: u32,
    ) -> Self {
        Self {
            scope,
            bytes,
            pins,
            dirty: false,
            candidate: false,
        }
    }

    pub(crate) const fn with_dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    pub(crate) const fn with_candidate(mut self, candidate: bool) -> Self {
        self.candidate = candidate;
        self
    }
}

impl PhysicalResidencyAccounting {
    pub(crate) fn admit_frame(
        &mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
        dirty: bool,
        candidate: bool,
    ) {
        self.counters.resident_bytes += bytes;
        self.counters.frame_entries += 1;
        self.counters.pinned_frames += 1;
        self.counters.pin_leases += 1;
        self.counters.active_loading_frames += 1;
        if dirty {
            self.counters.dirty_frames += 1;
        } else {
            self.counters.faults += 1;
        }
        if candidate {
            self.counters.candidate_frames += 1;
        }
        self.events
            .admit_scoped(PhysicalResidencyDimension::ResidentBytes, scope, bytes);
        self.events
            .admit_scoped(PhysicalResidencyDimension::TotalBytes, scope, bytes);
        self.events
            .admit_scoped(PhysicalResidencyDimension::FrameEntries, scope, 1);
        self.events
            .admit_scoped(PhysicalResidencyDimension::PinnedFrames, scope, 1);
        self.events
            .admit_scoped(PhysicalResidencyDimension::PinLeases, scope, 1);
        if dirty {
            self.events
                .admit_scoped(PhysicalResidencyDimension::DirtyFrames, scope, 1);
        }
        self.observe_frame_peaks();
        self.observe_admitted_peak();
    }

    pub(crate) fn finish_loading(&mut self) {
        self.counters.active_loading_frames -= 1;
    }

    pub(crate) fn resolve_bounded_frame(
        &mut self,
        scope: PhysicalOperationAllocationScope,
        reserved: u64,
        actual: u64,
    ) {
        let released = reserved
            .checked_sub(actual)
            .expect("a bounded frame cannot exceed its admitted reservation");
        self.counters.resident_bytes -= released;
        self.events
            .release_scoped(PhysicalResidencyDimension::ResidentBytes, scope, released);
        self.events
            .release_scoped(PhysicalResidencyDimension::TotalBytes, scope, released);
    }

    pub(crate) fn attach_loading_waiter(&mut self) {
        self.counters.pin_leases += 1;
        self.counters.coalesced_waiters += 1;
        self.counters.peak_pin_leases = self.counters.peak_pin_leases.max(self.counters.pin_leases);
        self.events.admit(PhysicalResidencyDimension::PinLeases, 1);
    }

    pub(crate) fn fail_loading(
        &mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
        pins: u32,
        retain_identity: bool,
    ) {
        self.counters.resident_bytes -= bytes;
        self.counters.pin_leases -= pins;
        self.counters.pinned_frames -= 1;
        self.counters.active_loading_frames -= 1;
        self.events
            .release_scoped(PhysicalResidencyDimension::ResidentBytes, scope, bytes);
        self.events
            .release_scoped(PhysicalResidencyDimension::TotalBytes, scope, bytes);
        self.events.release_scoped(
            PhysicalResidencyDimension::PinLeases,
            scope,
            u64::from(pins),
        );
        self.events
            .release_scoped(PhysicalResidencyDimension::PinnedFrames, scope, 1);
        if !retain_identity {
            self.release_failed_loading_identity(scope);
        }
    }

    pub(crate) fn release_failed_loading_identity(
        &mut self,
        scope: PhysicalOperationAllocationScope,
    ) {
        self.counters.frame_entries -= 1;
        self.events
            .release_scoped(PhysicalResidencyDimension::FrameEntries, scope, 1);
    }

    pub(crate) fn remove_frame(&mut self, removal: PhysicalFrameRemoval) {
        self.counters.resident_bytes -= removal.bytes;
        self.counters.frame_entries -= 1;
        self.counters.pin_leases -= removal.pins;
        if removal.pins > 0 {
            self.counters.pinned_frames -= 1;
        }
        if removal.dirty {
            self.counters.dirty_frames -= 1;
        }
        if removal.candidate {
            self.counters.candidate_frames -= 1;
        }
        self.events.release_scoped(
            PhysicalResidencyDimension::ResidentBytes,
            removal.scope,
            removal.bytes,
        );
        self.events.release_scoped(
            PhysicalResidencyDimension::TotalBytes,
            removal.scope,
            removal.bytes,
        );
        self.events
            .release_scoped(PhysicalResidencyDimension::FrameEntries, removal.scope, 1);
        self.events.release_scoped(
            PhysicalResidencyDimension::PinLeases,
            removal.scope,
            u64::from(removal.pins),
        );
        if removal.pins > 0 {
            self.events
                .release_scoped(PhysicalResidencyDimension::PinnedFrames, removal.scope, 1);
        }
        if removal.dirty {
            self.events
                .release_scoped(PhysicalResidencyDimension::DirtyFrames, removal.scope, 1);
        }
    }

    pub(crate) fn candidate_allocator_failed(
        &mut self,
        scope: PhysicalOperationAllocationScope,
        bytes: u64,
        pins: u32,
    ) {
        self.counters.resident_bytes -= bytes;
        self.counters.frame_entries -= 1;
        self.counters.pin_leases -= pins;
        self.counters.pinned_frames -= 1;
        self.counters.dirty_frames -= 1;
        self.counters.candidate_frames -= 1;
        self.counters.denials += 1;
        self.events.allocator_failure_scoped(
            PhysicalResidencyDimension::ResidentBytes,
            scope,
            bytes,
        );
        self.events
            .allocator_failure_scoped(PhysicalResidencyDimension::TotalBytes, scope, bytes);
        self.events
            .release_scoped(PhysicalResidencyDimension::FrameEntries, scope, 1);
        self.events.release_scoped(
            PhysicalResidencyDimension::PinLeases,
            scope,
            u64::from(pins),
        );
        self.events
            .release_scoped(PhysicalResidencyDimension::PinnedFrames, scope, 1);
        self.events
            .release_scoped(PhysicalResidencyDimension::DirtyFrames, scope, 1);
    }

    pub(crate) fn pin(&mut self, newly_pinned: bool) {
        self.counters.pin_leases += 1;
        self.events.admit(PhysicalResidencyDimension::PinLeases, 1);
        if newly_pinned {
            self.counters.pinned_frames += 1;
            self.events
                .admit(PhysicalResidencyDimension::PinnedFrames, 1);
        }
        self.counters.peak_pinned_frames = self
            .counters
            .peak_pinned_frames
            .max(self.counters.pinned_frames);
        self.counters.peak_pin_leases = self.counters.peak_pin_leases.max(self.counters.pin_leases);
        self.counters.hits += 1;
    }

    pub(crate) fn unpin(&mut self, became_unpinned: bool) {
        self.release_pin_counters(became_unpinned);
        self.events
            .release(PhysicalResidencyDimension::PinLeases, 1);
        if became_unpinned {
            self.events
                .release(PhysicalResidencyDimension::PinnedFrames, 1);
        }
    }

    pub(crate) fn mark_dirty(&mut self, candidate_added: bool) {
        self.counters.dirty_frames += 1;
        self.counters.dirty_transitions += 1;
        self.events
            .admit(PhysicalResidencyDimension::DirtyFrames, 1);
        if candidate_added {
            self.counters.candidate_frames += 1;
        }
        self.counters.peak_dirty_frames = self
            .counters
            .peak_dirty_frames
            .max(self.counters.dirty_frames);
        self.counters.peak_candidate_frames = self
            .counters
            .peak_candidate_frames
            .max(self.counters.candidate_frames);
    }

    pub(crate) fn mark_clean(&mut self, candidate_removed: bool, writeback: bool) {
        self.counters.dirty_frames -= 1;
        self.events
            .release(PhysicalResidencyDimension::DirtyFrames, 1);
        if candidate_removed {
            self.counters.candidate_frames -= 1;
            self.counters.candidate_publications += 1;
        }
        if writeback {
            self.counters.writebacks += 1;
        }
    }

    fn observe_frame_peaks(&mut self) {
        self.counters.peak_resident_bytes = self
            .counters
            .peak_resident_bytes
            .max(self.counters.resident_bytes);
        self.counters.peak_frame_entries = self
            .counters
            .peak_frame_entries
            .max(self.counters.frame_entries);
        self.counters.peak_pinned_frames = self
            .counters
            .peak_pinned_frames
            .max(self.counters.pinned_frames);
        self.counters.peak_pin_leases = self.counters.peak_pin_leases.max(self.counters.pin_leases);
        self.counters.peak_dirty_frames = self
            .counters
            .peak_dirty_frames
            .max(self.counters.dirty_frames);
        self.counters.peak_candidate_frames = self
            .counters
            .peak_candidate_frames
            .max(self.counters.candidate_frames);
    }
}
