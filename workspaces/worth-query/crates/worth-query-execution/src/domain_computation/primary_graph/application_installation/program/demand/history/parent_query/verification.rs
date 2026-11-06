//! A verified dependent begins with real C source admission, without projection.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHistoryOutputProfile, WorthQueryVerifiedHistoryParent,
};

/// Only parent-query admission constructs this handoff to the sealed issuer.
pub(in crate::domain_computation::primary_graph) struct HistoryAdmittedChildSource<S, P, D> {
    marker: std::marker::PhantomData<fn() -> (S, P, D)>,
    profile: WorthQueryHistoryOutputProfile,
    original: std::sync::Arc<WorthQueryApplicationReadObservation>,
    basis: std::sync::Arc<WorthQueryApplicationReadObservation>,
    target: std::any::TypeId,
    artifact: Option<Artifact>,
    source: worth_relational::facade::identity::EntityId,
    partition: [u8; 32],
    family: &'static str,
}
impl<S, P, D> HistoryAdmittedChildSource<S, P, D> {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        WorthQueryHistoryOutputProfile,
        std::sync::Arc<WorthQueryApplicationReadObservation>,
        std::sync::Arc<WorthQueryApplicationReadObservation>,
        std::any::TypeId,
        Option<Artifact>,
        worth_relational::facade::identity::EntityId,
        [u8; 32],
        &'static str,
    ) {
        (
            self.profile,
            self.original,
            self.basis,
            self.target,
            self.artifact,
            self.source,
            self.partition,
            self.family,
        )
    }
}
macro_rules! verify_child_entry {
    ($method:ident, $parent:ident, $posture:ident) => {
        pub fn $method<D, C>(
            &self, phase: &WorthQueryAdvancementPhase<'_>,
            _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            parent: &$parent<S, P, D>,
            authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
            basis: &std::sync::Arc<WorthQueryApplicationReadObservation>,
            minimum: &WorthQueryApplicationReadObservation,
            external: &WorthQueryAuthenticatedExternalPrincipal<S>, request: &WorthQueryRequestScope,
            demand: &ConnectionDemand<S, C>,
        ) -> Result<Option<WorthQueryVerifiedHistoryParent<S, P, ConnectionDemand<S, C>>>, Denial>
        where
            D: WorthQueryApplicationOutputDemand<S>, C: ApplicationConnectionShape<S>,
            ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
            <Source<S, ConnectionDemand<S, C>> as ApplicationQueryBinding<S>>::Input:
                ApplicationQueryIntent<S, Binding = Source<S, ConnectionDemand<S, C>>>,
            <Source<S, ConnectionDemand<S, C>> as ApplicationQueryBinding<S>>::ScopeBinding:
                ApplicationQueryScopeResolution<S, <Source<S, ConnectionDemand<S, C>> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
            SourceValue<S, ConnectionDemand<S, C>>: WorthQueryApplicationProjection<S, SourceQuery<S, ConnectionDemand<S, C>>>,
        {
            self.verify_parent_child::<D, C>(phase, Parent::$posture(parent), authority, basis, minimum, external, request, demand)
        }
    };
}
impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    verify_child_entry!(
        verify_owned_history_dependent_output,
        WorthQuerySettledHistoryProgramOutput,
        Settled
    );
    verify_child_entry!(
        verify_verified_history_dependent_output,
        WorthQueryVerifiedHistoryParent,
        Verified
    );

    fn verify_parent_child<D, C>(
        &self, phase: &WorthQueryAdvancementPhase<'_>, parent: Parent<'_, S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        basis: &std::sync::Arc<WorthQueryApplicationReadObservation>, minimum: &WorthQueryApplicationReadObservation,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>, request: &WorthQueryRequestScope,
        demand: &ConnectionDemand<S, C>,
    ) -> Result<Option<WorthQueryVerifiedHistoryParent<S, P, ConnectionDemand<S, C>>>, Denial>
    where
        D: WorthQueryApplicationOutputDemand<S>, C: ApplicationConnectionShape<S>,
        ConnectionBinding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
        <Source<S, ConnectionDemand<S, C>> as ApplicationQueryBinding<S>>::Input:
            ApplicationQueryIntent<S, Binding = Source<S, ConnectionDemand<S, C>>>,
        <Source<S, ConnectionDemand<S, C>> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<S, <Source<S, ConnectionDemand<S, C>> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
        SourceValue<S, ConnectionDemand<S, C>>: WorthQueryApplicationProjection<S, SourceQuery<S, ConnectionDemand<S, C>>>,
{
        let (verified, _, _) = self.with_parent_query_admission::<D, C, Source<S, ConnectionDemand<S, C>>, _>(
            phase, &parent, ParentQueryPurpose::ChildSource, authority, basis, minimum, external, request, demand.source_intent(),
            |_, execution, plan, native, artifact| {
                let source = plan.scope().entity_id();
                let partition = *plan.parameter_binding_identity().bytes();
                let family = <Family<S, ConnectionDemand<S, C>> as WorthQueryProducerOutputFamily<S>>::IDENTITY;
                // The admitted plan grants C permission only. No source projection,
                // disclosure, posting, original root or settlement is constructed.
                let selected = self.runtime().select_application_read_observation_with_request(execution, basis)
                    .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::ProductAdmission(cause)))?;
                native.with_runtime_mut(self.runtime().runtime.primary_graph().ok_or_else(foreign)?, execution,
                    foreign, |native| {
                        let evidence = authority.selected_output_read(execution, native)
                            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::RestoredOutputRead(cause)))?;
                        let route = evidence.selected_output_route(execution, family, source, partition)
                            .map_err(|cause| Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::RestoredOutputRead(cause)))?;
                        let Some(route) = route else { return Ok(None); };
                        if !selected.selected_route_is_current(execution, native, route).map_err(Denial::Demand)? {
                            return Ok(None);
                        }
                        WorthQueryVerifiedHistoryParent::from_admitted_child(execution, HistoryAdmittedChildSource {
                            marker: std::marker::PhantomData,
                            profile: authority.profile().clone(), original: std::sync::Arc::clone(authority.original()),
                            basis: std::sync::Arc::clone(basis), target: crate::equality_type_id::EqualityTypeId::of::<C::TargetFeature>().into_external(),
                            artifact, source, partition, family,
                        }).map(Some)
                    })
            },
        )?;
        authority
            .confirm_read(phase, self.runtime(), request, verified)
            .map_err(Denial::Authority)
    }
}
