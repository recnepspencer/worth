//! Exact selected conventional authority for one required producer execution.

use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
        ApplicationMutationScopeResolution, ApplicationMutationScopeResolutionMode,
    },
    application_schema::ApplicationEntityMarkerIdentity,
};
use worth_query_installation::facade::WorthQueryRetainedMutationBindingAdmissionStop;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::*;
use crate::domain_computation::authorization::{
    authorize_public_mutation_on_selected, SelectedConventionalAdmissionStop,
};
use crate::domain_computation::primary_graph::{
    SharedSelectedProductOperation, WorthQueryEntityResolutionDenial,
    WorthQueryIssuedSelectedPrincipal, WorthQueryIssuedSelectedScope,
    WorthQueryPrincipalResolutionDenial,
};

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
    Admission(CompanionPreflightStop),
    AccountingOverflow,
    ForeignSelectedRuntime,
    Issuer(WorthQueryRetainedMutationBindingAdmissionStop<CompanionPreflightStop>),
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
    pub(super) scope:
        WorthQueryIssuedSelectedScope<'selected, 'runtime, Schema, MutationScope<Schema, Binding>>,
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
    installed: &TypedInstalledProducer<Schema, Binding>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    shared: &'selected SharedSelectedProductOperation<'runtime, Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    input: &MutationInput<Schema, Binding>,
    admission: &mut InvalidationEditAdmission,
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
    admission.charge_external_work(4).map_err(Stop::Admission)?;
    let selected = shared.selected();
    if !std::ptr::eq(selected.application(), runtime) {
        return Err(Stop::ForeignSelectedRuntime);
    }
    let current = runtime
        .runtime
        .installed_packages()
        .validate_retained_mutation_binding_admitted(
            runtime.installed_schema(),
            &installed.mutation,
            |work, bytes| {
                admission.charge_external_work(work)?;
                admission.admit_read_scratch(bytes)
            },
        )
        .map_err(Stop::Issuer)?;
    // Three retained-binding/wrapper getters, the scope mode, the principal
    // identity reference, and the selected-scope call arguments are formed by
    // this owner before their admitted callees begin work.
    admission.charge_external_work(8).map_err(Stop::Admission)?;
    let principal = selected
        .resolve_authenticated_principal_issued(
            current.principal_binding(),
            external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
            None,
            admission,
        )
        .map_err(Stop::Principal)?;

    if current.scope_contract().mode() != ApplicationMutationScopeResolutionMode::InputField {
        // Principal-derived scope copies need their own admitted Declaration
        // owner. This selected route refuses before scope or graph effects.
        return Err(Stop::UnsupportedScopeMode);
    }
    // operation_input and scope_binding are authored producer callbacks. For
    // InputField, into_field_parts moves the already supplied field/value;
    // it performs no framework clone of the principal identity.
    let (field, value) = input
        .scope_binding()
        .into_field_parts(principal.principal().principal_identity());
    let scope = selected
        .resolve_mutation_scope_issued(field, value, request, admission)
        .map_err(Stop::Scope)?;
    if !source_readmission::is_current_selected_product::<Schema, Binding>(
        runtime, selected, admission,
    )
    .map_err(Stop::Head)?
    {
        return Err(Stop::MovedHead);
    }
    let operation = authorize_public_mutation_on_selected::<
        Schema,
        Mutation<Schema, Binding>,
        MutationScope<Schema, Binding>,
    >(
        runtime,
        shared,
        &current,
        &installed.prepared_graph,
        &principal,
        &scope,
        request,
        admission,
    )
    .map_err(Stop::Operation)?;
    let carrier_work = u64::try_from(std::mem::size_of::<
        PreparedSelectedPublicMutation<'selected, 'runtime, Schema, Binding>,
    >())
    .map_err(|_| Stop::AccountingOverflow)?;
    admission
        .charge_external_work(carrier_work)
        .map_err(Stop::Admission)?;
    Ok(PreparedSelectedPublicMutation {
        principal,
        scope,
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
    use crate::domain_computation::provider_session::WorthQueryAdmittedMutationSessionStartStop as SessionStop;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
    use worth_query_admission::integration::WorthQueryGraphWorkCapacityAdmissionStop as CapacityStop;
    use worth_query_installation::facade::WorthQueryRetainedMutationBindingAdmissionStop as IssuerStop;

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
        SelectedPublicPreparationStop::Admission(stop) => resource(stop),
        SelectedPublicPreparationStop::AccountingOverflow => work(),
        SelectedPublicPreparationStop::ForeignSelectedRuntime =>
            denial(WorthQueryOutputDemandDenialKind::ForeignSource, Binding::IDENTITY).into(),
        SelectedPublicPreparationStop::Issuer(IssuerStop::Admission(stop)) => resource(stop),
        SelectedPublicPreparationStop::Issuer(IssuerStop::AccountingOverflow) => work(),
        SelectedPublicPreparationStop::Issuer(IssuerStop::Installation(_)) =>
            denial(WorthQueryOutputDemandDenialKind::Superseded, Binding::IDENTITY).into(),
        SelectedPublicPreparationStop::Principal(denial) =>
            super::denial::principal_rejected(Binding::IDENTITY, denial),
        SelectedPublicPreparationStop::Scope(denial) =>
            super::denial::scope_rejected(Binding::IDENTITY, denial),
        SelectedPublicPreparationStop::UnsupportedScopeMode =>
            denial(WorthQueryOutputDemandDenialKind::ProducerUnavailable, Binding::IDENTITY).into(),
        SelectedPublicPreparationStop::Head(stop) => stop,
        SelectedPublicPreparationStop::MovedHead =>
            denial(WorthQueryOutputDemandDenialKind::Superseded, Binding::IDENTITY).into(),
        SelectedPublicPreparationStop::Operation(stop) => match stop {
            SelectedConventionalAdmissionStop::Admission(stop) => resource(stop),
            SelectedConventionalAdmissionStop::AccountingOverflow => work(),
            SelectedConventionalAdmissionStop::Interrupted(kind) => interrupted(kind),
            SelectedConventionalAdmissionStop::ExpiredAuthentication =>
                request(WorthQueryOutputDemandDenialKind::ProducerUnavailable),
            SelectedConventionalAdmissionStop::Preconditions(
                crate::domain_computation::primary_graph::EmptyMutationPreconditionAdmissionStop::Admission(stop)
            ) => resource(stop),
            SelectedConventionalAdmissionStop::Principal(
                crate::domain_computation::primary_graph::PrincipalCurrentnessCaptureStop::Admission(stop)
            ) => resource(stop),
            SelectedConventionalAdmissionStop::GraphWork(GraphStop::Source(stop)) => resource(stop),
            SelectedConventionalAdmissionStop::GraphWork(GraphStop::Interrupted(kind)) => interrupted(kind),
            SelectedConventionalAdmissionStop::GraphWork(GraphStop::Capacity(CapacityStop::Admission(stop))) => resource(stop),
            SelectedConventionalAdmissionStop::GraphWork(GraphStop::Session(SessionStop::Admission(stop))) => resource(stop),
            SelectedConventionalAdmissionStop::GraphWork(GraphStop::Session(SessionStop::WorkCounterOverflow)) => work(),
            _ => request(WorthQueryOutputDemandDenialKind::ProducerUnavailable),
        },
    }
}
