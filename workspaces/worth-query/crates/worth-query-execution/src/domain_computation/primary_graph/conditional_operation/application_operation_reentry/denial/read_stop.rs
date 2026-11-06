//! Read ceilings and computation ceilings share the operation's execution outcome.
use super::WorthQueryTemporalReentryDenial as Denial;
use {crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control, crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind as Kind};
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
        Terminal::ApplicationExecution {
            stage: crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
            cause: Ok(kind),
        }
    }
    pub(super) fn from_native_stop(
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    ) -> Self {
        Self::from_read_stop(crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(
            cause, None,
        ))
    }
}
