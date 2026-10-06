//! Native Recovery backing for the two root slots' read and canonical-check windows.
//! Projections and diagnostic vectors leave this temporary-buffer boundary unfunded.

use worth_store::physical_runtime::{
    BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact, PhysicalRecoveryReadAllocation,
    PhysicalRecoveryRejoinResidentAdmissionDenial, PhysicalRecoveryRejoinResidentDenial,
    RecoveryDiscoveryAllocationFailure, RecoveryDiscoveryFailure,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, RecordArtifactFile, RootSelectorRole, ROOT_SELECTOR_BYTES,
};

use crate::entry::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDimension as Dimension,
    PhysicalRecoveryRootProtocolArtifact as Artifact, PhysicalRecoverySourceDenial,
    PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};
use crate::orchestration::discovery::source_memory::source_allocation;
use crate::orchestration::discovery::{refused_read, DiscoveryFailure};
use crate::orchestration::reader_limit::{OversizedArtifact, ReadCeiling};

/// The inner `Err` is the artifact found larger than its own ceiling.
pub(super) type RootRead =
    Result<Result<ObservedRecoveryArtifact, OversizedArtifact>, DiscoveryFailure>;

pub(super) struct FundedRootReads<'window, 'owner> {
    allocation: &'window mut PhysicalRecoveryReadAllocation<'owner>,
    limits: PhysicalRecoveryLimitDeclaration,
    retained: u64,
    scratch: u64,
}

impl<'window, 'owner> FundedRootReads<'window, 'owner> {
    pub(super) fn new(
        allocation: &'window mut PhysicalRecoveryReadAllocation<'owner>,
        limits: PhysicalRecoveryLimitDeclaration,
    ) -> Self {
        Self {
            allocation,
            limits,
            retained: 0,
            scratch: 0,
        }
    }

    pub(super) fn read_selector(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        role: RootSelectorRole,
    ) -> RootRead {
        let address = match role {
            RootSelectorRole::Current => RecordArtifactFile::CurrentRootSelector,
            RootSelectorRole::Previous => RecordArtifactFile::PreviousRootSelector,
        };
        let artifact = super::selector_artifact(role);
        let ceiling = ReadCeiling::of_artifact(ROOT_SELECTOR_BYTES as u64);
        let limits = self.limits;
        match discovery.read_record_artifact_with_allocator(
            address,
            ceiling.requested(),
            |length| self.allocate(length),
        ) {
            Ok(observed) => Ok(Ok(observed)),
            Err(failure) => refused(&limits, artifact, failure, |failure| {
                refused_read(failure, ceiling, &limits, Dimension::ObservationBytes)
            })
            .map(Err),
        }
    }

    pub(super) fn read_root(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        role: RootSelectorRole,
        generation: u64,
        ceiling: ReadCeiling,
    ) -> RootRead {
        let artifact = super::root_artifact(role, generation);
        let limits = self.limits;
        match discovery.read_record_artifact_with_allocator(
            RecordArtifactFile::RootManifest { generation },
            ceiling.requested(),
            |length| self.allocate(length),
        ) {
            Ok(observed) => Ok(Ok(observed)),
            Err(failure) => refused(&limits, artifact, failure, |failure| {
                refused_read(failure, ceiling, &limits, Dimension::ManifestBytes)
            })
            .map(Err),
        }
    }

    fn allocate(&mut self, length: usize) -> Result<Vec<u8>, Cause> {
        let requested = length as u64;
        let retained = self.retained.saturating_add(requested);
        let total = retained.saturating_add(self.scratch);
        self.allocation
            .reserve_total(total)
            .map_err(Cause::Residency)?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(|cause| {
            Cause::Residency(PhysicalRecoveryRejoinResidentDenial::Allocation { requested, cause })
        })?;
        if bytes.capacity() != length {
            return Err(Cause::AllocatorExceededReservation {
                requested,
                actual: bytes.capacity() as u64,
            });
        }
        bytes.resize(length, 0);
        self.retained = retained;
        Ok(bytes)
    }

    /// Called only after the slot consumer has dropped selector/root buffers
    /// and returned inline projections. The native window retains its high-water.
    pub(super) fn finish_slot(&mut self) {
        self.retained = 0;
        self.scratch = 0;
    }

    pub(super) fn reserve_canonical_scratch(
        &mut self,
        artifact: Artifact,
    ) -> Result<(), DiscoveryFailure> {
        // Root admission re-encodes a canonical frame. Selector encode returns
        // a fixed array and needs no heap grant. Inline root fields retain no heap.
        let scratch = DurablePhysicalRootManifest::maximum_encoding_scratch_bytes() as u64;
        let total = self.retained.saturating_add(scratch.max(self.scratch));
        let limits = self.limits;
        self.allocation.reserve_total(total).map_err(|cause| {
            allocation_failure(
                &limits,
                artifact,
                Boundary::CanonicalValidation,
                scratch,
                Cause::Residency(cause),
            )
        })?;
        self.scratch = self.scratch.max(scratch);
        Ok(())
    }
}

fn refused(
    limits: &PhysicalRecoveryLimitDeclaration,
    artifact: Artifact,
    failure: RecoveryDiscoveryAllocationFailure<Cause>,
    refused_read: impl FnOnce(RecoveryDiscoveryFailure) -> Result<OversizedArtifact, DiscoveryFailure>,
) -> Result<OversizedArtifact, DiscoveryFailure> {
    match failure {
        RecoveryDiscoveryAllocationFailure::Discovery(failure) => refused_read(failure),
        RecoveryDiscoveryAllocationFailure::Allocation {
            requested, cause, ..
        } => Err(allocation_failure(
            limits,
            artifact,
            Boundary::ReadBuffer,
            requested as u64,
            cause,
        )),
        RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            requested,
            observed,
            ..
        } => Err(allocation_failure(
            limits,
            artifact,
            Boundary::ReadBuffer,
            requested as u64,
            Cause::ReadBufferLengthMismatch {
                requested,
                observed,
            },
        )),
    }
}

pub(in super::super) fn window_admission_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    cause: PhysicalRecoveryRejoinResidentAdmissionDenial,
) -> DiscoveryFailure {
    allocation_failure(
        limits,
        Artifact::CurrentSelector,
        Boundary::WindowAdmission,
        0,
        Cause::Admission(cause),
    )
}

fn allocation_failure(
    limits: &PhysicalRecoveryLimitDeclaration,
    artifact: Artifact,
    boundary: Boundary,
    requested: u64,
    cause: Cause,
) -> DiscoveryFailure {
    source_allocation(limits, cause, |cause| {
        PhysicalRecoverySourceDenial::SourceReadAllocation {
            artifact,
            boundary,
            requested,
            cause,
        }
    })
}
