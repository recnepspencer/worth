//! Read ceilings and computation ceilings share the operation's execution outcome.
use super::WorthQueryTemporalReentryDenial as Denial;
use crate::domain_computation::{WorthQueryProviderSessionControlStopKind as Control, WorthQueryProviderSessionDenialKind as Kind};
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
            stage: crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage::DecisionReadSet,
            cause: Ok(kind),
        }
    }
    pub(super) fn from_native_stop(
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    ) -> Self {
        Self::from_read_stop(crate::facade::primary_graph::relational_execution_kind(
            cause, None,
        ))
    }
}
