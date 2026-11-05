//! Current C supplies query authority; the admitted original D supplies rows.
use super::*;
use crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost;
use crate::domain_computation::primary_graph::application_query::OutputReadScopedOps;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::{
    application_query::ApplicationQueryBinding,
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use {
    crate::domain_computation::primary_graph::WorthQueryApplicationOneShotResult,
    crate::domain_computation::primary_graph::WorthQueryApplicationProjection,
    crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial,
    crate::domain_computation::primary_graph::WorthQueryHistoryOutputAuthority,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial;

type Rows<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// Keep the original read, admission and authority causes distinct.
#[derive(Debug)]
pub enum WorthQueryHistoryOutputReadDenial {
    Authority(WorthQueryHistoryNavigationDenial),
    Admission(WorthQueryApplicationQueryAdmissionDenial),
    Read(crate::domain_computation::primary_graph::WorthQueryManagedIndexDenial),
    Control(WorthQueryHistoryOutputControlDenial),
    Demand(crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial),
    Principal(crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::vocabulary::entity_resolution_denial::WorthQueryEntityResolutionDenial),
    Resource(worth_relational::facade::mvcc::CompanionPreflightStop),
    Preparation(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Dependencies(
        crate::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial,
    ),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryOutputControlDenial {
    ForeignPlan,
    StaleInstalledQuery,
    StalePrincipal,
    Cancelled,
    DeadlineExceeded,
    ExpiredBasis,
    BasisUnavailable,
}

/// The original completed root read remains tied to its actual live callback.
/// Readonly rows/receipt cannot manufacture the linear claim source.
pub struct WorthQueryHistoryRootOutputRead<'authority, 'phase, 'scope, Q, R> {
    authority: &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
    result: WorthQueryApplicationOneShotResult<Q, R>,
}

impl<'authority, 'phase, 'scope, Q, R>
    WorthQueryHistoryRootOutputRead<'authority, 'phase, 'scope, Q, R>
{
    pub fn rows(&self) -> &[R] {
        self.result.rows()
    }

    pub fn receipt(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessReceipt {
        self.result.receipt()
    }

    pub(in crate::domain_computation::primary_graph) fn into_claim(
        self,
    ) -> (
        &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
        WorthQueryApplicationOneShotResult<Q, R>,
    ) {
        (self.authority, self.result)
    }
}

pub trait HistoryOutputsOps<S: ApplicationSchema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<S>
{
    fn execute_history_root_output_query<'control, 'authority, 'phase, 'scope, B>(
        &'control self,
        phase: &WorthQueryAdvancementPhase<'_>,
        authority: &'authority WorthQueryHistoryOutputAuthority<'phase, 'scope>,
        binding: &'control WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        control: crate::domain_computation::primary_graph::application_query::HistoryOutputControlPlan<'control, S, B>,
    ) -> Result<
        WorthQueryHistoryRootOutputRead<'authority, 'phase, 'scope, B::Query, Rows<S, B>>,
        WorthQueryHistoryOutputReadDenial,
    >
    where
        B: ApplicationQueryBinding<S>,
        Rows<S, B>: WorthQueryApplicationProjection<S, B::Query>,
    {
        use WorthQueryHistoryOutputReadDenial as Denial;
        self.application_runtime()
            .validate_history_output_control(phase, authority, binding, &control)?;
        let access =
            WorthQueryApplicationQueryAccessContext::new(control.principal(), control.scope());
        if !authority.matches_read_access(&access) {
            return Err(Denial::Authority(
                WorthQueryHistoryNavigationDenial::ForeignDocument,
            ));
        }
        let execution = phase
            .checked_execution_request_for(self.product_runtime())
            .map_err(|cause| {
                Denial::Authority(WorthQueryHistoryNavigationDenial::Stopped(cause))
            })?;
        authority.profile().source_intent().require_control(
            self.application_runtime().application_query_host(),
            &execution,
            binding,
            &control,
        )?;
        let retained = self
            .application_runtime()
            .select_application_read_observation(phase, authority.original())
            .map_err(|cause| {
                Denial::Admission(super::selection_denial(cause, binding.query().name()))
            })?;
        let result = self
            .application_runtime()
            .execute_history_output_query_on_selected(
                phase, authority, binding, external, control, retained,
            )?;
        Ok(WorthQueryHistoryRootOutputRead { authority, result })
    }
}

impl<
        S: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                S,
            > + ?Sized,
    > HistoryOutputsOps<S> for HostRuntime
{
}
