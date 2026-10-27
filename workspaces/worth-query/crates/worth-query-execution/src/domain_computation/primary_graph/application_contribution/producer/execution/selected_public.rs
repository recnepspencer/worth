//! Exact selected conventional authority for one required producer execution.

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationInputScope;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolution;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolutionMode;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;

use super::*;
use crate::domain_computation::authorization::authorize_public_mutation_on_selected;
use crate::domain_computation::authorization::SelectedConventionalAdmissionStop;
use crate::domain_computation::primary_graph::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial;
use crate::domain_computation::primary_graph::WorthQueryIssuedSelectedPrincipal;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial;

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
    let scope = selected
        .resolve_mutation_scope_issued(execution, field, value, request)
        .map_err(Stop::Scope)?;
    if !source_readmission::is_current_selected_product::<Schema, Binding>(runtime, selected)
        .map_err(Stop::Head)?
    {
        return Err(Stop::MovedHead);
    }
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
    })
}

pub(super) fn preparation_stop<Schema, Binding>(
    stop: SelectedPublicPreparationStop,
) -> ProducerExecutionStop
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    use crate::domain_computation::authorization::SelectedOperationGraphWorkStop as GraphStop;

    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;

    let resource = |stop| source_readmission::ready_resource_denial("", stop);
    let work = || denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "").into();
    let request = |kind| super::denial::request_admission_rejected(denial(kind, Binding::IDENTITY));
    let interrupted = |kind| match kind {
        WorthQueryRequestInterruption::Cancelled => {
            request(WorthQueryOutputDemandDenialKind::Cancelled)
        }
        WorthQueryRequestInterruption::DeadlineExceeded => {
            request(WorthQueryOutputDemandDenialKind::TimedOut)
        }
    };
    match stop {
        SelectedPublicPreparationStop::ForeignSelectedRuntime => denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            Binding::IDENTITY,
        )
        .into(),

        SelectedPublicPreparationStop::Issuer => denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Binding::IDENTITY,
        )
        .into(),
        SelectedPublicPreparationStop::Principal(denial) => {
            super::denial::principal_rejected(Binding::IDENTITY, denial)
        }
        SelectedPublicPreparationStop::Scope(denial) => {
            super::denial::scope_rejected(Binding::IDENTITY, denial)
        }
        SelectedPublicPreparationStop::UnsupportedScopeMode => denial(
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
            Binding::IDENTITY,
        )
        .into(),
        SelectedPublicPreparationStop::Head(stop) => stop,
        SelectedPublicPreparationStop::MovedHead => denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Binding::IDENTITY,
        )
        .into(),
        SelectedPublicPreparationStop::Operation(stop) => match stop {
            SelectedConventionalAdmissionStop::AccountingOverflow => work(),
            SelectedConventionalAdmissionStop::Authorization(denial) => {
                super::denial::request_authority_denied(Binding::IDENTITY, denial)
            }
            SelectedConventionalAdmissionStop::Interrupted(kind) => interrupted(kind),
            SelectedConventionalAdmissionStop::ExpiredAuthentication => {
                request(WorthQueryOutputDemandDenialKind::ProducerUnavailable)
            }

            SelectedConventionalAdmissionStop::GraphWork(GraphStop::Source(stop)) => resource(stop),
            SelectedConventionalAdmissionStop::GraphWork(GraphStop::Interrupted(kind)) => {
                interrupted(kind)
            }

            _ => request(WorthQueryOutputDemandDenialKind::ProducerUnavailable),
        },
    }
}
