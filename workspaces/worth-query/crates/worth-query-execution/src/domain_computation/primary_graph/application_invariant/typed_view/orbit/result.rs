//! Both actual Native owners, with typed allocation-free successor iteration.
use super::super::{
    WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantRelation,
    WorthQueryApplicationInvariantWalk,
};
use crate::domain_computation::primary_graph::aggregate_identity::WorthQueryAggregateIdentity;
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::{identity::KindId, runtime::AggregateRead};

/// One complete orbit, retaining actual Native identity and business walk truth.
pub struct WorthQueryApplicationInvariantOrbit<Schema, Relation, Entity> {
    aggregate: AggregateRead,
    walk: WorthQueryApplicationInvariantWalk<Schema, Relation, Entity>,
    identity: WorthQueryAggregateIdentity<Schema, Relation>,
}
impl<Schema, Relation, Entity> WorthQueryApplicationInvariantOrbit<Schema, Relation, Entity> {
    pub(super) fn new(
        aggregate: AggregateRead,
        walk: WorthQueryApplicationInvariantWalk<Schema, Relation, Entity>,
        binding: ApplicationSchemaBindingIdentity,
        relation: KindId,
    ) -> Self {
        let identity =
            WorthQueryAggregateIdentity::from_invariant_read(&aggregate, binding, relation);
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
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationInvariantEntity<Schema, Entity>> + '_
    {
        self.walk.entities()
    }
    /// The complete source relation's physical edges, with no additional reads.
    pub fn relations(
        &self,
    ) -> impl ExactSizeIterator<
        Item = WorthQueryApplicationInvariantRelation<Schema, Relation, Entity, Entity>,
    > + '_ {
        self.walk.relations()
    }
    /// Constant-time indexing into the already materialized successor order.
    pub fn member(
        &self,
        index: usize,
    ) -> Option<WorthQueryApplicationInvariantEntity<Schema, Entity>> {
        self.members().nth(index)
    }
}
impl<Schema, Relation, Entity> std::fmt::Debug
    for WorthQueryApplicationInvariantOrbit<Schema, Relation, Entity>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationInvariantOrbit")
            .field("identity", &self.identity)
            .field("members", &self.len())
            .finish_non_exhaustive()
    }
}
