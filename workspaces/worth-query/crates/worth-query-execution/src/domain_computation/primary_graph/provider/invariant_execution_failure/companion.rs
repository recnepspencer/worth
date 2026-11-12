//! Keep Native companion admission and request-control causes intact.
use super::*;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop as Stop,
    transactions::{
        CommitExecutionDenial, CommitExecutionDenialKind, RelationalExecutionDenialCause as Cause,
    },
};

pub(in crate::domain_computation::primary_graph::provider) fn companion_failure(
    stop: Stop,
) -> WorthQueryInvariantExecutionFailure {
    let cause = match stop {
        Stop::Execution(cause) => cause,
        Stop::PreparationMemoryCounterOverflow => Cause::ChargedBytesOverflow,
        Stop::WorkCounterOverflow => Cause::WorkCounterOverflow,
        Stop::Interrupted(event) => return request_interruption(event.interruption()),
        cause => return WorthQueryInvariantExecutionFailure::publication_companion(cause),
    };
    map_execution_failure(CommitExecutionDenial {
        kind: CommitExecutionDenialKind::Cause(cause),
        partition_identity: None,
    })
}

pub(in crate::domain_computation::primary_graph::provider) fn head_cell_failure(
    stop: crate::domain_computation::primary_graph::output_lineage::HeadCellRegistrationStop,
) -> WorthQueryInvariantExecutionFailure {
    use crate::domain_computation::primary_graph::output_lineage::HeadCellRegistrationStop;
    match stop {
        HeadCellRegistrationStop::Admission(stop) => companion_failure(stop),
        HeadCellRegistrationStop::Native(stop) => {
            WorthQueryInvariantExecutionFailure::publication_cell_registration(stop)
        }
        HeadCellRegistrationStop::LookupChanged => WorthQueryInvariantExecutionFailure::new(
            WorthQueryInvariantExecutionDenialKind::ProductBasisStale,
            "source mark-cell lookup changed before producer qualification",
        ),
    }
}

pub(in crate::domain_computation::primary_graph::provider) fn readmission_failure(
    stop: worth_relational::facade::mvcc::TransactionCommitError,
) -> WorthQueryInvariantExecutionFailure {
    if matches!(&stop,
        worth_relational::facade::mvcc::TransactionCommitError::Publication { error, .. }
            if error.authored_retirement_refusal().is_some())
    {
        return map_validation_failure(stop);
    }
    match stop {
        native @ worth_relational::facade::mvcc::TransactionCommitError::Execution { .. } => {
            map_validation_failure(native)
        }
        native => WorthQueryInvariantExecutionFailure::publication_readmission(native),
    }
}
