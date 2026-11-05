//! Exact selected conventional authority for one required producer execution.

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationInputScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolution;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolutionMode;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;

use super::*;
use crate::domain_computation::authorization::authorize_current_control_mutation_on_selected;
use crate::domain_computation::authorization::authorize_public_mutation_on_selected;
use crate::domain_computation::authorization::SelectedConventionalAdmissionStop;
use crate::domain_computation::primary_graph::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::vocabulary::entity_resolution_denial::WorthQueryEntityResolutionDenial;
use crate::domain_computation::primary_graph::WorthQueryIssuedSelectedPrincipal;
use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial;

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

pub(super) enum SelectedPublicPreparationStop {
    CurrentPrincipal(WorthQueryPrincipalResolutionDenial),
    CurrentScope(WorthQueryEntityResolutionDenial),
    CurrentHead(ProducerExecutionStop),
    CurrentMovedHead,
    CurrentOperation(SelectedConventionalAdmissionStop),
    ForeignSelectedRuntime,
    Issuer,
    Principal(WorthQueryPrincipalResolutionDenial),
    Scope(WorthQueryEntityResolutionDenial),
    UnsupportedScopeMode,
    Head(ProducerExecutionStop),
    MovedHead,
    Operation(SelectedConventionalAdmissionStop),
}

pub(super) struct PreparedSelectedPublicMutation<'selected, 'runtime, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    MutationScope<Schema, Binding>: ApplicationEntityMarkerIdentity<Schema>,
{
    pub(super) principal: WorthQueryIssuedSelectedPrincipal<
        'selected,
        'runtime,
        Schema,
        MutationPrincipal<Schema, Binding>,
        MutationPrincipalIdentity<Schema, Binding>,
    >,
    pub(super) operation:
        crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            <Mutation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Operation,
            MutationInput<Schema, Binding>,
            MutationScope<Schema, Binding>,
        >,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare_selected_public_mutation<'selected, 'runtime, Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    installed: &TypedInstalledProducer<Schema, Binding>,
    runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
    shared: &'selected SharedSelectedProductOperation<'runtime, Schema>,
    control: Option<&'selected SharedSelectedProductOperation<'runtime, Schema>>,
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
        execution, installed, runtime, shared, control, external, request, input, None,
    )
}

pub(super) fn prepare_recovery_public_mutation<'selected, 'runtime, Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    installed: &TypedInstalledProducer<Schema, Binding>,
    runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
    shared: &'selected SharedSelectedProductOperation<'runtime, Schema>,
    control: Option<&'selected SharedSelectedProductOperation<'runtime, Schema>>,
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
        control,
        external,
        request,
        input,
        Some(recovery),
    )
}

fn prepare_public_mutation<'selected, 'runtime, Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    installed: &TypedInstalledProducer<Schema, Binding>,
    runtime: &dyn crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema>,
    shared: &'selected SharedSelectedProductOperation<'runtime, Schema>,
    control: Option<&'selected SharedSelectedProductOperation<'runtime, Schema>>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    input: &MutationInput<Schema, Binding>,
    recovery: Option<&super::unpublished::ContinuationState>,
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

    let selected = control.unwrap_or(shared).selected();
    if !std::ptr::eq(selected.application(), runtime) {
        return Err(Stop::ForeignSelectedRuntime);
    }
    let current = runtime
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
        .map_err(|cause| {
            if control.is_some() {
                Stop::CurrentPrincipal(cause)
            } else {
                Stop::Principal(cause)
            }
        })?;

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
    let scope = selected
        .resolve_mutation_scope_issued(execution, field, value, request)
        .map_err(|cause| {
            if control.is_some() {
                Stop::CurrentScope(cause)
            } else {
                Stop::Scope(cause)
            }
        })?;
    let source_matches = match recovery {
        Some(recovery) => recovery
            .admits_selected(shared.selected().product().observation())
            .map_err(|stop| Stop::Head(stop.into()))?,
        None => source_readmission::is_current_selected_product::<Schema, Binding>(
            runtime,
            shared.selected(),
        )
        .map_err(Stop::Head)?,
    };
    if !source_matches {
        return Err(Stop::MovedHead);
    }
    let operation = if let Some(control) = control {
        if !source_readmission::is_current_selected_product::<Schema, Binding>(runtime, selected)
            .map_err(Stop::CurrentHead)?
        {
            return Err(Stop::CurrentMovedHead);
        }
        let selector = scope.scope().retain_scope_selector_funded(execution).map_err(|cause| {
            Stop::CurrentOperation(SelectedConventionalAdmissionStop::Authorization(
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(
                    crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Stopped(cause),
                    Binding::IDENTITY,
                ),
            ))
        })?;
        let mutation_scope = shared
            .selected()
            .resolve_retained_query_scope_issued(execution, field, &selector, request)
            .map_err(Stop::Scope)?;
        authorize_current_control_mutation_on_selected::<
            Schema,
            Mutation<Schema, Binding>,
            MutationScope<Schema, Binding>,
        >(
            execution,
            runtime,
            shared,
            control,
            &current,
            &installed.prepared_graph,
            &principal,
            &scope,
            &mutation_scope,
            request,
        )
        .map_err(Stop::CurrentOperation)
    } else {
        authorize_public_mutation_on_selected::<
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
        .map_err(Stop::Operation)
    }?;

    Ok(PreparedSelectedPublicMutation {
        principal,
        operation,
    })
}

mod denial;
pub(super) use denial::preparation_stop;
use crate::domain_computation::vocabulary::primary_graph::resolution::WorthQueryPrincipalResolutionMode;
