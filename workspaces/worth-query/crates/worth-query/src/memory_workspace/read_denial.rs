//! Preserve Relational's request cause at the workspace read boundary.
use super::{WorthQueryWorkspaceError, WorthQueryWorkspaceErrorKind};

pub(super) fn execution(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> WorthQueryWorkspaceError {
    let kind = match worth_query_execution::facade::primary_graph::relational_execution_kind(
        cause, None,
    ) {
        Ok(kind) => WorthQueryWorkspaceErrorKind::ExecutionDenied(kind),
        Err(kind) => WorthQueryWorkspaceErrorKind::ExecutionControlStopped(kind),
    };
    WorthQueryWorkspaceError::with_kind(kind, format!("workspace read stopped: {cause:?}"))
}

impl WorthQueryWorkspaceError {
    /// Preserve a native retained-read refusal at the workspace boundary.
    pub fn observation_read_denied(
        denial: worth_relational::facade::change_source::RelationalObservationReadDenial,
    ) -> Self {
        use worth_relational::facade::change_source::RelationalObservationReadDenial as Denial;
        match denial {
            Denial::Execution(cause) => execution(cause),
            Denial::ForeignObservation => Self::with_kind(
                WorthQueryWorkspaceErrorKind::RelationalBasisUnavailable,
                "retained observation belongs to another runtime",
            ),
        }
    }
}
