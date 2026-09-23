use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::identity::data::{EntityId, KindId, RelationId};
use crate::transactions::data::{CreatedEntityRef, EntityReference};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PlannedRelationEdge {
    pub(crate) source: EntityReference,
    pub(crate) target: EntityReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PreparedVisibleRelationEdge {
    pub(crate) relation_id: RelationId,
    pub(crate) source: EntityId,
    pub(crate) target: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PreparedRelationPairKey {
    pub(crate) source: EntityReference,
    pub(crate) target: EntityReference,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PreparedRelationEndpointKey {
    pub(crate) entity_id: EntityReference,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PreparedRelationIntegrityScope {
    pub(crate) created_candidate_entities: BTreeSet<CreatedEntityRef>,
    pub(crate) minimum_touched_entities: BTreeSet<EntityId>,
    pub(crate) planned_edges: Vec<PlannedRelationEdge>,
    pub(crate) visible_edges: Vec<PreparedVisibleRelationEdge>,
    pub(crate) visible_successors: BTreeMap<EntityReference, Vec<EntityReference>>,
    pub(crate) source_counts: BTreeMap<PreparedRelationEndpointKey, usize>,
    pub(crate) target_counts: BTreeMap<PreparedRelationEndpointKey, usize>,
    pub(crate) directed_pair_counts: BTreeMap<PreparedRelationPairKey, usize>,
    pub(crate) normalized_pair_counts: BTreeMap<PreparedRelationPairKey, usize>,
    pub(crate) deleted_entities: BTreeSet<EntityId>,
    pub(crate) deleted_relation_count: usize,
    pub(crate) requires_global_evaluation: bool,
    pub(crate) requires_visible_successors: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedRelationIntegrityScopes(
    Arc<BTreeMap<KindId, PreparedRelationIntegrityScope>>,
);

impl PreparedRelationIntegrityScopes {
    pub(crate) fn new(scopes: BTreeMap<KindId, PreparedRelationIntegrityScope>) -> Self {
        Self(Arc::new(scopes))
    }

    pub(crate) fn scope_for(
        &self,
        relation_kind_id: KindId,
    ) -> Option<&PreparedRelationIntegrityScope> {
        self.0.get(&relation_kind_id)
    }
}

impl PreparedRelationIntegrityScope {
    pub(crate) fn is_empty(&self) -> bool {
        self.created_candidate_entities.is_empty()
            && self.minimum_touched_entities.is_empty()
            && self.planned_edges.is_empty()
            && self.visible_edges.is_empty()
            && self.source_counts.is_empty()
            && self.target_counts.is_empty()
            && self.directed_pair_counts.is_empty()
            && self.deleted_entities.is_empty()
            && self.deleted_relation_count == 0
    }

    pub(crate) fn should_execute(&self) -> bool {
        self.requires_global_evaluation || !self.is_empty()
    }

    pub(crate) fn record_visible_successor(
        &mut self,
        source: EntityReference,
        target: EntityReference,
    ) {
        self.visible_successors
            .entry(source)
            .or_default()
            .push(target);
    }

    pub(crate) fn increment_counts(&mut self, source: EntityReference, target: EntityReference) {
        *self
            .source_counts
            .entry(PreparedRelationEndpointKey {
                entity_id: source.clone(),
            })
            .or_insert(0) += 1;
        *self
            .target_counts
            .entry(PreparedRelationEndpointKey {
                entity_id: target.clone(),
            })
            .or_insert(0) += 1;
        *self
            .directed_pair_counts
            .entry(PreparedRelationPairKey {
                source: source.clone(),
                target: target.clone(),
            })
            .or_insert(0) += 1;
        let (left, right) = if target < source {
            (target, source)
        } else {
            (source, target)
        };
        *self
            .normalized_pair_counts
            .entry(PreparedRelationPairKey {
                source: left,
                target: right,
            })
            .or_insert(0) += 1;
    }
}
