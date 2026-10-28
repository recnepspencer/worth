//! Owned native facts with allocation-free typed iteration.
use std::marker::PhantomData;

use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::{CompleteStructuralWalk, StructuralWalkStep};

use super::super::{
    ViewPosture, WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantReadView,
    WorthQueryApplicationInvariantRelation,
};

/// Complete, canonically ordered reachability on one exact invariant view.
/// Borrowed iteration materializes fixed-size typed values; native facts keep
/// the original payer alive for the lifetime of this owner.
pub struct WorthQueryApplicationInvariantWalk<Schema, Relation, Entity> {
    native: CompleteStructuralWalk,
    binding_identity: ApplicationSchemaBindingIdentity,
    proposal_affinity: Option<(u64, u64)>,
    posture: ViewPosture,
    kind: KindId,
    _marker: PhantomData<fn() -> (Schema, Relation, Entity)>,
}

pub struct WorthQueryApplicationInvariantWalkStep<Schema, Entity> {
    entity: WorthQueryApplicationInvariantEntity<Schema, Entity>,
    distance: usize,
}

pub struct WorthQueryApplicationInvariantWalkIter<'walk, Schema, Relation, Entity> {
    walk: &'walk WorthQueryApplicationInvariantWalk<Schema, Relation, Entity>,
    steps: std::slice::Iter<'walk, StructuralWalkStep>,
}

impl<Schema, Relation, Entity> WorthQueryApplicationInvariantWalk<Schema, Relation, Entity> {
    pub(super) fn new(
        view: &WorthQueryApplicationInvariantReadView<'_, Schema>,
        native: CompleteStructuralWalk,
        kind: KindId,
    ) -> Self {
        Self {
            native,
            binding_identity: view.binding_identity.clone(),
            proposal_affinity: view.proposal_affinity,
            posture: view.posture,
            kind,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.native.steps().len()
    }

    pub fn is_empty(&self) -> bool {
        self.native.steps().is_empty()
    }

    pub fn iter(&self) -> WorthQueryApplicationInvariantWalkIter<'_, Schema, Relation, Entity> {
        WorthQueryApplicationInvariantWalkIter {
            walk: self,
            steps: self.native.steps().iter(),
        }
    }

    pub fn entities(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthQueryApplicationInvariantEntity<Schema, Entity>> + '_
    {
        self.native
            .steps()
            .iter()
            .map(|step| self.entity(step.entity))
    }

    /// Every matching physical edge, including self/back/diamond edges that
    /// discover no new entity. Iteration performs no additional graph reads.
    pub fn relations(
        &self,
    ) -> impl ExactSizeIterator<
        Item = WorthQueryApplicationInvariantRelation<Schema, Relation, Entity, Entity>,
    > + '_ {
        self.native
            .relations()
            .iter()
            .map(|record| WorthQueryApplicationInvariantRelation {
                relation_id: record.relation_id,
                from: self.entity(record.source),
                to: self.entity(record.target),
                _marker: PhantomData,
            })
    }

    pub(super) fn contains_id(&self, entity: EntityId) -> bool {
        self.native.steps().iter().any(|step| step.entity == entity)
    }

    fn entity(&self, entity_id: EntityId) -> WorthQueryApplicationInvariantEntity<Schema, Entity> {
        WorthQueryApplicationInvariantEntity {
            entity_id,
            kind: self.kind,
            binding_identity: self.binding_identity.clone(),
            proposal_affinity: self.proposal_affinity,
            posture: self.posture,
            _marker: PhantomData,
        }
    }
}

impl<Schema, Entity> WorthQueryApplicationInvariantWalkStep<Schema, Entity> {
    pub fn entity(&self) -> &WorthQueryApplicationInvariantEntity<Schema, Entity> {
        &self.entity
    }

    pub const fn distance(&self) -> usize {
        self.distance
    }
}

impl<Schema, Relation, Entity> Iterator
    for WorthQueryApplicationInvariantWalkIter<'_, Schema, Relation, Entity>
{
    type Item = WorthQueryApplicationInvariantWalkStep<Schema, Entity>;

    fn next(&mut self) -> Option<Self::Item> {
        self.steps
            .next()
            .map(|step| WorthQueryApplicationInvariantWalkStep {
                entity: self.walk.entity(step.entity),
                distance: step.distance,
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.steps.size_hint()
    }

    fn nth(&mut self, index: usize) -> Option<Self::Item> {
        self.steps
            .nth(index)
            .map(|step| WorthQueryApplicationInvariantWalkStep {
                entity: self.walk.entity(step.entity),
                distance: step.distance,
            })
    }
}

impl<Schema, Relation, Entity> ExactSizeIterator
    for WorthQueryApplicationInvariantWalkIter<'_, Schema, Relation, Entity>
{
}

impl<Schema, Relation, Entity> std::iter::FusedIterator
    for WorthQueryApplicationInvariantWalkIter<'_, Schema, Relation, Entity>
{
}

impl<Schema, Relation, Entity> DoubleEndedIterator
    for WorthQueryApplicationInvariantWalkIter<'_, Schema, Relation, Entity>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.steps
            .next_back()
            .map(|step| WorthQueryApplicationInvariantWalkStep {
                entity: self.walk.entity(step.entity),
                distance: step.distance,
            })
    }
}

impl<'walk, Schema, Relation, Entity> IntoIterator
    for &'walk WorthQueryApplicationInvariantWalk<Schema, Relation, Entity>
{
    type Item = WorthQueryApplicationInvariantWalkStep<Schema, Entity>;
    type IntoIter = WorthQueryApplicationInvariantWalkIter<'walk, Schema, Relation, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<Schema, Relation, Entity> std::fmt::Debug
    for WorthQueryApplicationInvariantWalk<Schema, Relation, Entity>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationInvariantWalk")
            .field("entities", &self.native.steps().len())
            .field("relations", &self.native.relations().len())
            .finish_non_exhaustive()
    }
}
