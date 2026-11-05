//! Retained output liveness keeps request failures separate from absent output.
use super::*;
pub fn read(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &worth_relational::facade::runtime::VisibilityProjectionView,
    entity: worth_relational::facade::identity::EntityId,
) -> Result<bool, WorthQueryOutputDemandDenial> {
    view.entity_record_with_projection_scope(
        execution,
        entity,
        worth_relational::facade::runtime::ProjectionAspectScope::empty(),
        |record| Some(record.lifecycle()),
    )
    .map(|state| state == Some(worth_relational::facade::storage::RecordLifecycleState::Live))
    .map_err(denial)
}
pub fn denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> WorthQueryOutputDemandDenial {
    use crate::domain_computation::primary_graph::vocabulary::application_query::one_shot_denial::WorthQueryApplicationOneShotDenialKind as Execution;
    use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control;
    let kind=match crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause,None) {
                Ok(kind)=>WorthQueryOutputDemandDenialKind::SourceQueryExecution(Execution::ExecutionRequest(kind)),
                Err(Control::Cancelled)=>WorthQueryOutputDemandDenialKind::Cancelled,
                Err(Control::TimedOut)=>WorthQueryOutputDemandDenialKind::TimedOut,
            };
    WorthQueryOutputDemandDenial::new(kind, "retained output liveness")
}
