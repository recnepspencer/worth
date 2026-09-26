use worth_foundational::facade::AspectFieldLocator;

use crate::identity::data::{EntityId, KindId};
use crate::snapshots::data::SnapshotHandle;
use crate::transactions::data::RecordRef;

use super::{
    plan_validation::validate_plan, RelationalAuthorizationEntityAnchor,
    RelationalAuthorizationExactAdjacencyConstraint, RelationalAuthorizationFieldConstraint,
    RelationalAuthorizationPlanDenial, RelationalAuthorizationPredicate,
    RelationalAuthorizationRelatedEntityConstraint,
};

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum RelationalAuthorizationTraversalDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationTraversal {
    relation_kind: KindId,
    from_kind: KindId,
    to_kind: KindId,
    direction: RelationalAuthorizationTraversalDirection,
}

impl RelationalAuthorizationTraversal {
    pub fn new(
        relation_kind: KindId,
        from_kind: KindId,
        to_kind: KindId,
        direction: RelationalAuthorizationTraversalDirection,
    ) -> Self {
        Self {
            relation_kind,
            from_kind,
            to_kind,
            direction,
        }
    }

    pub const fn relation_kind(&self) -> KindId {
        self.relation_kind
    }

    pub const fn from_kind(&self) -> KindId {
        self.from_kind
    }

    pub const fn to_kind(&self) -> KindId {
        self.to_kind
    }

    pub const fn direction(&self) -> RelationalAuthorizationTraversalDirection {
        self.direction
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationPathPlan {
    traversals: Vec<RelationalAuthorizationTraversal>,
    predicates: Vec<RelationalAuthorizationPredicate>,
    field_constraints: Vec<RelationalAuthorizationFieldConstraint>,
    entity_anchors: Vec<RelationalAuthorizationEntityAnchor>,
    related_entities: Vec<RelationalAuthorizationRelatedEntityConstraint>,
    exact_adjacencies: Vec<RelationalAuthorizationExactAdjacencyConstraint>,
}

impl RelationalAuthorizationPathPlan {
    pub fn new(
        traversals: impl IntoIterator<Item = RelationalAuthorizationTraversal>,
        predicates: impl IntoIterator<Item = RelationalAuthorizationPredicate>,
    ) -> Self {
        Self {
            traversals: traversals.into_iter().collect(),
            predicates: predicates.into_iter().collect(),
            field_constraints: Vec::new(),
            entity_anchors: Vec::new(),
            related_entities: Vec::new(),
            exact_adjacencies: Vec::new(),
        }
    }

    pub fn with_field_constraints(
        mut self,
        constraints: impl IntoIterator<Item = RelationalAuthorizationFieldConstraint>,
    ) -> Self {
        self.field_constraints = constraints.into_iter().collect();
        self
    }

    pub fn with_predicates(
        mut self,
        predicates: impl IntoIterator<Item = RelationalAuthorizationPredicate>,
    ) -> Self {
        self.predicates = predicates.into_iter().collect();
        self
    }

    pub fn with_entity_anchors(
        mut self,
        anchors: impl IntoIterator<Item = RelationalAuthorizationEntityAnchor>,
    ) -> Self {
        self.entity_anchors = anchors.into_iter().collect();
        self
    }

    pub fn with_related_entities(
        mut self,
        constraints: impl IntoIterator<Item = RelationalAuthorizationRelatedEntityConstraint>,
    ) -> Self {
        self.related_entities = constraints.into_iter().collect();
        self
    }

    pub fn with_exact_adjacencies(
        mut self,
        constraints: impl IntoIterator<Item = RelationalAuthorizationExactAdjacencyConstraint>,
    ) -> Self {
        self.exact_adjacencies = constraints.into_iter().collect();
        self
    }

    pub fn traversals(&self) -> &[RelationalAuthorizationTraversal] {
        &self.traversals
    }

    pub fn predicates(&self) -> &[RelationalAuthorizationPredicate] {
        &self.predicates
    }

    pub fn field_constraints(&self) -> &[RelationalAuthorizationFieldConstraint] {
        &self.field_constraints
    }

    pub fn entity_anchors(&self) -> &[RelationalAuthorizationEntityAnchor] {
        &self.entity_anchors
    }

    pub fn related_entities(&self) -> &[RelationalAuthorizationRelatedEntityConstraint] {
        &self.related_entities
    }

    pub fn exact_adjacencies(&self) -> &[RelationalAuthorizationExactAdjacencyConstraint] {
        &self.exact_adjacencies
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationEffectTarget {
    record: RecordRef,
    field: Option<AspectFieldLocator>,
}

impl RelationalAuthorizationEffectTarget {
    pub fn record(record: RecordRef) -> Self {
        Self {
            record,
            field: None,
        }
    }

    pub fn field(record: RecordRef, field: AspectFieldLocator) -> Self {
        Self {
            record,
            field: Some(field),
        }
    }

    pub const fn record_ref(&self) -> &RecordRef {
        &self.record
    }

    pub fn field_locator(&self) -> Option<&AspectFieldLocator> {
        self.field.as_ref()
    }
}

#[derive(Debug)]
pub struct RelationalAuthorizationObservationPlan {
    snapshot: SnapshotHandle,
    principal: EntityId,
    scope: EntityId,
    principal_kind: KindId,
    scope_kind: KindId,
    paths: Vec<RelationalAuthorizationPathPlan>,
    proposed_effects: Vec<RelationalAuthorizationEffectTarget>,
}

impl RelationalAuthorizationObservationPlan {
    pub fn try_new(
        snapshot: SnapshotHandle,
        principal: EntityId,
        scope: EntityId,
        principal_kind: KindId,
        scope_kind: KindId,
        paths: impl IntoIterator<Item = RelationalAuthorizationPathPlan>,
        proposed_effects: impl IntoIterator<Item = RelationalAuthorizationEffectTarget>,
    ) -> Result<Self, RelationalAuthorizationPlanDenial> {
        let plan = Self {
            snapshot,
            principal,
            scope,
            principal_kind,
            scope_kind,
            paths: paths.into_iter().collect(),
            proposed_effects: proposed_effects.into_iter().collect(),
        };
        validate_plan(&plan)?;
        Ok(plan)
    }

    pub fn snapshot(&self) -> &SnapshotHandle {
        &self.snapshot
    }

    pub const fn principal(&self) -> EntityId {
        self.principal
    }

    pub const fn scope(&self) -> EntityId {
        self.scope
    }

    pub const fn principal_kind(&self) -> KindId {
        self.principal_kind
    }

    pub const fn scope_kind(&self) -> KindId {
        self.scope_kind
    }

    pub fn paths(&self) -> &[RelationalAuthorizationPathPlan] {
        &self.paths
    }

    pub fn proposed_effects(&self) -> &[RelationalAuthorizationEffectTarget] {
        &self.proposed_effects
    }

    pub(crate) fn comparison_at(
        &self,
        snapshot: SnapshotHandle,
    ) -> Result<Self, RelationalAuthorizationPlanDenial> {
        let plan = Self {
            snapshot,
            principal: self.principal,
            scope: self.scope,
            principal_kind: self.principal_kind,
            scope_kind: self.scope_kind,
            paths: self.paths.clone(),
            proposed_effects: self.proposed_effects.clone(),
        };
        validate_plan(&plan)?;
        Ok(plan)
    }
}
