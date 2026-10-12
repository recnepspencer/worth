//! Typed caller stops crossing the workflow's native mutation door.
use crate::domain_installation::WorthQueryWorkflowAdvanceDenialKind;
use crate::memory_workspace::WorthQueryWorkspaceErrorKind as Workspace;
use crate::runtime::WorthQueryRuntimeError as Runtime;

pub(super) fn native(error: &Runtime, stage: &str) -> Option<WorthQueryWorkflowAdvanceDenialKind> {
    let cause = match error {
        Runtime::Workspace(error) => match error.kind() {
            Workspace::ExecutionDenied(cause) => Ok(cause),
            Workspace::ExecutionControlStopped(cause) => Err(cause),
            _ => return None,
        },
        Runtime::IntentExecutionRoutingFailed { source, .. } => return native(source, stage),
        _ => return None,
    };
    Some(WorthQueryWorkflowAdvanceDenialKind::StageExecutionRequest {
        stage_identity: stage.into(),
        cause,
    })
}
