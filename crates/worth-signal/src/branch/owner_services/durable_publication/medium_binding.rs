//! The branch cutover subject is sealed by its exact original medium answer.
use super::prepared::SignalPreparedDurableAdvance;
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
pub(super) fn admit_advance<D, I, T>(
    prepared: SignalPreparedDurableAdvance<D, I, T>,
    artifact: SignalBranchAdvanceArtifact,
    definition: Arc<SignalDefinitionSourceRecord>,
    completion: SignalPublicationCompletion,
) -> Result<
    DurablyCommitted<SignalCommittedBranchAdvance<D, I, T>>,
    (
        SignalPreparedDurableAdvance<D, I, T>,
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
        Ok(record) => Ok(DurablyCommitted::from_owner(SignalCommittedBranchAdvance {
            prepared,
            record,
        })),
        Err((artifact, fate)) => Err((prepared, artifact, fate)),
    }
}
