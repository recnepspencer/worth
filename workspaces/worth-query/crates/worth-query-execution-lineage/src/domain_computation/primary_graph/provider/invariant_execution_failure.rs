pub mod companion;
use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantExecutionFailure,
};
pub use companion::{companion_failure, head_cell_failure, readmission_failure};

pub fn map_transaction_admission_failure(
    denial: worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial,
) -> WorthQueryInvariantExecutionFailure {
    use worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial as Denial;
    match denial {
        Denial::StaleBasis => WorthQueryInvariantExecutionFailure::new(
            WorthQueryInvariantExecutionDenialKind::ProductBasisStale,
            "the exact product basis became stale before invariant candidate admission",
        ),
        Denial::Cancelled => request_interruption(
            worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled,
        ),
        Denial::TimedOut => request_interruption(
            worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut,
        ),
        Denial::RetentionCapacityExhausted => retention_capacity_failure(),
        Denial::RetentionIdentityExhausted => exhausted_failure(
            WorthQueryInvariantExecutionDenialKind::RetentionIdentityExhausted,
            "Relational invariant execution exhausted retention identity space",
        ),
        _ => owner_failure(),
    }
}

pub fn map_transaction_staging_failure(
    denial: worth_relational::facade::mvcc::RelationalTransactionStagingDenial,
) -> WorthQueryInvariantExecutionFailure {
    use worth_relational::facade::mvcc::RelationalTransactionStagingDenial as Denial;
    let kind = match denial {
        native @ (Denial::SymbolIdentityExhausted
        | Denial::AllocationDenied(_)
        | Denial::CardinalityOverflow
        | Denial::InputDirectoryAllocationDenied { .. }) => {
            return WorthQueryInvariantExecutionFailure::native_staging(
                native,
                "Relational staging owner refused backing, cardinality or symbol identity",
            );
        }
        Denial::SavepointCapacityExhausted { maximum_savepoints } => {
            WorthQueryInvariantExecutionDenialKind::SavepointCapacityExhausted {
                maximum_savepoints,
            }
        }
        Denial::SavepointIdentityExhausted => {
            WorthQueryInvariantExecutionDenialKind::SavepointIdentityExhausted
        }
        Denial::MaterializationAuthorityRequired => {
            return provider_failure(
                "Relational invariant transaction requires materialization authority",
            );
        }
        Denial::MaterializationModeMismatch => {
            return provider_failure(
                "Relational invariant transaction materialization mode does not match its intents",
            );
        }
    };
    exhausted_failure(
        kind,
        "Relational invariant transaction staging exhausted an owner budget",
    )
}

pub fn map_validation_failure(
    failure: worth_relational::facade::mvcc::TransactionCommitError,
) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::relational_validation(failure)
}

pub fn map_execution_failure(
    denial: worth_relational::facade::transactions::CommitExecutionDenial,
) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::relational_execution(denial)
}

fn retention_capacity_failure() -> WorthQueryInvariantExecutionFailure {
    exhausted_failure(
        WorthQueryInvariantExecutionDenialKind::RetentionCapacityExhausted,
        "Relational invariant execution exhausted owner retention capacity",
    )
}

fn exhausted_failure(
    kind: WorthQueryInvariantExecutionDenialKind,
    detail: &'static str,
) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::exhausted(kind, detail)
}

fn owner_failure() -> WorthQueryInvariantExecutionFailure {
    provider_failure("Relational rejected the installed proposed-state invariant")
}

fn provider_failure(detail: impl Into<std::sync::Arc<str>>) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::ProviderRejected,
        detail,
    )
}

pub fn request_interruption(
    reason: worth_relational::facade::mvcc::RelationalOperationInterruption,
) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(
        WorthQueryInvariantExecutionDenialKind::RequestInterrupted(reason),
        "the admitted request interrupted Relational candidate validation",
    )
}
