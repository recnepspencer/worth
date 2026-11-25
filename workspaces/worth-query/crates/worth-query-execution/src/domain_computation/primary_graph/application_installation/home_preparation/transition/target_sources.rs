//! B's compiled managed-index Sources serve its positioned steps through A's
//! active publication companion, whose cells and generation stay its own.
use super::*;
use crate::domain_computation::primary_graph::application_query::derived_view::{
    registry::TransitionTargetSourcesGuard, ManagedDerivedViewRegistry,
};

/// Held across the positioned steps; dropping it ends B's Source lookup on A.
pub struct HomeProgramTransitionSourceAttachment {
    _guard: Option<TransitionTargetSourcesGuard>,
}

impl<S: ApplicationSchemaComposition, P> PreparedHomeProgramTransitionTarget<S, P> {
    /// Without this, Native's preparation of B's declared Sources at the target
    /// step finds only A's registry and refuses with a missing participant.
    pub fn attach_home_transition_sources<A: ApplicationSchemaComposition, AP>(
        &self,
        predecessor: &PreparedExistingHomeSignalProgram<A, AP>,
    ) -> Result<HomeProgramTransitionSourceAttachment, WorthQueryPrimaryGraphInstallationDenial>
    {
        if self._pending._managed_sources.is_empty() {
            return Ok(HomeProgramTransitionSourceAttachment { _guard: None });
        }
        let registry = predecessor
            .predecessor_managed_views()
            .into_any_arc()
            .downcast::<ManagedDerivedViewRegistry>()
            .map_err(|_| author_source::denied("A carries no managed-view registry"))?;
        registry
            .attach_transition_target_sources(
                Arc::clone(&self._pending._managed_sources),
                self.contracts.catalog_receipt(),
            )
            .map(|guard| HomeProgramTransitionSourceAttachment {
                _guard: Some(guard),
            })
            .ok_or_else(|| author_source::denied("A's managed Sources already serve a transition"))
    }
}
