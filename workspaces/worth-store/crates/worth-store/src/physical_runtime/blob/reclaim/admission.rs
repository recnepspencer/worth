use crate::physical_runtime::{
    durability::{
        AdmittedFailedIngestDrop, AdmittedManifestResidueRetirement,
        AdmittedReleasedGenerationDrop, DisplacedArtifact, PhysicalBlobReclaimAdmissionDenial,
        PhysicalBlobSessionClaim,
    },
    AdmittedRecordPlacementPolicy, BlobPhysicalAllocation, BlobSessionId, PhysicalMutationDeadline,
    ServingPhysicalRuntime,
};

use super::{
    contracts::BlobReclaimSource, selection, BlobReclaimDeferral, BlobReclaimDisposition,
    BlobReclaimFailure, BlobReclaimObservation, BlobReclaimReceipt, BlobReclaimRequest,
};

mod released;

/// A pre-effect handle retaining Store claim, memory and reader/fence custody.
/// `wait` consumes it; dropping or cancelling it cannot publish a drop.
pub struct BlobReclaimHandle<'runtime> {
    pub(super) runtime: &'runtime ServingPhysicalRuntime,
    pub(super) placement: AdmittedRecordPlacementPolicy,
    pub(super) deadline: PhysicalMutationDeadline,
    pub(super) limits: super::BlobReclaimLimits,
    pub(super) admitted: Option<AdmittedBlobDrop>,
    pub(super) manifest_residue: Option<AdmittedManifestResidueRetirement>,
    pub(super) observation: BlobReclaimObservation,
    pub(super) _claim: PhysicalBlobSessionClaim,
    pub(super) _allocation: BlobPhysicalAllocation<'runtime>,
}

pub(super) enum AdmittedBlobDrop {
    Failed(AdmittedFailedIngestDrop),
    Released(AdmittedReleasedGenerationDrop),
}

impl AdmittedBlobDrop {
    pub(super) fn dropped(&self) -> &[worth_store_physical_format::PersistedRecordIdentity] {
        match self {
            Self::Failed(value) => value.dropped(),
            Self::Released(value) => value.dropped(),
        }
    }

    pub(super) fn displaced(&self) -> &[DisplacedArtifact] {
        match self {
            Self::Failed(value) => value.displaced(),
            Self::Released(value) => value.displaced(),
        }
    }

    pub(super) fn remaining(&self) -> u64 {
        match self {
            Self::Failed(value) => value.remaining(),
            Self::Released(value) => value.remaining(),
        }
    }

    pub(super) fn complete(self) -> bool {
        match self {
            Self::Failed(value) => value.complete(),
            Self::Released(value) => value.complete(),
        }
    }
}

impl BlobReclaimHandle<'_> {
    pub const fn observation(&self) -> BlobReclaimObservation {
        self.observation
    }

    pub fn cancel(self) -> BlobReclaimDisposition {
        BlobReclaimDisposition::ProvenNoEffect
    }

    pub fn wait(self) -> Result<BlobReclaimReceipt, BlobReclaimFailure> {
        super::execution::execute(self)
    }
}

impl ServingPhysicalRuntime {
    pub(in crate::physical_runtime) fn reclaim_blob(
        &self,
        request: BlobReclaimRequest<'_>,
    ) -> Result<BlobReclaimHandle<'_>, BlobReclaimFailure> {
        let source = request.source;
        self.blobs()
            .map_err(|_| BlobReclaimFailure::ServingRequiresInspection)?;
        let source_store = match &source {
            BlobReclaimSource::Abandoned { token, .. } => token.store,
            BlobReclaimSource::Released(proof) => proof.store(),
        };
        if source_store != self.store_identity().bytes() {
            return Err(BlobReclaimFailure::ForeignStore);
        }
        let memory = request
            .limits
            .memory_bytes()
            .map_err(|_| BlobReclaimFailure::ScratchUnavailable)?;
        let allocation = self
            .physical_allocations()
            .admit_blob(memory)
            .map_err(BlobReclaimFailure::Allocation)?;
        let (token, scope) = match source {
            BlobReclaimSource::Abandoned { token, scope } => (token, scope),
            BlobReclaimSource::Released(proof) => {
                return released::admit(
                    self,
                    proof,
                    request.placement,
                    request.deadline,
                    request.limits,
                    memory,
                    allocation,
                );
            }
        };
        let (reader, mut claim, checkpoint) = self
            .claimed_blob_reclaim_reader(BlobSessionId::from_selected(token.session))
            .map_err(|cause| BlobReclaimFailure::Claim(cause.into()))?;
        let selected = selection::select(
            self,
            reader,
            token,
            scope,
            request.limits,
            request.placement,
            checkpoint,
            &allocation,
        )?;
        let (admitted, manifest_residue, work) = if selected.is_empty() {
            if selected.remaining() != 0 {
                return Err(BlobReclaimFailure::Deferred(
                    BlobReclaimDeferral::SharedReferences,
                ));
            }
            let (manifest_residue, work) = self.admit_selected_blob_manifest_residue(
                selected,
                &mut claim,
                &allocation,
                request.limits,
                request.placement,
            )?;
            (None, manifest_residue, work)
        } else {
            let work = selected.inspection();
            (
                Some(AdmittedBlobDrop::Failed(
                    self.admit_blob_reclaim_drop(selected, &mut claim, &allocation)
                        .map_err(admission_failure)?,
                )),
                None,
                work,
            )
        };
        let observation = BlobReclaimObservation {
            inspected_records: work.records,
            inspected_payload_bytes: work.bytes,
            admitted_memory_bytes: memory.get(),
        };
        Ok(BlobReclaimHandle {
            runtime: self,
            placement: request.placement,
            deadline: request.deadline,
            limits: request.limits,
            admitted,
            manifest_residue,
            observation,
            _claim: claim,
            _allocation: allocation,
        })
    }
}

fn admission_failure(cause: PhysicalBlobReclaimAdmissionDenial) -> BlobReclaimFailure {
    use PhysicalBlobReclaimAdmissionDenial as Denial;
    let deferred = match cause {
        Denial::Claim(cause) => return BlobReclaimFailure::Claim(cause.into()),
        Denial::ExternalProtectedReader => BlobReclaimDeferral::ProtectedReader,
        Denial::PendingPublication => BlobReclaimDeferral::PendingPublication,
        Denial::AlreadyFenced => BlobReclaimDeferral::CompetingReclaim,
        Denial::SourceRootChanged => BlobReclaimDeferral::SourceRootChanged,
        Denial::Capacity => BlobReclaimDeferral::RetirementCapacity,
        Denial::EntropyUnavailable | Denial::AttemptCollision => {
            return BlobReclaimFailure::AttemptIdentityUnavailable
        }
        Denial::SelectedResidueInvalid => return BlobReclaimFailure::ConflictingSelectedFate,
        Denial::RouteUnavailable => return BlobReclaimFailure::RouteUnavailable,
    };
    BlobReclaimFailure::Deferred(deferred)
}
