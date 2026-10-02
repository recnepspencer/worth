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
    PhysicalRecoveryBlockKind, PhysicalRecoveryRootProtocolArtifact as Artifact,
    PhysicalRecoverySourceDenial, PhysicalRecoverySourceReadAllocationBoundary as Boundary,
    PhysicalRecoverySourceReadAllocationDenial as Cause,
};
use crate::orchestration::discovery::DiscoveryFailure;

pub(super) struct FundedRootReads<'window, 'owner> {
    allocation: &'window mut PhysicalRecoveryReadAllocation<'owner>,
    retained: u64,
    scratch: u64,
}

impl<'window, 'owner> FundedRootReads<'window, 'owner> {
    pub(super) fn new(allocation: &'window mut PhysicalRecoveryReadAllocation<'owner>) -> Self {
        Self {
            allocation,
            retained: 0,
            scratch: 0,
        }
    }

    pub(super) fn read_selector(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        role: RootSelectorRole,
    ) -> Result<ObservedRecoveryArtifact, DiscoveryFailure> {
        let address = match role {
            RootSelectorRole::Current => RecordArtifactFile::CurrentRootSelector,
            RootSelectorRole::Previous => RecordArtifactFile::PreviousRootSelector,
        };
        let artifact = super::selector_artifact(role);
        discovery
            .read_record_artifact_with_allocator(address, ROOT_SELECTOR_BYTES as u64, |length| {
                self.allocate(length)
            })
            .map_err(|failure| {
                map_read_failure(
                    artifact,
                    failure,
                    super::super::map_selector_discovery_failure,
                )
            })
    }

    pub(super) fn read_root(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        role: RootSelectorRole,
        generation: u64,
        maximum: u64,
        map_discovery: impl FnOnce(RecoveryDiscoveryFailure) -> DiscoveryFailure,
    ) -> Result<ObservedRecoveryArtifact, DiscoveryFailure> {
        let artifact = super::root_artifact(role, generation);
        discovery
            .read_record_artifact_with_allocator(
                RecordArtifactFile::RootManifest { generation },
                maximum,
                |length| self.allocate(length),
            )
            .map_err(|failure| map_read_failure(artifact, failure, map_discovery))
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
        self.allocation.reserve_total(total).map_err(|cause| {
            allocation_failure(
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

fn map_read_failure(
    artifact: Artifact,
    failure: RecoveryDiscoveryAllocationFailure<Cause>,
    map_discovery: impl FnOnce(RecoveryDiscoveryFailure) -> DiscoveryFailure,
) -> DiscoveryFailure {
    match failure {
        RecoveryDiscoveryAllocationFailure::Discovery(failure) => map_discovery(failure),
        RecoveryDiscoveryAllocationFailure::Allocation {
            requested, cause, ..
        } => allocation_failure(artifact, Boundary::ReadBuffer, requested as u64, cause),
        RecoveryDiscoveryAllocationFailure::BufferLengthMismatch {
            requested,
            observed,
            ..
        } => allocation_failure(
            artifact,
            Boundary::ReadBuffer,
            requested as u64,
            Cause::ReadBufferLengthMismatch {
                requested,
                observed,
            },
        ),
    }
}

pub(in super::super) fn window_admission_failure(
    cause: PhysicalRecoveryRejoinResidentAdmissionDenial,
) -> DiscoveryFailure {
    allocation_failure(
        Artifact::CurrentSelector,
        Boundary::WindowAdmission,
        0,
        Cause::Admission(cause),
    )
}

fn allocation_failure(
    artifact: Artifact,
    boundary: Boundary,
    requested: u64,
    cause: Cause,
) -> DiscoveryFailure {
    let mut failure = DiscoveryFailure::from(PhysicalRecoveryBlockKind::DiscoveryLimit);
    failure
        .source_denials
        .push(PhysicalRecoverySourceDenial::SourceReadAllocation {
            artifact,
            boundary,
            requested,
            cause,
        });
    failure
}
