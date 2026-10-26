//! Owned Native facts; typed iteration acquires no more graph metadata.
use super::super::{WorthQueryInvariantEntityIdentity, WorthQueryInvariantRelation};
use std::{marker::PhantomData, sync::Arc};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{PinnedStructuralWalk, PinnedStructuralWalkVisit},
};

/// Complete reachability on one pinned projection, with its original result payer.
/// Borrowed iteration mints local typed identities without additional graph reads.
pub struct WorthQueryInvariantProjectionWalk<Schema, Relation, Entity> {
    pub(super) native: PinnedStructuralWalk,
    entity: Option<Arc<str>>,
    authority_identity: u64,
    kind: Option<KindId>,
    _marker: PhantomData<fn() -> (Schema, Relation, Entity)>,
}

/// One reached entity and its minimum distance from the supplied roots.
pub struct WorthQueryInvariantProjectionWalkStep<Schema, Entity> {
    entity: WorthQueryInvariantEntityIdentity<Schema, Entity>,
    distance: usize,
}

/// Allocation-free typed iteration in minimum-distance, then native-identity order.
pub struct WorthQueryInvariantProjectionWalkIter<'walk, Schema, Relation, Entity> {
    walk: &'walk WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
    visits: std::slice::Iter<'walk, PinnedStructuralWalkVisit>,
}

impl<Schema, Relation, Entity> WorthQueryInvariantProjectionWalk<Schema, Relation, Entity> {
    pub(super) fn new(
        native: PinnedStructuralWalk,
        root: Option<&WorthQueryInvariantEntityIdentity<Schema, Entity>>,
    ) -> Self {
        Self {
            native,
            entity: root.map(|r| Arc::clone(&r.entity)),
            authority_identity: root.map_or(0, |r| r.authority_identity),
            kind: root.map(|r| r.kind),
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.native.visits().len()
    }
    pub fn is_empty(&self) -> bool {
        self.native.visits().is_empty()
    }
    pub fn iter(&self) -> WorthQueryInvariantProjectionWalkIter<'_, Schema, Relation, Entity> {
        WorthQueryInvariantProjectionWalkIter {
            walk: self,
            visits: self.native.visits().iter(),
        }
    }
    pub fn entities(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryInvariantEntityIdentity<Schema, Entity>> + '_ {
        self.native
            .visits()
            .iter()
            .map(|v| self.entity(v.entity.entity_id))
    }
    /// Every physical matching edge, including nondiscovery edges.
    pub fn relations(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryInvariantRelation<Schema, Relation, Entity, Entity>> + '_
    {
        self.native
            .relations()
            .iter()
            .map(|r| WorthQueryInvariantRelation {
                relation_id: r.relation_id,
                from: self.entity(r.source),
                to: self.entity(r.target),
                _relation: PhantomData,
            })
    }
    fn entity(&self, entity_id: EntityId) -> WorthQueryInvariantEntityIdentity<Schema, Entity> {
        WorthQueryInvariantEntityIdentity {
            entity_id,
            kind: self.kind.expect("a nonempty walk has a validated root"),
            entity: Arc::clone(
                self.entity
                    .as_ref()
                    .expect("a nonempty walk has a root name"),
            ),
            authority_identity: self.authority_identity,
            _marker: PhantomData,
        }
    }
}

impl<Schema, Entity> WorthQueryInvariantProjectionWalkStep<Schema, Entity> {
    pub fn entity(&self) -> &WorthQueryInvariantEntityIdentity<Schema, Entity> {
        &self.entity
    }
    pub const fn distance(&self) -> usize {
        self.distance
    }
}

impl<Schema, Relation, Entity> Iterator
    for WorthQueryInvariantProjectionWalkIter<'_, Schema, Relation, Entity>
{
    type Item = WorthQueryInvariantProjectionWalkStep<Schema, Entity>;
    fn next(&mut self) -> Option<Self::Item> {
        self.visits
            .next()
            .map(|v| WorthQueryInvariantProjectionWalkStep {
                entity: self.walk.entity(v.entity.entity_id),
                distance: v.distance,
            })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.visits.size_hint()
    }
}
impl<Schema, Relation, Entity> ExactSizeIterator
    for WorthQueryInvariantProjectionWalkIter<'_, Schema, Relation, Entity>
{
}
impl<Schema, Relation, Entity> std::iter::FusedIterator
    for WorthQueryInvariantProjectionWalkIter<'_, Schema, Relation, Entity>
{
}
impl<Schema, Relation, Entity> DoubleEndedIterator
    for WorthQueryInvariantProjectionWalkIter<'_, Schema, Relation, Entity>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.visits
            .next_back()
            .map(|v| WorthQueryInvariantProjectionWalkStep {
                entity: self.walk.entity(v.entity.entity_id),
                distance: v.distance,
            })
    }
}
impl<'walk, Schema, Relation, Entity> IntoIterator
    for &'walk WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>
{
    type Item = WorthQueryInvariantProjectionWalkStep<Schema, Entity>;
    type IntoIter = WorthQueryInvariantProjectionWalkIter<'walk, Schema, Relation, Entity>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<Schema, Relation, Entity> std::fmt::Debug
    for WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryInvariantProjectionWalk")
            .field("facts", &self.native)
            .finish_non_exhaustive()
    }
}
