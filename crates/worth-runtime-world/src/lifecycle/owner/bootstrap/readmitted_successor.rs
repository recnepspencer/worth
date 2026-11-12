//! Reconstruct original meaning from each actual recovered owner basis.
use super::*;
use crate::basis::CompositeBasisBackingDenial;
use crate::durability::{WorldOpenAttemptRecord, WorldReadmissionDenial as Denial};
use crate::history::RecoveredWorldOwnerEvidenceRef;
use crate::history::{ReadmittedPublicationRecordBacking, ReadmittedWorldCommitDenial};
use worth_runtime_bridge::facade::AdmittedRuntimeWorldCorrespondenceBasis;

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
        owners: RecoveredWorldOwnerEvidenceRef<'_>,
        bridge: AdmittedRuntimeWorldCorrespondenceBasis,
    ) -> Result<PreparedReadmittedWorldSuccessor, Denial> {
        let owner = self.owner_identity();
        if owners.relational().is_some_and(|native| {
            native.predecessor().admission_identity()
                != before.basis().relational_basis().admission_identity()
        }) || owners
            .signal()
            .is_some_and(|signal| signal.original().association() != Some(&open.association))
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
            owners.relational().map_or_else(
                || before.basis().relational_basis().clone(),
                |native| native.successor().clone(),
            ),
            owners.signal().map_or_else(
                || before.basis().signal_basis().clone(),
                |signal| signal.basis().clone(),
            ),
            bridge,
        )
        .map_err(|cause| match cause {
            CompositeBasisBackingDenial::Request(cause) => {
                Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
            }
            CompositeBasisBackingDenial::CapacityOverflow => Denial::RecoveryCapacity,
        })?;
        let commit = CompositeRuntimeWorldCommit::from_readmitted_publication(
            request,
            commit,
            before.snapshot().commit(),
            basis,
            attempt.clone(),
            owners,
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
