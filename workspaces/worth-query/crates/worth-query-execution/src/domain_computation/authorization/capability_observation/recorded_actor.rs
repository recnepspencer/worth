//! Revalidation of native-carried evidence at this exact admission basis.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::workflow_transition_program::WorkflowRecordedActorObservation;
use std::sync::Arc;

pub(super) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    request: &WorthQueryRetainedCapabilityRequest,
) -> Result<Option<Arc<WorkflowRecordedActorObservation>>, WorthQueryOperationAuthorizationDenial> {
    let Some(native) = request.recorded_actor() else {
        return Ok(None);
    };
    if !native
        .remains_current(execution, relational, snapshot)
        .map_err(|cause| {
            super::super::denial::relational_read_denial(cause, "workflow recorded role")
        })?
    {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::StaleAuthorization,
            ),
            "original workflow recorded-role evidence",
        ));
    }
    Ok(Some(Arc::clone(native)))
}
