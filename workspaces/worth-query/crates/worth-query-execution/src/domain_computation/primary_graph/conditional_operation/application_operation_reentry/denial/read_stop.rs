//! Read ceilings and computation ceilings share the operation's execution outcome.
use super::WorthQueryTemporalReentryDenial as Denial;
use crate::domain_computation::{WorthQueryProviderSessionControlStopKind as Control, WorthQueryProviderSessionDenialKind as Kind, WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage};
use crate::domain_computation::primary_graph::conditional_operation::application_operation_reentry::{WorthQueryTemporalControlStop, WorthQueryTemporalTerminalFailure as Terminal};
impl Denial {
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn from_read_stop(
        stop: Result<Kind, Control>,
    ) -> Self {
        match stop {
            Err(Control::Cancelled) => {
                Self::ControlStopped(WorthQueryTemporalControlStop::Cancelled)
            }
            Err(Control::TimedOut) => Self::ControlStopped(WorthQueryTemporalControlStop::TimedOut),
            Ok(kind) => Self::ExecutionStopped(kind),
        }
    }
    pub(in crate::domain_computation::primary_graph::conditional_operation) fn execution_failure(
        kind: Kind,
    ) -> Terminal {
        let failure = WorthQueryProviderSessionFailure::checked_read_stopped(
            Ok(kind),
            WorthQueryProviderSessionProtocolStage::Commit,
            "operation read stopped",
        );
        let commit = crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied(failure);
        Terminal::ApplicationCommit(commit.kind())
    }
    pub(super) fn from_native_stop(
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    ) -> Self {
        Self::from_read_stop(crate::facade::primary_graph::relational_execution_kind(
            cause, None,
        ))
    }
}
