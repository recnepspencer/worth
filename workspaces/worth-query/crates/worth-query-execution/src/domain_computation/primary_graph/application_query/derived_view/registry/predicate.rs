//! Managed Source posting axes and their World-qualified registration protocol.
mod axis;
mod preparation;

mod world;
pub(in crate::domain_computation::primary_graph) use axis::SourcePredicateAxis;
pub(in crate::domain_computation::primary_graph::application_query) use axis::{
    PredicateRegistration, PredicateRootReservation,
};
pub(in crate::domain_computation::primary_graph::application_query) use preparation::{
    mark_prepared_predicates_unavailable, route_prepared_predicates,
};
pub(in crate::domain_computation::primary_graph) use preparation::{
    PredicatePreparationFactory, PredicatePreparationOwner, PredicatePreparationRetirement,
    PreparedPredicateNotices, PreparedPredicatePublication,
};
pub(in crate::domain_computation::primary_graph) use world::PredicateWorld;

impl super::ManagedDerivedViewRegistry {
    /// Share the existing registry-wide issuer across every Source and provider.
    /// Failed openings consume their coordinate; it is never reused.
    pub(in crate::domain_computation::primary_graph::application_query) fn issue_predicate_slot(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::domain_computation::primary_graph::vocabulary::application_query::tree::WatchSlot, crate::domain_computation::primary_graph::application_query::managed_index::retention::WorthQueryManagedIndexLifecycleDenial>{
        use {crate::domain_computation::primary_graph::vocabulary::application_query::tree::WatchSlot, crate::domain_computation::primary_graph::application_query::managed_index::retention::WorthQueryManagedIndexLifecycleDenial as Denial};
        crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::work(
            execution, 1,
        )?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let id = state
            .next_id
            .checked_add(1)
            .ok_or(Denial::RegistrationIdentityExhausted)?;
        state.next_id = id;
        Ok(WatchSlot(id))
    }
    pub(in crate::domain_computation::primary_graph) fn has_predicate_routing(&self) -> bool {
        self.sources.has_predicate_routing()
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn reserve_predicate_root(
        &self, execution: &worth_execution::ExecutionRequest<'_, '_>,
        program: &worth_relational::facade::runtime::ManagedIndexSourceProgram,
        world: PredicateWorld, scope: worth_relational::facade::identity::EntityId,
        parent: &worth_query_installation::facade::WorthQueryInstalledManagedIndexParent,
    ) -> Result<PredicateRootReservation, crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial>{
        use crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop;
        self.sources
            .service(program)
            .and_then(|service| service.predicate_axis())
            .ok_or(Stop::MembershipUnavailable)?
            .reserve(execution, world, scope, parent)
    }
}
