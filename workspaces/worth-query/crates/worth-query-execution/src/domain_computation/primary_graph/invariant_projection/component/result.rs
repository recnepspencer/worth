//! Typed membership iterates the original funded Native body without a copy.
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_identity::WorthQueryAggregateIdentity, aggregate_observation::AggregateObservation,
};
use std::{marker::PhantomData, sync::Arc};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::KindId;

/// A complete component inspected under broad installation projection authority.
/// Retention preserves the original Native request custody, not read authority.
pub struct WorthQueryInvariantProjectionComponent<Schema, Set, Entity> {
    aggregate: Arc<AggregateObservation>,
    identity: WorthQueryAggregateIdentity<Schema, Set>,
    member: Arc<str>,
    member_kind: KindId,
    authority: u64,
    _entity: PhantomData<fn() -> Entity>,
}
impl<Schema, Set, Entity> WorthQueryInvariantProjectionComponent<Schema, Set, Entity> {
    pub(super) fn new(
        aggregate: Arc<AggregateObservation>,
        layout: &ComponentSetLayout,
        binding: ApplicationSchemaBindingIdentity,
        authority: u64,
    ) -> Self {
        Self {
            identity: WorthQueryAggregateIdentity::from_component_read(aggregate.native(), binding),
            aggregate,
            member: Arc::clone(&layout.member),
            member_kind: layout.member_kind,
            authority,
            _entity: PhantomData,
        }
    }
    pub fn identity(&self) -> &WorthQueryAggregateIdentity<Schema, Set> {
        &self.identity
    }
    pub fn len(&self) -> usize {
        self.aggregate.native().len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    /// Native canonical membership order; iteration performs no graph reads.
    pub fn members(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryInvariantEntityIdentity<Schema, Entity>> + '_ {
        self.aggregate
            .native()
            .members()
            .map(|entity_id| WorthQueryInvariantEntityIdentity {
                entity_id,
                kind: self.member_kind,
                entity: Arc::clone(&self.member),
                authority_identity: self.authority,
                _marker: PhantomData,
            })
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn aggregate(
        &self,
    ) -> &Arc<AggregateObservation> {
        &self.aggregate
    }
}
impl<Schema, Set, Entity> std::fmt::Debug
    for WorthQueryInvariantProjectionComponent<Schema, Set, Entity>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryInvariantProjectionComponent")
            .field("identity", &self.identity)
            .field("members", &self.len())
            .finish_non_exhaustive()
    }
}
