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
    use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind as Execution;
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    let kind=match crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause,None) {
                Ok(kind)=>WorthQueryOutputDemandDenialKind::SourceQueryExecution(Execution::ExecutionRequest(kind)),
                Err(Control::Cancelled)=>WorthQueryOutputDemandDenialKind::Cancelled,
                Err(Control::TimedOut)=>WorthQueryOutputDemandDenialKind::TimedOut,
            };
    WorthQueryOutputDemandDenial::new(kind, "retained output liveness")
}

/// An applied source can change while its old route still has custody. Native
/// liveness, independently of that custody, chooses the installed lifecycle.
pub(in crate::domain_computation::primary_graph) fn applied_lifecycle<S, F>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    observation: &worth_runtime_world::facade::ProductBranchObservation,
    root: worth_relational::facade::identity::EntityId,
    partition: [u8; 32],
    checkpoint: [u8; 32],
    pin: &worth_relational::facade::runtime::RelationalPinnedRead<
        worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    >,
    predecessor: Option<&worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
) -> Result<super::super::super::WorthQueryProducerLifecyclePosture, WorthQueryOutputDemandDenial>
where
    S: ApplicationSchema,
    F: WorthQueryProducerOutputFamily<S>,
{
    use super::super::super::WorthQueryProducerLifecyclePosture as Posture;
    let view = pin.project_candidate();
    if let Some(prior) = predecessor {
        // The current Native route is the actual selected publication. A warm
        // Query row for an older binding cannot choose its lifecycle instead.
        if let Some(entry) = application
            .installed_producers
            .entries
            .get(prior.producer())
        {
            for (role, _, posture, entity) in prior.members() {
                execution.consult().map_err(|cause| denial(cause.into()))?;
                if role == entry.declaration.output_role
                    && posture != worth_relational::facade::mvcc::RequiredOutputRolePosture::Retire
                    && read(execution, &view, entity)?
                {
                    return Ok(Posture::Preserve);
                }
            }
        }
        return Ok(Posture::Initial);
    }
    let bindings = application
        .installed_producers
        .family_output_bindings::<F>();
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root);
    let coordinate =
        crate::basis::WorthQueryProductBranchReadIdentity::from_observation(observation);
    let candidates = {
        let mut lineage = application
            .runtime
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        application
            .record_recovered_output_candidates(
                Some(execution),
                partition,
                &checkpoint,
                &coordinate,
                scope,
                &bindings,
                &mut lineage,
            )
            .map_err(|stop| super::selected_basis::selection_lookup_denial(stop, F::IDENTITY))?;
        lineage
            .retained_output_candidates_metered(
                application.runtime.authority_identity().as_u64(),
                &application.installed_schema.binding_identity(),
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                &bindings,
                partition,
                execution,
            )
            .map_err(|stop| super::selected_basis::selection_lookup_denial(stop, F::IDENTITY))?
    };
    for candidate in candidates {
        execution.consult().map_err(|cause| denial(cause.into()))?;
        let role = application
            .installed_producers
            .family_output_role::<F>(candidate.binding)?;
        if let Some(entity) = candidate.correspondence.active_entity_for_role(role)
            && read(execution, &view, entity)?
        {
            return Ok(Posture::Preserve);
        }
    }
    Ok(Posture::Initial)
}
