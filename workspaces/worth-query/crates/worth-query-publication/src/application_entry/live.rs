use worth_query_declaration::facade::application_query::{
    ApplicationLiveQueryIntent, ApplicationQueryBinding, ApplicationQueryScopeBinding,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveLease,
    WorthQueryApplicationLiveOutcome, WorthQueryApplicationProjection,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryApplicationRequest;

type Binding<Schema, Intent> =
    <Intent as worth_query_declaration::facade::application_query::ApplicationQueryIntent<
        Schema,
    >>::Binding;
type Query<Schema, Intent> = <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Query;
type Parameters<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
type QueryResult<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Principal<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::Principal;
type PrincipalIdentity<Schema, Intent> =
    <Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::PrincipalIdentity;
type Scope<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;

pub struct WorthQueryApplicationLiveLimits {
    pub(super) buffer_capacity: usize,
    pub(super) maximum_results: usize,
    pub(super) maximum_work: usize,
}

impl WorthQueryApplicationLiveLimits {
    pub const fn bounded(
        buffer_capacity: usize,
        maximum_results: usize,
        maximum_work: usize,
    ) -> Self {
        Self {
            buffer_capacity,
            maximum_results,
            maximum_work,
        }
    }
}

#[derive(Debug)]
pub enum WorthQueryApplicationLiveOpenRequestDenial {
    RetainedBasis,
    BindingInstallation(
        worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial,
    ),
    Limit(worth_query_installation::facade::WorthQueryApplicationQueryLimitDenial),
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    PrincipalResolution(
        worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionDenial,
    ),
    ScopeResolution(worth_query_execution::facade::primary_graph::WorthQueryEntityResolutionDenial),
    Controls(worth_query_execution::facade::primary_graph::WorthQueryApplicationLiveControlDenial),
    Open(worth_query_execution::facade::primary_graph::WorthQueryApplicationLiveOpenDenial),
}

#[derive(Debug)]
pub enum WorthQueryApplicationLiveNextDenial {
    ForeignApplication,
    ForeignBranch,
    BindingInstallation(
        worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial,
    ),
    ProductSelection(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    PrincipalResolution(
        worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionDenial,
    ),
}

pub struct WorthQueryApplicationLiveSubscription<'application, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationLiveQueryIntent<Schema>,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    lease: WorthQueryApplicationLiveLease<
        'application,
        Schema,
        Query<Schema, Intent>,
        Parameters<Schema, Intent>,
        QueryResult<Schema, Intent>,
        Principal<Schema, Intent>,
        PrincipalIdentity<Schema, Intent>,
        Scope<Schema, Intent>,
        Intent::Target,
        Intent::LiveCause,
    >,
}

impl<'application, Schema, Intent>
    WorthQueryApplicationLiveSubscription<'application, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationLiveQueryIntent<Schema>,
    QueryResult<Schema, Intent>: WorthQueryApplicationProjection<Schema, Query<Schema, Intent>>,
{
    pub(super) const fn new(
        application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        lease: WorthQueryApplicationLiveLease<
            'application,
            Schema,
            Query<Schema, Intent>,
            Parameters<Schema, Intent>,
            QueryResult<Schema, Intent>,
            Principal<Schema, Intent>,
            PrincipalIdentity<Schema, Intent>,
            Scope<Schema, Intent>,
            Intent::Target,
            Intent::LiveCause,
        >,
    ) -> Self {
        Self {
            application,
            branch,
            lease,
        }
    }

    pub fn next(
        &mut self,
        fresh_request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationLiveOutcome<Query<Schema, Intent>, QueryResult<Schema, Intent>>,
        WorthQueryApplicationLiveNextDenial,
    > {
        if !std::ptr::eq(self.application, fresh_request.application) {
            return Err(WorthQueryApplicationLiveNextDenial::ForeignApplication);
        }
        if self.branch != fresh_request.branch {
            return Err(WorthQueryApplicationLiveNextDenial::ForeignBranch);
        }
        let binding = self
            .application
            .installed_schema()
            .installed_query_binding::<Binding<Schema, Intent>>()
            .map_err(WorthQueryApplicationLiveNextDenial::BindingInstallation)?;
        let selected = self
            .application
            .on_branch(fresh_request.branch)
            .select()
            .map_err(WorthQueryApplicationLiveNextDenial::ProductSelection)?;
        let principal = selected
            .resolve_authenticated_principal(
                binding.principal_binding(),
                fresh_request.principal,
                fresh_request.scope,
                worth_query_execution::facade::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationLiveNextDenial::PrincipalResolution)?;
        Ok(self.lease.next(&principal, fresh_request.scope))
    }

    pub fn close(self) -> WorthQueryApplicationLiveCloseOutcome {
        self.lease.close()
    }
}
