//! Both actual Native owners survive borrowed, allocation-free typed iteration.
use super::super::{
    WorthQueryInvariantEntityIdentity, WorthQueryInvariantProjectionWalk,
    WorthQueryInvariantRelation,
};
use crate::domain_computation::primary_graph::aggregate_identity::WorthQueryAggregateIdentity;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::{identity::KindId, runtime::PinnedAggregateRead};

/// Complete orbit inspected at one projection snapshot. Membership order comes
/// from the successor relation, while persistent naming comes from Native storage.
pub struct WorthQueryInvariantProjectionOrbit<Schema, Relation, Entity> {
    aggregate: PinnedAggregateRead,
    walk: WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
    identity: WorthQueryAggregateIdentity<Schema, Relation>,
}
impl<Schema, Relation, Entity> WorthQueryInvariantProjectionOrbit<Schema, Relation, Entity> {
    pub(super) fn new(
        aggregate: PinnedAggregateRead,
        walk: WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
        binding: ApplicationSchemaBindingIdentity,
        relation: KindId,
    ) -> Self {
        let identity = WorthQueryAggregateIdentity::from_pinned_read(&aggregate, binding, relation);
        Self {
            aggregate,
            walk,
            identity,
        }
    }
    pub fn identity(&self) -> &WorthQueryAggregateIdentity<Schema, Relation> {
        &self.identity
    }
    pub fn len(&self) -> usize {
        self.aggregate.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
    /// Successor order from the current anchor, independent of the selected member.
    pub fn members(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryInvariantEntityIdentity<Schema, Entity>> + '_ {
        self.walk.entities()
    }
    /// All actual physical successor edges. Iteration performs no graph reads.
    pub fn relations(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryInvariantRelation<Schema, Relation, Entity, Entity>> + '_
    {
        self.walk.relations()
    }
    /// Constant-time selection from the already materialized successor order.
    pub fn member(
        &self,
        index: usize,
    ) -> Option<WorthQueryInvariantEntityIdentity<Schema, Entity>> {
        self.members().nth(index)
    }
}
impl<Schema, Relation, Entity> std::fmt::Debug
    for WorthQueryInvariantProjectionOrbit<Schema, Relation, Entity>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryInvariantProjectionOrbit")
            .field("identity", &self.identity)
            .field("members", &self.len())
            .finish_non_exhaustive()
    }
}
