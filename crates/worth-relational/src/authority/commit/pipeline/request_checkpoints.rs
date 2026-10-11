//! Named safe points of one borrowed commit request; no stage fee is charged.
use crate::transactions::data::{
    CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CommitStage {
    Validation,
    Admission,
    Draft,
    Boundary,
    Mutation,
    History,
    Snapshot,
    Artifact,
    PublicationPreparation,
    CandidateRegistration,
    Publication,
}

pub(crate) fn consult_stage(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    _stage: CommitStage,
) -> Result<(), TransactionCommitError> {
    #[cfg(test)]
    observation::visit(_stage);
    execution.consult().map_err(|denial| {
        use worth_execution::{MapKernelFailure, WorkCeilingDenial};
        let cause = match denial {
            WorkCeilingDenial::Admission(denial) => crate::execution::denial::lease_denial(denial),
            WorkCeilingDenial::Stopped(stop) => {
                crate::execution::kernel_failure(MapKernelFailure::Stop(stop))
            }
            WorkCeilingDenial::Panicked => {
                crate::execution::kernel_failure(MapKernelFailure::Panic)
            }
        };
        TransactionCommitError::execution(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        })
    })
}

#[cfg(test)]
pub(super) mod observation;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod stage_counts;
