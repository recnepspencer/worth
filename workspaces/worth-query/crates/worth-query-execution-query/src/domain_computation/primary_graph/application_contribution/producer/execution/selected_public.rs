//! Exact selected conventional authority for one required producer execution.

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationInputScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolution;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolutionMode;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

use super::*;
mod applied;
use crate::domain_computation::authorization::SelectedConventionalAdmissionStop;
use crate::domain_computation::authorization::{
    authorize_public_mutation_on_selected,
};
use crate::domain_computation::primary_graph::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial;
use crate::domain_computation::primary_graph::WorthQueryIssuedSelectedPrincipal;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial;
pub use applied::prepare_applied_public_mutation;

type Mutation<Schema, Binding> =
    <Binding as WorthQueryApplicationProducerBinding<Schema>>::Operation;
type MutationInput<Schema, Binding> =
    <Mutation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Input;
type MutationScope<Schema, Binding> = <<Mutation<Schema, Binding> as ApplicationMutationBinding<
    Schema,
>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope;
type MutationPrincipal<Schema, Binding> =
    <Mutation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Principal;
type MutationPrincipalIdentity<Schema, Binding> =
    <Mutation<Schema, Binding> as ApplicationMutationBinding<Schema>>::PrincipalIdentity;

pub enum SelectedPublicPreparationStop {
    ForeignSelectedRuntime,
    Issuer,
    Principal(WorthQueryPrincipalResolutionDenial),
    Scope(WorthQueryEntityResolutionDenial),
    UnsupportedScopeMode,
    Head(ProducerExecutionStop),
    MovedHead,
    Operation(SelectedConventionalAdmissionStop),
}

pub struct PreparedSelectedPublicMutation<'selected, 'runtime, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    MutationScope<Schema, Binding>: ApplicationEntityMarkerIdentity<Schema>,
{
    pub principal: WorthQueryIssuedSelectedPrincipal<
        'selected,
        'runtime,
        Schema,
        MutationPrincipal<Schema, Binding>,
        MutationPrincipalIdentity<Schema, Binding>,
    >,
    pub scope: Option<
        crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            MutationScope<Schema, Binding>,
        >,
    >,
    pub operation: crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
        Schema,
        <Mutation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Operation,
        MutationInput<Schema, Binding>,
        MutationScope<Schema, Binding>,
    >,
}

#[allow(clippy::too_many_arguments)]
pub fn prepare_selected_public_mutation<'selected, 'runtime, Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    installed: &TypedInstalledProducer<Schema, Binding>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    shared: &'selected SharedSelectedProductOperation<'runtime, Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    input: &MutationInput<Schema, Binding>,
) -> Result<
    PreparedSelectedPublicMutation<'selected, 'runtime, Schema, Binding>,
    SelectedPublicPreparationStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    MutationScope<Schema, Binding>: ApplicationEntityMarkerIdentity<Schema>,
{
    prepare_public_mutation::<Schema, Binding>(
        execution, installed, runtime, shared, external, request, input, None,
        None,
    )
}

pub fn prepare_recovery_public_mutation<'selected, 'runtime, Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    installed: &TypedInstalledProducer<Schema, Binding>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    shared: &'selected SharedSelectedProductOperation<'runtime, Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    input: &MutationInput<Schema, Binding>,
    recovery: &super::unpublished::ContinuationState,
) -> Result<
    PreparedSelectedPublicMutation<'selected, 'runtime, Schema, Binding>,
    SelectedPublicPreparationStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    MutationScope<Schema, Binding>: ApplicationEntityMarkerIdentity<Schema>,
{
    prepare_public_mutation::<Schema, Binding>(
        execution,
        installed,
        runtime,
        shared,
        external,
        request,
        input,
        Some(recovery),
        None,
    )
}

fn prepare_public_mutation<'selected, 'runtime, Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    installed: &TypedInstalledProducer<Schema, Binding>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    shared: &'selected SharedSelectedProductOperation<'runtime, Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    input: &MutationInput<Schema, Binding>,
    recovery: Option<&super::unpublished::ContinuationState>,
    applied: Option<(
        &worth_relational::facade::runtime::RelationalPinnedRead,
        &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    )>,
) -> Result<
    PreparedSelectedPublicMutation<'selected, 'runtime, Schema, Binding>,
    SelectedPublicPreparationStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    MutationScope<Schema, Binding>: ApplicationEntityMarkerIdentity<Schema>,
{
    use SelectedPublicPreparationStop as Stop;
    // The actual runtime index/schema, cold installed binding, and shared
    // selected Product are caller operands. The Installation owner admits its
    // selected catalog and authority work before issuing the borrowed proof.

    let selected = shared.selected();
    if !std::ptr::eq(selected.application(), runtime) {
        return Err(Stop::ForeignSelectedRuntime);
    }
    let current = runtime
        .runtime
        .installed_packages()
        .validate_retained_mutation_binding(runtime.installed_schema(), &installed.mutation)
        .map_err(|_| Stop::Issuer)?;
    // Three retained-binding/wrapper getters, the scope mode, the principal
    // identity reference, and the selected-scope call arguments are formed by
    // this owner before their admitted callees begin work.

    let principal = selected
        .resolve_authenticated_principal_issued(
            execution,
            current.principal_binding(),
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
            None,
        )
        .map_err(Stop::Principal)?;

    if current.scope_contract().mode() != ApplicationMutationScopeResolutionMode::InputField {
        // Principal-derived scope copies need their own admitted Declaration
        // owner. This selected route refuses before scope or graph effects.
        return Err(Stop::UnsupportedScopeMode);
    }
    // operation_input and scope_for_binding are authored producer callbacks. For
    // InputField, into_field_parts moves the already supplied field/value;
    // it performs no framework clone of the principal identity.
    let (field, value) =
        <_ as ApplicationMutationInputScope<Schema, Mutation<Schema, Binding>>>::scope_for_binding(
            input,
        )
        .into_field_parts(principal.principal().principal_identity());
    let scope = selected.resolve_mutation_scope_issued(execution, field, value, request);
    let scope = match scope {
        Ok(scope) => scope,
        Err(cause) if applied.is_some() && cause.kind() == crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(
            crate::domain_computation::primary_graph::WorthQueryEntityResolutionRefusalKind::UnknownEntity,
        ) => {
            require_selected_head::<Schema, Binding>(runtime, shared, recovery)?;
            let (before, candidate) = applied.expect("applied source posture");
            let (field, value) = <_ as ApplicationMutationInputScope<Schema, Mutation<Schema, Binding>>>::scope_for_binding(input)
                .into_field_parts(principal.principal().principal_identity());
            let scope = runtime.runtime.primary_graph().ok_or(Stop::ForeignSelectedRuntime)?.retain_entity_resolution_context()
                .resolve_applied_field(execution, before, candidate, None, field, value).map_err(Stop::Scope)?;
            let operation = crate::domain_computation::authorization::authorize_applied_control_mutation_on_selected::<Schema, Mutation<Schema, Binding>, MutationScope<Schema, Binding>>(
                execution, runtime, shared, &current, &installed.prepared_graph, &principal, &scope, before, candidate, request,
            ).map_err(Stop::Operation)?;
            return Ok(PreparedSelectedPublicMutation { principal, operation, scope: Binding::PREDECESSOR_READS.or(Binding::ORIGINAL_READS).map(|_| scope) });
        }
        Err(cause) => return Err(Stop::Scope(cause)),
    };
    require_selected_head::<Schema, Binding>(runtime, shared, recovery)?;
    let operation = authorize_public_mutation_on_selected::<
        Schema,
        Mutation<Schema, Binding>,
        MutationScope<Schema, Binding>,
    >(
        execution,
        runtime,
        shared,
        &current,
        &installed.prepared_graph,
        &principal,
        &scope,
        request,
    )
    .map_err(Stop::Operation)?;

    Ok(PreparedSelectedPublicMutation {
        principal,
        operation,
        scope: Binding::PREDECESSOR_READS
            .or(Binding::ORIGINAL_READS)
            .map(|_| {
                scope
                    .scope()
                    .retain_description()
                    .into_application_identity()
            }),
    })
}

fn require_selected_head<S, B>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
    shared: &SharedSelectedProductOperation<'_, S>,
    recovery: Option<&super::unpublished::ContinuationState>,
) -> Result<(), SelectedPublicPreparationStop>
where
    S: ApplicationSchema + 'static,
    B: WorthQueryApplicationProducerBinding<S>,
{
    use SelectedPublicPreparationStop as Stop;
    let source_matches = match recovery {
        Some(recovery) => recovery
            .admits_selected(shared.selected().product().observation())
            .map_err(|stop| Stop::Head(stop.into()))?,
        None => source_readmission::is_current_selected_product::<S, B>(runtime, shared.selected())
            .map_err(Stop::Head)?,
    };
    if !source_matches {
        return Err(Stop::MovedHead);
    }
    Ok(())
}

mod denial;
pub use denial::preparation_stop;
