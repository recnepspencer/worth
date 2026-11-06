//! Discovery has an explicit purpose even when it shares the child's query binding.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputDiscovery;
type Query<S, C> = <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::Query;
type Rows<S, C> = QueryRows<S, DiscoveryBinding<S, C>>;
impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn execute_verified_history_discovery<D, C>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQueryVerifiedHistoryParent<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        intent: Option<<ConnectionBinding<S, C> as WorthQueryApplicationDependentOutputConnection<S>>::Discovery>,
    ) -> Result<WorthQueryHistoryOutputDiscovery<Query<S, C>, Rows<S, C>>, Denial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
        Rows<S, C>: WorthQueryApplicationProjection<S, Query<S, C>>,
    {
        let basis = parent.observation();
        self.execute_parent_query::<D, C, DiscoveryBinding<S, C>>(
            phase,
            Parent::Verified(parent),
            ParentQueryPurpose::Discovery,
            authority,
            basis,
            basis,
            external,
            request,
            intent,
        )?
        .into_discovery()
    }

    pub fn execute_owned_history_discovery<D, C>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQuerySettledHistoryProgramOutput<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        basis: &std::sync::Arc<WorthQueryApplicationReadObservation>,
        minimum: &WorthQueryApplicationReadObservation,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        intent: Option<<ConnectionBinding<S, C> as WorthQueryApplicationDependentOutputConnection<S>>::Discovery>,
    ) -> Result<WorthQueryHistoryOutputDiscovery<Query<S, C>, Rows<S, C>>, Denial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <DiscoveryBinding<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
        Rows<S, C>: WorthQueryApplicationProjection<S, Query<S, C>>,
    {
        self.execute_parent_query::<D, C, DiscoveryBinding<S, C>>(
            phase,
            Parent::Settled(parent),
            ParentQueryPurpose::Discovery,
            authority,
            basis,
            minimum,
            external,
            request,
            intent,
        )?
        .into_discovery()
    }
}
