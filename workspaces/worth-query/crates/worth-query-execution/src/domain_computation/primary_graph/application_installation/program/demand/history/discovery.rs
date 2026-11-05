//! Fresh C permission precedes every reuse or retention of discovery evidence.
use super::dependent::{DiscoveryBinding, QueryRows};
use super::*;
use crate::domain_computation::primary_graph::application_query::RetainedPermissionScopedOps;
use crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost;
use crate::domain_computation::primary_graph::{
    WorthQueryHistoryOutputDiscovery, WorthQueryHistoryOutputReadDenial,
};
type DiscoveryQuery<S, C> = <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::Query;

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn authorize_history_discovery<D, C>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQuerySettledHistoryProgramOutput<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        discovery: &WorthQueryHistoryOutputDiscovery<
            DiscoveryQuery<S, C>,
            QueryRows<S, DiscoveryBinding<S, C>>,
        >,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
    ) -> Result<(), WorthQueryHistoryOutputReadDenial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        use WorthQueryHistoryOutputReadDenial as Denial;
        let runtime = self.runtime();
        authority
            .validate(phase, runtime.application_runtime(), request)
            .map_err(Denial::Authority)?;
        if !discovery.profile.same_custody(authority.profile())
            || !discovery.profile.matches_program(self)
            || !std::sync::Arc::ptr_eq(&discovery.original, authority.original())
            || discovery.binding
                != crate::equality_type_id::EqualityTypeId::of::<DiscoveryBinding<S, C>>()
                    .into_external()
            || discovery.connection
                != crate::equality_type_id::EqualityTypeId::of::<C>().into_external()
            || !std::sync::Arc::ptr_eq(&discovery.parent, &parent.settled.retained)
            || !parent.profile.same_custody(&discovery.profile)
            || !std::sync::Arc::ptr_eq(&discovery.original, &parent.original)
            || !discovery.parent.belongs_to(runtime)
        {
            return Err(Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ForeignDocument));
        }
        self.validate_dependent_output_parent::<D, C>(&parent.settled)
            .map_err(Denial::Demand)?;
        let installed = runtime.installed_schema().installed_query_binding::<DiscoveryBinding<S, C>>()
            .map_err(|stop| Denial::Demand(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::SourceQueryInstallation(stop.kind()), "history discovery query")))?;
        if &discovery.query != installed.query().identity() {
            return Err(Denial::Demand(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::ForeignSource, "history discovery binding changed")));
        }
        // The set observation exists independently of row cardinality, so an
        // empty cached discovery cannot bypass its own current READ admission.
        let source = discovery.source.result_set_observation().ok_or_else(|| Denial::Demand(
            WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
                "history discovery has no original set observation"),
        ))?.source();
        let execution = phase.checked_execution_request_for(runtime.product_runtime())
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
        runtime
            .application_runtime()
            .authorize_retained_history_query::<DiscoveryBinding<S, C>>(
                &execution,
                authority.producer_invocation(),
                source,
                principal,
                request,
            )
            .map_err(Denial::from)
    }

    pub fn retain_history_discovery_dependencies<D, C>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQuerySettledHistoryProgramOutput<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        discovery: &WorthQueryHistoryOutputDiscovery<
            DiscoveryQuery<S, C>,
            QueryRows<S, DiscoveryBinding<S, C>>,
        >,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        crate::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies,
        WorthQueryHistoryOutputReadDenial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        use WorthQueryHistoryOutputReadDenial as Denial;
        self.authorize_history_discovery::<D, C>(
            phase, access, parent, authority, discovery, principal, request,
        )?;
        let selected = self.runtime().application_runtime().select_application_read_observation(phase, &discovery.basis)
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ProductAdmission(cause)))?;
        let retained = selected
            .retain_query_dependencies(phase, access, &discovery.source)
            .map_err(Denial::Dependencies)?;
        authority
            .confirm_read(
                phase,
                self.runtime().application_runtime(),
                request,
                retained,
            )
            .map_err(Denial::Authority)
    }
}
