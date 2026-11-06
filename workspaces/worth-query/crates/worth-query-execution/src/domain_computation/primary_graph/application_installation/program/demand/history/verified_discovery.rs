//! Discovery over an exact verified D parent, with every grant taken at C.
use super::dependent::{DiscoveryBinding, QueryRows};
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::HistoryOutputControlPlan,
    product_operation::document_history::navigation::HistoryDependentParent,
    WorthQueryHistoryOutputDiscovery, WorthQueryHistoryOutputReadDenial as Denial,
    WorthQueryVerifiedHistoryParent,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

type Query<S, C> = <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::Query;
type Rows<S, C> = QueryRows<S, DiscoveryBinding<S, C>>;

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn execute_verified_history_discovery<'control, D, C>(
        &'control self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQueryVerifiedHistoryParent<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        binding: &'control WorthQueryInstalledApplicationQueryBinding<S, DiscoveryBinding<S, C>>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        control: HistoryOutputControlPlan<'control, S, DiscoveryBinding<S, C>>,
    ) -> Result<WorthQueryHistoryOutputDiscovery<Query<S, C>, Rows<S, C>>, Denial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        Rows<S, C>: WorthQueryApplicationProjection<S, Query<S, C>>,
    {
        let runtime = self.runtime();
        // Do not inspect the historical parent's identity before current READ.
        runtime.validate_history_output_control(phase, authority, binding, &control)?;
        self.validate_verified_history_parent::<D, C>(parent, authority)
            .map_err(Denial::Demand)?;
        let installed = runtime
            .installed_schema()
            .installed_query_binding::<DiscoveryBinding<S, C>>()
            .map_err(|cause| Denial::Demand(installation(cause.kind())))?;
        if binding.query().identity() != installed.query().identity() {
            return Err(foreign());
        }
        let basis = parent.custody().original();
        let selected = runtime.select_application_read_observation(phase, basis)
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ProductAdmission(cause)))?;
        let result = runtime.execute_history_output_query_on_selected(
            phase, authority, binding, external, control, selected,
        )?;
        authority
            .confirm_read(
                phase,
                runtime,
                authority.request(),
                WorthQueryHistoryOutputDiscovery {
                    source: result.into_admitted_disclosed().into_output_demand_source(),
                    profile: authority.profile().clone(),
                    original: std::sync::Arc::clone(authority.original()),
                    parent: HistoryDependentParent::Verified(std::sync::Arc::clone(
                        parent.custody(),
                    )),
                    connection: crate::equality_type_id::EqualityTypeId::of::<C>().into_external(),
                    binding: crate::equality_type_id::EqualityTypeId::of::<DiscoveryBinding<S, C>>(
                    )
                    .into_external(),
                    query: binding.query().identity().clone(),
                    basis: std::sync::Arc::clone(basis),
                },
            )
            .map_err(Denial::Authority)
    }

    pub fn authorize_verified_history_discovery<D, C>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQueryVerifiedHistoryParent<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        discovery: &WorthQueryHistoryOutputDiscovery<Query<S, C>, Rows<S, C>>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
    ) -> Result<(), Denial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        let runtime = self.runtime();
        authority
            .validate(phase, runtime, request)
            .map_err(Denial::Authority)?;
        // Reauthorize the actual query subject first, including empty sets.
        let source = discovery
            .source
            .result_set_observation()
            .ok_or_else(foreign)?
            .source();
        let execution = phase.checked_execution_request_for(runtime.product_runtime())
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::Stopped(cause)))?;
        runtime
            .authorize_retained_history_query::<DiscoveryBinding<S, C>>(
                &execution,
                authority.producer_invocation(),
                source,
                principal,
                request,
            )
            .map_err(Denial::from)?;
        self.validate_verified_history_parent::<D, C>(parent, authority)
            .map_err(Denial::Demand)?;
        let installed = runtime
            .installed_schema()
            .installed_query_binding::<DiscoveryBinding<S, C>>()
            .map_err(|cause| Denial::Demand(installation(cause.kind())))?;
        if !discovery
            .parent
            .same_custody(&HistoryDependentParent::Verified(std::sync::Arc::clone(
                parent.custody(),
            )))
            || !discovery.profile.same_custody(authority.profile())
            || !std::sync::Arc::ptr_eq(&discovery.original, authority.original())
            || !std::sync::Arc::ptr_eq(&discovery.basis, parent.custody().original())
            || discovery.connection
                != crate::equality_type_id::EqualityTypeId::of::<C>().into_external()
            || discovery.binding
                != crate::equality_type_id::EqualityTypeId::of::<DiscoveryBinding<S, C>>()
                    .into_external()
            || &discovery.query != installed.query().identity()
        {
            return Err(foreign());
        }
        Ok(())
    }

    pub fn retain_verified_history_discovery_dependencies<D, C>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQueryVerifiedHistoryParent<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        discovery: &WorthQueryHistoryOutputDiscovery<Query<S, C>, Rows<S, C>>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
    ) -> Result<
        crate::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies,
        Denial,
    >
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        let runtime = self.runtime();
        self.authorize_verified_history_discovery::<D, C>(
            phase, access, parent, authority, discovery, principal, request,
        )?;
        let selected = runtime.select_application_read_observation(phase, &discovery.basis)
            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ProductAdmission(cause)))?;
        let retained = selected
            .retain_query_dependencies(phase, access, &discovery.source)
            .map_err(Denial::Dependencies)?;
        authority
            .confirm_read(phase, runtime, request, retained)
            .map_err(Denial::Authority)
    }

    pub(super) fn validate_verified_history_parent<D, C>(
        &self,
        parent: &WorthQueryVerifiedHistoryParent<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
    ) -> Result<Option<Artifact>, WorthQueryOutputDemandDenial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    {
        let custody = parent.custody();
        if !custody.profile().same_custody(authority.profile())
            || !custody.profile().matches_program(self)
            || !std::sync::Arc::ptr_eq(custody.original(), authority.original())
            || custody.original().branch_incarnation() != authority.original().branch_incarnation()
        {
            return Err(WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource, format!("verified history parent {} source {:?}", parent.family(), parent.source())));
        }
        self.validate_dependent_parent_meaning::<D, C>(
            crate::equality_type_id::EqualityTypeId::from(custody.target_feature()),
            custody.artifact(),
        )
        .map_err(|cause| cause.for_verified(parent.source(), parent.family()))
    }
}
fn foreign() -> Denial {
    Denial::Demand(WorthQueryOutputDemandDenial::new(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource,
        "verified history parent or installed discovery connection",
    ))
}
fn installation(
    kind: worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::SourceQueryInstallation(kind), "verified history discovery query")
}
