//! A source C holds and the candidate removes is admitted and classified at C.
//! It has no D description, so it never reaches a D read or a producer.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryControls, WorthQueryAuthenticatedPrincipal,
    WorthQueryEntityResolutionDenial, WorthQuerySelectedProductOperation,
};
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_relational::facade::mvcc::RelationalDeclaredCoupledOwner as Owner;
type Scope<S, B> = <<SourceBinding<S, B> as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;

/// Only the catalog may name the removed source: a route key, its retirement
/// or its restoration. Every other demand keeps the D absence as its refusal.
pub fn read<S, B, P, I>(
    installed: &TypedInstalledProducer<S, B>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    request: &WorthQueryRequestScope,
    selected: WorthQuerySelectedProductOperation<'_, S>,
    principal: &WorthQueryAuthenticatedPrincipal<S, P, I>,
    scope: &WorthQueryApplicationEntityIdentity<S, Scope<S, B>>,
    parameters: ApplicationQueryParameterSet<SourceQuery<S, B>>,
    applied_absence: WorthQueryEntityResolutionDenial,
    assembly: &worth_relational::facade::mvcc::RelationalAuthoredCoupledCandidate<'_>,
    demand: &DeclaredCoupledOutputDemand,
    purpose: CoupledSourceReadPurpose,
) -> Result<CoupledSourceRead, CoupledSourceReadStop>
where
    S: ApplicationSchema,
    B: WorthQueryApplicationProducerBinding<S>,
{
    let (_, product, application_basis) = selected.into_parts();
    let maximum_results = runtime
        .resolve_application_query_limits(installed.source_query.limits())
        .maximum_results();
    let controls = WorthQueryApplicationQueryControls::product_one_shot(
        product,
        application_basis,
        maximum_results,
        request,
    );
    let access = WorthQueryApplicationQueryAccessContext::new(principal, scope);
    let plan = runtime
        .admit_application_query(
            execution,
            installed.source_query.query(),
            &access,
            parameters,
            controls,
        )
        .map_err(|cause| query_admission_denied(B::IDENTITY, cause))?;
    let partition = *plan.parameter_binding_identity().bytes();
    let root = scope.entity_id();
    if purpose == CoupledSourceReadPurpose::Route {
        return Ok(CoupledSourceRead::Route {
            scope: root,
            partition,
        });
    }
    match assembly.read_parent_owner(execution, demand.family, root, &partition)? {
        Some(Owner::Retiring) => Ok(CoupledSourceRead::Retiring),
        Some(Owner::Restored(input)) => Ok(CoupledSourceRead::Restored { scope: root, input }),
        Some(Owner::Surviving(_) | Owner::RequiredSurviving(_)) | None => {
            Err(scope_rejected(B::IDENTITY, applied_absence).into())
        }
    }
}
