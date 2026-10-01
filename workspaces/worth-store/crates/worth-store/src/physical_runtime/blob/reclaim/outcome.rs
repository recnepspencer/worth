use std::num::NonZeroU32;

use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{ExtentArenaRange, PersistedRecordIdentity};

use crate::physical_runtime::{
    PhysicalMutationDeadline, PhysicalRetirementDenial, RuntimeIdentity,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReclaimDeferral {
    ProtectedReader,
    PendingPublication,
    CompetingReclaim,
    SourceRootChanged,
    RetirementCapacity,
    ReleaseCertificateCapacity,
    SharedReferences,
    UnconnectedSessionResidue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReclaimPublicationStage {
    Manifest,
    Reservation,
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReclaimDisposition {
    ProvenNoEffect,
    Dropped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReclaimRetirement {
    NotRequired,
    AwaitingRetirement,
    Completed,
    Pending(PhysicalRetirementDenial),
}

/// Bounds one continuation call. The deadline duration starts when that call
/// begins; a native retirement already in progress is not interrupted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReclaimRetirementBudget {
    maximum_work: NonZeroU32,
    deadline: PhysicalMutationDeadline,
}

impl BlobReclaimRetirementBudget {
    pub const fn new(maximum_work: NonZeroU32, deadline: PhysicalMutationDeadline) -> Self {
        Self {
            maximum_work,
            deadline,
        }
    }

    pub const fn maximum_work(self) -> u32 {
        self.maximum_work.get()
    }

    pub const fn deadline(self) -> PhysicalMutationDeadline {
        self.deadline
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobReclaimContinuationFailure {
    ServingRequiresInspection,
    ForeignStoreOrRuntime,
}

/// Reports actual semantic inspection; underlying physical amplification stays
/// in the Store's C5 counters, not an estimate derived from payload length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReclaimObservation {
    pub(super) inspected_records: u64,
    pub(super) inspected_payload_bytes: u64,
    pub(super) admitted_memory_bytes: u64,
}

impl BlobReclaimObservation {
    pub const fn inspected_records(self) -> u64 {
        self.inspected_records
    }
    pub const fn inspected_payload_bytes(self) -> u64 {
        self.inspected_payload_bytes
    }
    pub const fn admitted_memory_bytes(self) -> u64 {
        self.admitted_memory_bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobReclaimDisplacedExtent {
    pub(super) extent: u64,
    pub(super) generation: u64,
    pub(super) range: ExtentArenaRange,
    pub(super) source_root: u64,
    pub(super) bytes: u64,
}

impl BlobReclaimDisplacedExtent {
    pub const fn extent(self) -> u64 {
        self.extent
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
    pub const fn range(self) -> ExtentArenaRange {
        self.range
    }
}

#[derive(Debug)]
pub struct BlobReclaimReceipt {
    pub(super) store: StableStoreIdentity,
    pub(super) runtime: RuntimeIdentity,
    pub(super) disposition: BlobReclaimDisposition,
    pub(super) dropped: Vec<PersistedRecordIdentity>,
    pub(super) displaced: Vec<BlobReclaimDisplacedExtent>,
    pub(super) completed: Vec<bool>,
    pub(super) remaining_payload_records: u64,
    pub(super) bytes_released: u64,
    pub(super) retirement: BlobReclaimRetirement,
    pub(super) observation: BlobReclaimObservation,
}

impl BlobReclaimReceipt {
    pub const fn disposition(&self) -> BlobReclaimDisposition {
        self.disposition
    }
    pub fn dropped_records(&self) -> &[PersistedRecordIdentity] {
        &self.dropped
    }
    pub fn displaced_extents(&self) -> &[BlobReclaimDisplacedExtent] {
        &self.displaced
    }
    pub const fn remaining_payload_records(&self) -> u64 {
        self.remaining_payload_records
    }
    pub const fn bytes_released(&self) -> u64 {
        self.bytes_released
    }
    pub const fn retirement(&self) -> BlobReclaimRetirement {
        self.retirement
    }
    pub const fn observation(&self) -> BlobReclaimObservation {
        self.observation
    }
}
