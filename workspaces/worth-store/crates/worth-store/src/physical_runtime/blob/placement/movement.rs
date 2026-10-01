use worth_store_physical_format::PersistedRecordIdentity;

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, BlobReadSession, IndeterminatePhysicalMutation,
    PhysicalExtentCopyPhase, PhysicalExtentCopyProgress, PhysicalExtentCopyResolutionProgress,
    PhysicalMutationOutcome, PhysicalMutationPreparationOutcome, PhysicalMutationRequest,
    ProvenNoEffectPhysicalMutation, RecordAppendDenial, RecordAppendError, ServingPhysicalRuntime,
};

/// Only a Store-selected and authenticated direct chunk can enter SourceCopy.
/// The read retains its exact protected C.5 root through publication.
pub struct BlobMovementReadHold<'runtime> {
    read: BlobReadSession<'runtime>,
    record: PersistedRecordIdentity,
    bytes: u64,
    runtime: crate::physical_runtime::RuntimeIdentity,
}

impl<'runtime> BlobMovementReadHold<'runtime> {
    pub(in crate::physical_runtime::blob) fn from_selected(
        read: BlobReadSession<'runtime>,
        record: PersistedRecordIdentity,
        bytes: u64,
        runtime: crate::physical_runtime::RuntimeIdentity,
    ) -> Self {
        Self {
            read,
            record,
            bytes,
            runtime,
        }
    }

    pub(in crate::physical_runtime) const fn selected_record(&self) -> PersistedRecordIdentity {
        self.record
    }

    pub(in crate::physical_runtime) fn selected_root_generation(&self) -> u64 {
        self.read.reader.protected_root().root().generation().get()
    }

    pub fn protected_root(&self) -> crate::physical_runtime::PhysicalProtectedRootObservation {
        self.read.reader.protected_root()
    }
}

pub struct BlobMovementSession<'runtime> {
    runtime: &'runtime ServingPhysicalRuntime,
    hold: BlobMovementReadHold<'runtime>,
    progress: PhysicalExtentCopyProgress,
    attempted: bool,
}

pub struct BlobMovementReceipt {
    record: PersistedRecordIdentity,
    logical_chunk_bytes: u64,
    root_generation: u64,
}

impl BlobMovementReceipt {
    pub const fn logical_chunk_bytes(&self) -> u64 {
        self.logical_chunk_bytes
    }
    pub const fn root_generation(&self) -> u64 {
        self.root_generation
    }
    pub const fn record(&self) -> PersistedRecordIdentity {
        self.record
    }
}

pub enum BlobMovementFailure {
    ForeignStore,
    ForeignRuntime,
    Preparation(PhysicalMutationPreparationOutcome),
    Copy(RecordAppendError),
    CopyAdmission(RecordAppendDenial),
    Incomplete,
    AlreadyAttempted,
    ProvenNoEffect(ProvenNoEffectPhysicalMutation),
    Indeterminate(IndeterminatePhysicalMutation),
}

impl std::fmt::Debug for BlobMovementFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            Self::ForeignStore => "ForeignStore",
            Self::ForeignRuntime => "ForeignRuntime",
            Self::Preparation(_) => "Preparation",
            Self::Copy(_) => "Copy",
            Self::CopyAdmission(_) => "CopyAdmission",
            Self::Incomplete => "Incomplete",
            Self::AlreadyAttempted => "AlreadyAttempted",
            Self::ProvenNoEffect(_) => "ProvenNoEffect",
            Self::Indeterminate(_) => "Indeterminate",
        };
        f.write_str(kind)
    }
}

impl<'runtime> BlobMovementSession<'runtime> {
    pub(in crate::physical_runtime::blob) fn begin(
        runtime: &'runtime ServingPhysicalRuntime,
        hold: BlobMovementReadHold<'runtime>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> Result<Self, BlobMovementFailure> {
        Self::begin_inner(runtime, hold, placement, request, None)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::blob) fn begin_in_tier(
        runtime: &'runtime ServingPhysicalRuntime,
        hold: BlobMovementReadHold<'runtime>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        target_tier: worth_store_physical_format::PhysicalTierClass,
    ) -> Result<Self, BlobMovementFailure> {
        Self::begin_inner(runtime, hold, placement, request, Some(target_tier))
    }

    fn begin_inner(
        runtime: &'runtime ServingPhysicalRuntime,
        hold: BlobMovementReadHold<'runtime>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
        target_tier: Option<worth_store_physical_format::PhysicalTierClass>,
    ) -> Result<Self, BlobMovementFailure> {
        if hold.read.reader.store_identity() != runtime.store_identity() {
            return Err(BlobMovementFailure::ForeignStore);
        }
        if hold.runtime != runtime.runtime_identity() {
            return Err(BlobMovementFailure::ForeignRuntime);
        }
        let submission = runtime.record_submission();
        let copy = match target_tier {
            Some(tier) => {
                submission.begin_selected_blob_chunk_copy_in_tier(&hold, placement, request, tier)
            }
            None => submission.begin_selected_blob_chunk_copy(&hold, placement, request),
        };
        let progress = copy
            .map_err(BlobMovementFailure::Copy)?
            .map_err(BlobMovementFailure::Preparation)?;
        Ok(Self {
            runtime,
            hold,
            progress,
            attempted: false,
        })
    }

    pub const fn progress(&self) -> PhysicalExtentCopyProgress {
        self.progress
    }

    pub fn advance(&mut self) -> Result<PhysicalExtentCopyProgress, BlobMovementFailure> {
        if self.attempted {
            return Err(BlobMovementFailure::AlreadyAttempted);
        }
        self.progress = self
            .runtime
            .record_submission()
            .advance_extent_copy()
            .map_err(BlobMovementFailure::Copy)?;
        Ok(self.progress)
    }

    pub fn publish(&mut self) -> Result<BlobMovementReceipt, BlobMovementFailure> {
        if self.attempted {
            return Err(BlobMovementFailure::AlreadyAttempted);
        }
        if self.progress.phase != PhysicalExtentCopyPhase::ReadyForAdoption {
            return Err(BlobMovementFailure::Incomplete);
        }
        let prepared = self
            .runtime
            .record_submission()
            .prepare_completed_extent_copy()
            .map_err(BlobMovementFailure::CopyAdmission)?;
        self.attempted = true;
        match prepared.execute() {
            PhysicalMutationOutcome::Completed(completed) => Ok(BlobMovementReceipt {
                record: self.hold.record,
                logical_chunk_bytes: self.hold.bytes,
                root_generation: completed.completed_breadth().current_root_generation(),
            }),
            PhysicalMutationOutcome::ProvenNoEffect(value) => {
                Err(BlobMovementFailure::ProvenNoEffect(value))
            }
            PhysicalMutationOutcome::Indeterminate(value) => {
                Err(BlobMovementFailure::Indeterminate(value))
            }
        }
    }

    pub fn cancel(self) -> Result<PhysicalExtentCopyResolutionProgress, BlobMovementFailure> {
        self.runtime
            .record_submission()
            .cancel_extent_copy()
            .map_err(BlobMovementFailure::Copy)
    }
}
