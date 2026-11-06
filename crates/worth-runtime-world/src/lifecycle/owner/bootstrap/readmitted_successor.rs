//! Reconstruct original meaning from the real Native cap without live-result fiction.
use super::*;
use crate::basis::CompositeBasisBackingDenial;
use crate::durability::{WorldOpenAttemptRecord, WorldReadmissionDenial as Denial};
use crate::history::{ReadmittedPublicationRecordBacking, ReadmittedWorldCommitDenial};
use worth_relational::facade::durability::RelationalRecoveredPublicationCustody;

pub(crate) struct PreparedReadmittedWorldSuccessor {
    pub(crate) after: ProductBranchReferenceSnapshot,
    pub(crate) attempt: crate::identity::CompositePublicationAttemptIdentity,
    pub(crate) record: ReadmittedPublicationRecordBacking,
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn prepare_readmitted_successor(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        before: &ProductBranchObservation,
        open: &WorldOpenAttemptRecord,
        native: &RelationalRecoveredPublicationCustody,
    ) -> Result<PreparedReadmittedWorldSuccessor, Denial> {
        let owner = self.owner_identity();
        if native.predecessor().admission_identity()
            != before.basis().relational_basis().admission_identity()
        {
            return Err(Denial::OccurrenceEvidenceMismatch);
        }
        let commit = crate::identity::readmission::readmit_commit(owner, open.commit)
            .ok_or(Denial::ForeignOwner)?;
        let attempt =
            crate::identity::readmission::readmit_attempt(owner, open.association.attempt())
                .ok_or(Denial::ForeignOwner)?;
        let generation = before
            .reference_generation()
            .advance()
            .map_err(Denial::IdentityExhaustion)?;
        let record =
            ReadmittedPublicationRecordBacking::prepare(request).map_err(Denial::Medium)?;
        let basis = crate::basis::admit_validated_for_request(
            request,
            &self.state.identities,
            native.successor().clone(),
            before.basis().signal_basis().clone(),
            before.basis().correspondence_basis().clone(),
        )
        .map_err(|cause| match cause {
            CompositeBasisBackingDenial::Request(cause) => {
                Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
            }
            CompositeBasisBackingDenial::CapacityOverflow => Denial::RecoveryCapacity,
        })?;
        let commit = CompositeRuntimeWorldCommit::from_readmitted_native_publication(
            request,
            commit,
            before.snapshot().commit(),
            basis,
            attempt.clone(),
            native,
        )
        .map_err(|cause| match cause {
            ReadmittedWorldCommitDenial::Request(cause) => {
                Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
            }
            ReadmittedWorldCommitDenial::CapacityOverflow => Denial::RecoveryCapacity,
            ReadmittedWorldCommitDenial::Native(cause) => Denial::NativeSuccessor(cause),
            ReadmittedWorldCommitDenial::Construction(
                crate::history::CompositeCommitConstructionDenial::OwnerMismatch
                | crate::history::CompositeCommitConstructionDenial::BasisMismatch,
            ) => Denial::OccurrenceEvidenceMismatch,
        })?;
        let after = ProductBranchReferenceSnapshot::owner_issued(
            owner,
            before.branch_identity().clone(),
            before.lifecycle_incarnation(),
            generation,
            commit,
        )
        .map_err(|_| Denial::BranchLineage)?;
        Ok(PreparedReadmittedWorldSuccessor {
            after,
            attempt,
            record,
        })
    }
}
