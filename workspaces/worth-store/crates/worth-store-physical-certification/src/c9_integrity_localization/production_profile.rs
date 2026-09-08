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
}

impl ProductionWorldProfile {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Primary16KiB => "primary-16kib",
            Self::Pages32KiB => "pages-32kib",
            Self::Pages64KiB => "pages-64kib",
        }
    }

    pub(crate) const fn page_size(self) -> PhysicalPageSizeClass {
        match self {
            Self::Primary16KiB => PhysicalPageSizeClass::KiB16,
            Self::Pages32KiB => PhysicalPageSizeClass::KiB32,
            Self::Pages64KiB => PhysicalPageSizeClass::KiB64,
        }
    }

    pub(crate) const fn resident_bytes(self) -> u64 {
        4 * self.page_size().bytes() as u64
    }

    pub(crate) const fn batches(self) -> usize {
        match self {
            Self::Primary16KiB => 8,
            Self::Pages32KiB | Self::Pages64KiB => 2,
        }
    }

    pub(crate) const fn inline_records_per_batch(self) -> usize {
        match self {
            Self::Primary16KiB => 64,
            Self::Pages32KiB | Self::Pages64KiB => 2,
        }
    }

    pub(crate) const fn inline_record_bytes(self) -> usize {
        match self {
            Self::Primary16KiB => 3_000,
            Self::Pages32KiB | Self::Pages64KiB => self.page_size().bytes() as usize / 2,
        }
    }

    pub(crate) fn residency(
        self,
        format: AdmittedPhysicalRecordFormat,
    ) -> AdmittedPhysicalRecordResidencyPolicy {
        use PhysicalOperationAllocationScope as Scope;
        use PhysicalSpeculativeWorkKind as Speculation;
        let resident = self.resident_bytes();
        let operation = 16 * 1024 * 1024;
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
