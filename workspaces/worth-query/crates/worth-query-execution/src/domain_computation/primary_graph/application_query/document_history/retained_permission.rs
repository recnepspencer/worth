//! A cache-entry permission check reads only current C, never output truth E.
use super::*;
use crate::domain_computation::primary_graph::application_query::FreshScopedOps;
use crate::domain_computation::primary_graph::application_query::WorkLimitOps;
use crate::domain_computation::primary_graph::product_operation::SharedQueryBasisScopedOps;
use crate::domain_computation::primary_graph::{
    application_query::{FreshQueryPermissionStop, WorthQueryApplicationQueryControls},
    product_operation::{
        document_history::HistoryProducerInvocation, SelectedQueryBasisRetentionStop,
    },
    WorthQueryHistoryNavigationDenial, WorthQueryObservedSource,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub(in crate::domain_computation::primary_graph) enum HistoryRetainedPermissionStop {
    Current(WorthQueryHistoryNavigationDenial),
    Resource(worth_relational::facade::mvcc::CompanionPreflightStop),
    Installation(
        worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind,
    ),
    Permission(FreshQueryPermissionStop<WorthQueryHistoryNavigationDenial>),
}

impl From<HistoryRetainedPermissionStop>
    for crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial
{
    fn from(stop: HistoryRetainedPermissionStop) -> Self {
        use HistoryRetainedPermissionStop as Stop;
        match stop {
            Stop::Current(stop) => Self::Authority(stop),
            Stop::Resource(stop) => Self::Resource(stop),
            Stop::Installation(kind) => Self::Demand(crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::SourceQueryInstallation(kind), "history source query")),
            Stop::Permission(stop) => match stop {
                FreshQueryPermissionStop::Principal(stop) => Self::Principal(stop),
                FreshQueryPermissionStop::Scope(stop) => Self::Scope(stop),
                FreshQueryPermissionStop::Query(stop) => Self::Admission(stop),
                FreshQueryPermissionStop::Inspect(stop) => Self::Authority(stop),
            },
        }
    }
}

pub(in crate::domain_computation::primary_graph) trait RetainedPermissionScopedOps<
    S: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>
{
    fn authorize_retained_history_query<B>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: HistoryProducerInvocation<'_>,
        observed: &WorthQueryObservedSource<B::Query>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
    ) -> Result<(), HistoryRetainedPermissionStop>
    where
        B: ApplicationQueryBinding<S>,
    {
        use HistoryRetainedPermissionStop as Stop;
        if !history.admits_request(request) {
            return Err(Stop::Current(
                WorthQueryHistoryNavigationDenial::ForeignDocument,
            ));
        }
        history
            .confirm_in_execution(execution, self.product_operation_host(), ())
            .map_err(Stop::Current)?;
        let installed = self
            .installed_schema()
            .installed_query_binding::<B>()
            .map_err(|cause| Stop::Installation(cause.kind()))?;
        let limits = self.resolve_application_query_limits(installed.limits());
        let selected = self
            .application_runtime()
            .on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                history.current().observation().lifecycle_incarnation(),
            ))
            .select_with_request(execution)
            .map_err(|cause| {
                Stop::Current(WorthQueryHistoryNavigationDenial::ProductAdmission(cause))
            })?;
        if selected.product().observation() != history.current().observation() {
            return Err(Stop::Current(
                WorthQueryHistoryNavigationDenial::HeadChanged,
            ));
        }
        let shared = selected
            .prepare_shared_query_basis()
            .map_err(|(_, cause)| Stop::Resource(cause))?;
        let (product, basis) =
            self.retain_selected_query_basis_admitted(&shared)
                .map_err(|cause| match cause {
                    SelectedQueryBasisRetentionStop::Basis => {
                        Stop::Current(WorthQueryHistoryNavigationDenial::HeadChanged)
                    }
                    SelectedQueryBasisRetentionStop::Admission(cause) => Stop::Resource(cause),
                })?;
        let controls = WorthQueryApplicationQueryControls::selected_read_one_shot(
            product,
            basis,
            limits.maximum_results(),
            request,
        )
        .map_err(Stop::Resource)?;
        self.with_fresh_retained_query_permission(
            execution,
            &installed,
            None,
            external,
            observed,
            shared.selected(),
            None,
            controls,
            |permission| {
                permission
                    .require_history_principal(history)
                    .map_err(FreshQueryPermissionStop::Query)?;
                permission
                    .require_original_scope(observed)
                    .map_err(FreshQueryPermissionStop::Query)?;
                permission
                    .validate_current_request()
                    .map_err(FreshQueryPermissionStop::Query)?;
                history
                    .confirm_in_execution(execution, self.product_operation_host(), ())
                    .map_err(FreshQueryPermissionStop::Inspect)?;
                permission
                    .validate_current_request()
                    .map_err(FreshQueryPermissionStop::Query)
            },
        )
        .map_err(|stop| Stop::Permission(flatten_permission(stop)))
    }
}

impl<
        S: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                S,
            > + ?Sized,
    > RetainedPermissionScopedOps<S> for HostRuntime
{
}

fn flatten_permission(
    stop: FreshQueryPermissionStop<FreshQueryPermissionStop<WorthQueryHistoryNavigationDenial>>,
) -> FreshQueryPermissionStop<WorthQueryHistoryNavigationDenial> {
    match stop {
        FreshQueryPermissionStop::Principal(stop) => FreshQueryPermissionStop::Principal(stop),
        FreshQueryPermissionStop::Scope(stop) => FreshQueryPermissionStop::Scope(stop),
        FreshQueryPermissionStop::Query(stop) => FreshQueryPermissionStop::Query(stop),
        FreshQueryPermissionStop::Inspect(stop) => stop,
    }
}
