use std::num::NonZeroU64;

use crate::physical_runtime::{
    BlobPhysicalAllocation, PhysicalScopedAllocationAdmission, PhysicalScopedAllocationFailure,
};

const MIB: u64 = 1024 * 1024;
pub(super) const MAX_SOURCE_WINDOW: u64 = 64 * MIB;
pub(super) const INGEST_OVERHEAD: u64 = 5 * MIB;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobResidentComponent {
    SourceWindow,
    PendingChunk,
    EncodedChunk,
    CanonicalRedo,
    Writeback,
    WalFrame,
    TreeNodeA,
    TreeNodeB,
    Publication,
    SchedulerHead,
    Scratch,
    ResumeMetadata,
    ResumeScratch,
}

impl BlobResidentComponent {
    const COUNT: usize = 13;

    const fn index(self) -> usize {
        self as usize
    }

    const fn limit(self, window: u64) -> u64 {
        match self {
            Self::SourceWindow => window,
            Self::PendingChunk => 256 * 1024,
            Self::EncodedChunk | Self::CanonicalRedo | Self::Writeback | Self::WalFrame => MIB,
            Self::TreeNodeA | Self::TreeNodeB | Self::Scratch => 512 * 1024,
            Self::ResumeScratch => 512 * 1024,
            Self::ResumeMetadata => window + INGEST_OVERHEAD,
            Self::Publication | Self::SchedulerHead => 64 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobMemoryDenial {
    EmptyWindow,
    WindowTooLarge,
    WindowCoversObject,
    ComponentTooLarge {
        component: BlobResidentComponent,
        requested: u64,
        maximum: u64,
    },
    CeilingExceeded {
        requested: u64,
        maximum: u64,
    },
    Allocation(PhysicalScopedAllocationFailure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobMemoryObservation {
    admitted_window: u64,
    ceiling: u64,
    peak_charged_bytes: u64,
}

impl BlobMemoryObservation {
    pub const fn admitted_window(self) -> u64 {
        self.admitted_window
    }

    pub const fn ceiling(self) -> u64 {
        self.ceiling
    }

    pub const fn peak_charged_bytes(self) -> u64 {
        self.peak_charged_bytes
    }
}

/// One operation-scoped Store charge and a live-buffer ledger. The ledger
/// checks simultaneous buffers; the process allocator probe independently
/// checks that implementations do not hide a copy outside this accounting.
pub(in crate::physical_runtime) struct BlobIngestAllocation<'runtime> {
    _charge: BlobPhysicalAllocation<'runtime>,
    window: u64,
    ceiling: u64,
    live: [u64; BlobResidentComponent::COUNT],
    peak: u64,
}

impl<'runtime> BlobIngestAllocation<'runtime> {
    pub(super) fn admit(
        admission: &PhysicalScopedAllocationAdmission<'runtime>,
        window: u64,
    ) -> Result<Self, BlobMemoryDenial> {
        if window == 0 {
            return Err(BlobMemoryDenial::EmptyWindow);
        }
        if window > MAX_SOURCE_WINDOW {
            return Err(BlobMemoryDenial::WindowTooLarge);
        }
        let ceiling = window + INGEST_OVERHEAD;
        let charge = admission
            .admit_blob(NonZeroU64::new(ceiling).expect("positive window has positive ceiling"))
            .map_err(BlobMemoryDenial::Allocation)?;
        let mut allocation = Self {
            _charge: charge,
            window,
            ceiling,
            live: [0; BlobResidentComponent::COUNT],
            peak: 0,
        };
        // These bounded tree/publication buffers are shared by fresh ingest
        // and reconstruction, never charged again through a second grant.
        for (component, bytes) in [
            (BlobResidentComponent::TreeNodeA, 512 * 1024),
            (BlobResidentComponent::TreeNodeB, 512 * 1024),
            (BlobResidentComponent::Publication, 64 * 1024),
            (BlobResidentComponent::SchedulerHead, 64 * 1024),
        ] {
            allocation.set_live(component, bytes)?;
        }
        Ok(allocation)
    }

    pub(super) fn set_live(
        &mut self,
        component: BlobResidentComponent,
        bytes: u64,
    ) -> Result<(), BlobMemoryDenial> {
        let maximum = component.limit(self.window);
        if bytes > maximum {
            return Err(BlobMemoryDenial::ComponentTooLarge {
                component,
                requested: bytes,
                maximum,
            });
        }
        let prior = self.live[component.index()];
        let current: u64 = self.live.iter().sum();
        let requested = current - prior + bytes;
        if requested > self.ceiling {
            return Err(BlobMemoryDenial::CeilingExceeded {
                requested,
                maximum: self.ceiling,
            });
        }
        self.live[component.index()] = bytes;
        self.peak = self.peak.max(requested);
        Ok(())
    }

    pub(super) fn observation(&self) -> BlobMemoryObservation {
        BlobMemoryObservation {
            admitted_window: self.window,
            ceiling: self.ceiling,
            peak_charged_bytes: self.peak,
        }
    }

    pub(super) const fn source_window(&self) -> u64 {
        self.window
    }
}
