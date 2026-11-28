pub mod preflight;
pub use preflight::preflight_execution_cause;
use worth_query_execution_contracts::facade::commit_execution_kind;
pub use worth_query_execution_contracts::facade::relational_execution_kind;

/// Preparation executes before publication; a refusal never asks for recovery.
pub fn relational_execution_stop(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    partition_identity: Option<u64>,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped, WorthQueryProviderSessionCommitStop as Stop,
        WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
    };
    let detail = format!("Relational execution stopped: {cause:?} at {partition_identity:?}");
    match relational_execution_kind(cause, partition_identity) {
        Ok(kind) => Stop::PreEffectDenied(
            WorthQueryProviderSessionFailure::new(
                kind, WorthQueryProviderSessionProtocolStage::Commit, detail, Default::default(),
            ).with_native_execution_denial(worth_relational::facade::transactions::CommitExecutionDenial {
                kind: worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause),
                partition_identity,
            }),
        ),
        Err(kind) => Stop::ControlStopped(
            WorthQueryProviderSessionCommitControlStopped::execution_preparation(kind, detail),
        ),
    }
}

/// Native preparation evidence keeps the typed cause at the application boundary.
pub fn native_preparation_kind(
    failure: &crate::domain_computation::WorthQueryProviderSessionFailure,
) -> Result<
    Option<crate::domain_computation::WorthQueryProviderSessionDenialKind>,
    crate::domain_computation::WorthQueryProviderSessionControlStopKind,
> {
    if let Some(stop) = failure.checked_read_stop() {
        return stop.map(Some);
    }
    use worth_relational::facade::transactions::TransactionCommitError;
    let Some(TransactionCommitError::Execution { denial, .. }) = failure.native_preparation_error()
    else {
        return Ok(None);
    };
    commit_execution_kind(*denial).map(Some)
}

/// Query reads retain native resource and control causes through this same conversion.
pub fn query_read_execution_kind(
    stop: &worth_relational::facade::runtime::QueryReadExecutionStop,
) -> Result<
    Option<crate::domain_computation::WorthQueryProviderSessionDenialKind>,
    crate::domain_computation::WorthQueryProviderSessionControlStopKind,
> {
    let Some(denial) = stop.execution_denial() else {
        return Ok(None);
    };
    commit_execution_kind(denial).map(Some)
}

#[cfg(test)]
#[path = "relational_execution_denial/tests.rs"]
mod tests;

pub fn request_commit_stop(
    cause: worth_execution::WorkCeilingDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    use crate::domain_computation::{
        WorthQueryProviderSessionCommitControlStopped, WorthQueryProviderSessionCommitStop as Stop,
    };
    match crate::domain_computation::primary_graph::application_contribution::publication_execution_kind(cause) {
        Ok(kind) => Stop::PreEffectDenied(crate::domain_computation::WorthQueryProviderSessionFailure::checked_read_stopped(Ok(kind), crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit, "coupled publication request stopped before effect")),
        Err(kind) => Stop::ControlStopped(WorthQueryProviderSessionCommitControlStopped::new(kind, "coupled publication request stopped before effect")),
    }
}
