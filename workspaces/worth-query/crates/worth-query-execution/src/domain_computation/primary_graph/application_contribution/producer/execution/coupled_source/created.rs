//! A new candidate source is described at D, with principal and READ at C.
use super::*;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity;
type Scope<S, B> = <<SourceBinding<S, B> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;
pub(super) fn read<S, B>(
    installed: &TypedInstalledProducer<S, B>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    external: &WorthQueryAuthenticatedExternalPrincipal<S>,
    request: &WorthQueryRequestScope,
    selected: crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'_, S>,
    before: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    value: &worth_foundational::facade::AspectValue,
    parameters: &crate::domain_computation::primary_graph::application_query::CapturedApplicationQueryParameters,
    current_absence: crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial,
    assembly: &worth_relational::facade::mvcc::RelationalAuthoredCoupledCandidate<'_>,
    demand: &DeclaredCoupledOutputDemand,
) -> Result<CoupledSourceRead, CoupledSourceReadStop>
where
    S: ApplicationSchema,
    B: WorthQueryApplicationProducerBinding<S>,
    SourceValue<S, B>: crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
        S,
        SourceQuery<S, B>,
    >,
{
    let shared = selected
        .prepare_shared_query_basis()
        .map_err(|(_, stop)| source_readmission::ready_resource_denial(B::IDENTITY, stop))?;
    let current = shared.selected();
    let principal = current
        .resolve_authenticated_principal_issued(
            execution,
            installed.source_query.principal_binding(),
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
            None,
        )
        .map_err(|cause| principal_rejected(B::IDENTITY, cause))?;
    let foreign = || {
        denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            demand.connection,
        )
    };
    let graph = runtime.runtime.primary_graph().ok_or_else(foreign)?;
    let field = SourceBinding::<S, B>::scope_field();
    let selected_scope = graph
        .retain_entity_resolution_context()
        .resolve_applied_identity::<S, Scope<S, B>>(
            execution,
            before,
            candidate,
            field.entity(),
            field.aspect(),
            field.field(),
            value,
        );
    if let Some(lapse) = principal.request_lapse() {
        return Err(request_authority_denied(
            B::IDENTITY,
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(
                lapse.denial_kind(),
                B::IDENTITY,
            ),
        )
        .into());
    }
    let scope = selected_scope.map_err(|cause| {
        let disclosed = if matches!(
            cause.kind(),
            crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(
                _
            )
        ) {
            current_absence.clone()
        } else {
            cause
        };
        scope_rejected(B::IDENTITY, disclosed)
    })?;
    let (product, basis) = runtime
        .retain_selected_query_basis_admitted(&shared)
        .map_err(|stop| failed(B::IDENTITY, stop))?;
    let limits = runtime.resolve_application_query_limits(installed.source_query.limits());
    let controls = crate::domain_computation::primary_graph::WorthQueryApplicationQueryControls::selected_read_one_shot(product, basis, limits.maximum_results(), request)
        .map_err(|stop| source_readmission::ready_resource_denial(B::IDENTITY, stop))?;
    let access =
        crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessContext::new(
            principal.principal(),
            &scope,
        );
    let plan = runtime
        .admit_applied_source_query(
            execution,
            installed.source_query.query(),
            access,
            parameters,
            controls,
            current,
            &principal,
            before,
            candidate,
            request,
        )
        .map_err(|cause| {
            // Hide only source/permission diagnostics before C READ succeeds.
            // Authentication, resource and request failures keep their true cause.
            use crate::domain_computation::authorization::{
                WorthQueryOperationAuthorizationDenialKind as Authorization,
                WorthQueryOperationAuthorizationRefusalKind as Refusal,
            };
            if matches!(cause.kind(), crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
                Authorization::Refused(Refusal::PermissionDenied | Refusal::ScopeMismatch | Refusal::RelationalObservationRejected),
            )) {
                scope_rejected(B::IDENTITY, current_absence)
            } else { query_admission_denied(B::IDENTITY, cause) }
        })?;
    if demand.reuse_prepared {
        if let Some(index) = assembly.prepared_route_index(
            execution,
            demand.family,
            scope.entity_id(),
            parameters.parameters().identity().bytes(),
        )? {
            return Ok(CoupledSourceRead::Prepared(index));
        }
    }
    match assembly.declared_owner(
        execution,
        demand.family,
        scope.entity_id(),
        parameters.parameters().identity().bytes(),
    )? {
        Some(worth_relational::facade::mvcc::RelationalDeclaredCoupledOwner::Retiring) => {
            return Ok(CoupledSourceRead::Retiring)
        }
        Some(worth_relational::facade::mvcc::RelationalDeclaredCoupledOwner::Surviving(_)) => {
            return Err(foreign().into())
        }
        None => {}
    }
    let source = runtime
        .project_applied_source_plan(execution, plan, candidate)
        .map_err(|cause| failed(B::IDENTITY, cause))?;
    let profile =
        <B::OutputFamily as super::super::super::WorthQueryProducerOutputFamily<S>>::profile_kind(
            &source.value,
        );
    Ok(CoupledSourceRead::Source(ErasedAppliedProducerSource {
        lifecycle: super::super::super::WorthQueryProducerLifecyclePosture::Initial,
        predecessor: None,
        profile,
        source: Box::new(source),
    }))
}
