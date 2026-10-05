use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryScopeBinding,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

use super::{
    denial, failed, principal_rejected, query_admission_denied, query_execution_denied,
    scope_rejected, ProducerExecutionStop, SourceBinding, SourceQuery, SourceValue,
    WorthQueryApplicationProducerBinding, WorthQueryOutputDemandDenialKind,
};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits;
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::{InvalidationEditAdmission, SettlementRegistrationStop},
    AcceptedCurrentCandidate, CurrentAcceptedResult, CurrentAcceptedStop,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationProjection,
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryControls,
    WorthQueryAuthenticatedPrincipal, WorthQueryObservedSource, WorthQueryOutputDemandDenial,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
    WorthQuerySelectedProductOperation,
};

mod current_product;
mod ready;
mod ready_on_selected;
mod selected_fresh;
pub(super) use current_product::is_current_selected_product;
pub(super) use ready::ready_resource_denial;
pub(super) use ready_on_selected::certify_ready_on_selected_with_disclosure;
pub(super) use selected_fresh::disclose_prepared_on_selected;

type BoundQuery<Schema, Binding> = SourceBinding<Schema, Binding>;
type BoundPrincipal<Schema, Binding> =
    <BoundQuery<Schema, Binding> as ApplicationQueryBinding<Schema>>::Principal;
type BoundPrincipalIdentity<Schema, Binding> =
    <BoundQuery<Schema, Binding> as ApplicationQueryBinding<Schema>>::PrincipalIdentity;
type BoundScope<Schema, Binding> = <<BoundQuery<Schema, Binding> as ApplicationQueryBinding<
    Schema,
>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;
/// Source query binding, principal, and scope a retained source re-authorizes.
type RetainedSourceAuthority<Schema, Binding> = (
    WorthQueryInstalledApplicationQueryBinding<Schema, SourceBinding<Schema, Binding>>,
    WorthQueryAuthenticatedPrincipal<
        Schema,
        BoundPrincipal<Schema, Binding>,
        BoundPrincipalIdentity<Schema, Binding>,
    >,
    WorthQueryApplicationEntityIdentity<Schema, BoundScope<Schema, Binding>>,
);

/// Fresh request authority for the original descriptive source selector. It
/// retains the selected Product and governed identities until the caller
/// either proves a current accepted output or prepares a new query read.
pub(super) struct AuthorizedRetainedSource<'runtime, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    installed: WorthQueryInstalledApplicationQueryBinding<Schema, BoundQuery<Schema, Binding>>,
    selected: WorthQuerySelectedProductOperation<'runtime, Schema>,
    principal: WorthQueryAuthenticatedPrincipal<
        Schema,
        BoundPrincipal<Schema, Binding>,
        BoundPrincipalIdentity<Schema, Binding>,
    >,
    scope: WorthQueryApplicationEntityIdentity<Schema, BoundScope<Schema, Binding>>,
}

pub(super) fn authorize_retained_source<'runtime, Schema, Binding>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    retained: &WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
    admission: &mut InvalidationEditAdmission,
) -> Result<AuthorizedRetainedSource<'runtime, Schema, Binding>, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let selected = runtime
        .on_branch(branch)
        .select()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let (installed, principal, scope) = authorize_retained_source_on_selected::<Schema, Binding>(
        runtime,
        external_principal,
        request,
        retained,
        &selected,
        admission,
    )?;
    Ok(AuthorizedRetainedSource {
        installed,
        selected,
        principal,
        scope,
    })
}

/// Fresh source-query identity, principal, and scope under an already selected
/// Product occurrence. A required-work wave may borrow this same selection for
/// each exact producer without choosing another Product or Query authority.
pub(super) fn authorize_retained_source_on_selected<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    retained: &WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    admission: &mut InvalidationEditAdmission,
) -> Result<RetainedSourceAuthority<Schema, Binding>, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let installed = runtime
        .installed_schema()
        .installed_query_binding::<SourceBinding<Schema, Binding>>()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let query = installed.query();
    if retained.runtime_authority != runtime.runtime.authority_identity().as_u64()
        || retained.schema_binding != *query.binding_identity()
        || retained.query_identity != *query.identity()
    {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Binding::IDENTITY,
        )
        .into());
    }
    let principal = selected
        .resolve_authenticated_principal_admitted(
            installed.principal_binding(),
            external_principal,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
            admission,
        )
        .map_err(|error| principal_rejected(Binding::IDENTITY, error))?;
    let scope = selected
        .resolve_retained_query_scope(
            SourceBinding::<Schema, Binding>::scope_field(),
            retained.retained_scope_selector(),
            request,
            admission,
        )
        .map_err(|error| scope_rejected(Binding::IDENTITY, error))?;
    Ok((installed, principal, scope))
}

pub(super) fn readmit_source<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    retained: &WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
    limits: WorthQueryOutputDemandLimits,
    admission: &mut InvalidationEditAdmission,
    edition: super::super::InstalledProducerEdition,
) -> Result<
    super::super::demand::disclosure::FreshOutputDisclosure<
        SourceQuery<Schema, Binding>,
        SourceValue<Schema, Binding>,
    >,
    ProducerExecutionStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Binding>>,
{
    let authorized = authorize_retained_source::<Schema, Binding>(
        runtime,
        external_principal,
        request,
        branch,
        retained,
        admission,
    )?;
    readmit_authorized_source::<Schema, Binding>(
        runtime,
        external_principal,
        request,
        branch,
        retained,
        limits,
        admission,
        edition,
        authorized,
    )
}

/// Continue the same selected principal, scope, and Product admission after a
/// current-output probe requires a real source disclosure. The owned selection
/// stays live through the query plan; no second branch or identity is chosen.
pub(super) fn readmit_authorized_source<'runtime, Schema, Binding>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    retained: &WorthQueryObservedSource<SourceQuery<Schema, Binding>>,
    limits: WorthQueryOutputDemandLimits,
    admission: &mut InvalidationEditAdmission,
    edition: super::super::InstalledProducerEdition,
    authorized: AuthorizedRetainedSource<'runtime, Schema, Binding>,
) -> Result<
    super::super::demand::disclosure::FreshOutputDisclosure<
        SourceQuery<Schema, Binding>,
        SourceValue<Schema, Binding>,
    >,
    ProducerExecutionStop,
>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Binding>>,
{
    let AuthorizedRetainedSource {
        installed,
        selected,
        principal,
        scope,
    } = authorized;
    let query = installed.query();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let installed_limits = runtime.resolve_application_query_limits(installed.limits());
    let maximum_results = installed_limits.maximum_results();
    let query_ceiling = installed_limits
        .maximum_work()
        .get()
        .min(limits.producer_work())
        .min(admission.remaining_work());
    let maximum_work = NonZeroUsize::new(query_ceiling).ok_or_else(|| {
        denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            Binding::IDENTITY,
        )
    })?;
    let (_, product, application_basis) = selected.into_parts();
    let controls = WorthQueryApplicationQueryControls::product_one_shot(
        product,
        application_basis,
        maximum_results,
        maximum_work,
        request,
    );
    let plan = runtime
        .readmit_application_query_from_observed(query, &access, retained, controls, admission)
        .map_err(|error| query_admission_denied(Binding::IDENTITY, error))?;
    let read = runtime
        .execute_application_query_one_shot(plan)
        .map_err(|error| query_execution_denied(Binding::IDENTITY, error))?;
    admission
        .charge_external_work(read.receipt().total_work_units() as u64)
        .map_err(|_| {
            denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                Binding::IDENTITY,
            )
        })?;

    super::super::demand::disclosure::validate_readmitted(
        runtime,
        retained,
        external_principal,
        request,
        branch,
        read.into_admitted_disclosed().into_output_demand_source(),
        edition,
        admission,
        Binding::IDENTITY,
    )
    .map_err(ProducerExecutionStop::ExecutionStopped)
}
