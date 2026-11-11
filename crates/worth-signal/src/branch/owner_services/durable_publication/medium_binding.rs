//! The branch cutover subject is sealed by its exact original medium answer.
use super::effect::PreparedSignalOwnerEffect;
use super::prepared::SignalPreparedDurableAdvance;
use crate::branch::owner_services::owner::fork_reservation::SignalPreparedOwnedFork;
use crate::durability::publication_binding::{
    admit_branch_record, SignalCommittedAdvanceRecord, SignalPublicationBindingDenial,
};
use crate::durability::readmission::SignalDefinitionSourceRecord;
use crate::durability::{SignalBranchAdvanceArtifact, SignalPublicationCompletion};
use std::sync::Arc;
use worth_foundational::facade::DurablePublicationFate;
use worth_proof::{AuthorityMarker, DurablyCommitted};

pub(in crate::branch::owner_services) struct SignalCommittedBranchAdvance<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    prepared: SignalPreparedDurableAdvance<D, I, T>,
    record: SignalCommittedAdvanceRecord,
}
impl<D, I, T> AuthorityMarker for SignalCommittedBranchAdvance<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
}
impl<D, I, T> SignalCommittedBranchAdvance<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn into_original(
        self,
    ) -> (
        SignalPreparedDurableAdvance<D, I, T>,
        SignalCommittedAdvanceRecord,
    ) {
        (self.prepared, self.record)
    }
}
pub(in crate::branch::owner_services) struct SignalCommittedBranchFork<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    prepared: SignalPreparedOwnedFork<D, I, T>,
    record: SignalCommittedAdvanceRecord,
}
impl<D, I, T> AuthorityMarker for SignalCommittedBranchFork<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
}
impl<D, I, T> SignalCommittedBranchFork<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) fn into_original(
        self,
    ) -> (
        SignalPreparedOwnedFork<D, I, T>,
        SignalCommittedAdvanceRecord,
    ) {
        (self.prepared, self.record)
    }
}
pub(super) enum SignalCommittedOwnerEffect<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    Advanced(DurablyCommitted<SignalCommittedBranchAdvance<D, I, T>>),
    Forked(DurablyCommitted<SignalCommittedBranchFork<D, I, T>>),
}
pub(super) fn admit_publication<D, I, T>(
    prepared: PreparedSignalOwnerEffect<D, I, T>,
    artifact: SignalBranchAdvanceArtifact,
    definition: Arc<SignalDefinitionSourceRecord>,
    completion: SignalPublicationCompletion,
) -> Result<
    SignalCommittedOwnerEffect<D, I, T>,
    (
        PreparedSignalOwnerEffect<D, I, T>,
        SignalBranchAdvanceArtifact,
        DurablePublicationFate<SignalPublicationBindingDenial>,
    ),
>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    match admit_branch_record(artifact, definition, completion) {
        Ok(record) => Ok(match prepared {
            PreparedSignalOwnerEffect::Advanced(prepared) => SignalCommittedOwnerEffect::Advanced(
                DurablyCommitted::from_owner(SignalCommittedBranchAdvance {
                    prepared: *prepared,
                    record,
                }),
            ),
            PreparedSignalOwnerEffect::Forked(prepared) => SignalCommittedOwnerEffect::Forked(
                DurablyCommitted::from_owner(SignalCommittedBranchFork {
                    prepared: *prepared,
                    record,
                }),
            ),
        }),
        Err((artifact, fate)) => Err((prepared, artifact, fate)),
    }
}
