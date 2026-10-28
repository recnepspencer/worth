use std::num::{NonZeroU32, NonZeroU64};

use serde::{Deserialize, Serialize};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    PhysicalOperationAllocationScope, PhysicalPageSizeClass, PhysicalRecordResidencyPolicy,
    PhysicalSpeculativeWorkKind,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProductionWorldProfile {
    Primary16KiB,
    Pages32KiB,
    Pages64KiB,
    ReusedTails16KiB,
}

impl ProductionWorldProfile {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Primary16KiB => "primary-16kib",
            Self::Pages32KiB => "pages-32kib",
            Self::Pages64KiB => "pages-64kib",
            Self::ReusedTails16KiB => "reused-tails-16kib",
        }
    }

    pub(crate) const fn page_size(self) -> PhysicalPageSizeClass {
        match self {
            Self::Primary16KiB | Self::ReusedTails16KiB => PhysicalPageSizeClass::KiB16,
            Self::Pages32KiB => PhysicalPageSizeClass::KiB32,
            Self::Pages64KiB => PhysicalPageSizeClass::KiB64,
        }
    }

    pub(crate) const fn resident_bytes(self) -> u64 {
        4 * self.page_size().bytes() as u64
    }

    pub(crate) const fn operation_bytes(self) -> u64 {
        match self {
            // The 64 KiB producer's second batch reserves 202 pages (13,238,272
            // bytes) alongside 4,666,496 retained bytes: 17,904,768 total.
            // Round that measured overlap up to the next whole MiB.
            Self::Pages64KiB => 18 * 1024 * 1024,
            Self::Primary16KiB | Self::Pages32KiB | Self::ReusedTails16KiB => 16 * 1024 * 1024,
        }
    }

    pub(crate) const fn batches(self) -> usize {
        match self {
            Self::Primary16KiB | Self::ReusedTails16KiB => 8,
            Self::Pages32KiB | Self::Pages64KiB => 2,
        }
    }

    pub(crate) const fn inline_records_per_batch(self, batch: usize) -> usize {
        match self {
            // Leave the final segment partially occupied for the subsequent
            // poisoned allocation journey, without adding unrelated reuse
            // transitions to every earlier checkpoint workload batch.
            Self::Primary16KiB if batch + 1 == self.batches() => 65,
            Self::Primary16KiB => 64,
            Self::ReusedTails16KiB => 65,
            Self::Pages32KiB | Self::Pages64KiB => 2,
        }
    }

    pub(crate) const fn inline_record_bytes(self) -> usize {
        match self {
            Self::Primary16KiB | Self::ReusedTails16KiB => 3_000,
            // The default placement threshold is inclusive: half-page records
            // become extents. Stay inline while exceeding half the fill budget.
            Self::Pages32KiB | Self::Pages64KiB => self.page_size().bytes() as usize / 2 - 1,
        }
    }

    pub(crate) fn residency(
        self,
        format: AdmittedPhysicalRecordFormat,
    ) -> AdmittedPhysicalRecordResidencyPolicy {
        use PhysicalOperationAllocationScope as Scope;
        use PhysicalSpeculativeWorkKind as Speculation;
        let resident = self.resident_bytes();
        let operation = self.operation_bytes();
        let metadata = 64 * 1024;
        PhysicalRecordResidencyPolicy::builder()
            .total_bytes(bytes(operation + metadata + 2 * resident))
            .resident_bytes(bytes(resident))
            .metadata_bytes(bytes(metadata))
            .frame_entries(count(8))
            .pinned_frames(count(8))
            .pin_leases(count(8))
            .dirty_frames(count(2))
            .dirty_replacement_bytes(bytes(resident))
            .operation_bytes(bytes(operation))
            .scope_bytes(Scope::ForegroundRead, bytes(operation))
            .scope_bytes(Scope::ForegroundWrite, bytes(operation))
            .scope_bytes(Scope::Recovery, bytes(operation))
            .scope_bytes(Scope::Scrub, bytes(4 * 1024 * 1024))
            .scope_bytes(Scope::Maintenance, bytes(operation))
            .scope_bytes(Scope::Verification, bytes(operation))
            .scope_bytes(Scope::Blob, bytes(operation))
            .speculative_frames(Speculation::Prefetch, count(8))
            .speculative_frames(Speculation::ReadAhead, count(8))
            .speculative_frames(Speculation::WriteBehind, count(2))
            .admit(format)
            .into_result()
            .expect("production-world residency policy")
    }
}

fn bytes(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).expect("nonzero byte budget")
}

fn count(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("nonzero count budget")
}
