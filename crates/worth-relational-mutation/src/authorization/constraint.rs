use worth_foundational::facade::{AspectFieldLocator, AspectValue};

use worth_relational_contracts::internal_port::identity::data::primitives::KindId;
use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;

use super::RelationalAuthorizationTraversal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalAuthorizationFieldComparison {
    Equal,
    AtMost,
    AtLeast,
    StrictlyLess,
    StrictlyGreater,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationPredicate {
    traversal_ordinal: usize,
    entity_kind: KindId,
    field: AspectFieldLocator,
    comparison: RelationalAuthorizationFieldComparison,
    expected: AspectValue,
}

impl RelationalAuthorizationPredicate {
    pub fn new(
        traversal_ordinal: usize,
        entity_kind: KindId,
        field: AspectFieldLocator,
        expected: AspectValue,
    ) -> Self {
        Self::compare(
            traversal_ordinal,
            entity_kind,
            field,
            RelationalAuthorizationFieldComparison::Equal,
            expected,
        )
    }

    pub fn compare(
        traversal_ordinal: usize,
        entity_kind: KindId,
        field: AspectFieldLocator,
        comparison: RelationalAuthorizationFieldComparison,
        expected: AspectValue,
    ) -> Self {
        Self {
            traversal_ordinal,
            entity_kind,
            field,
            comparison,
            expected,
        }
    }

    pub const fn traversal_ordinal(&self) -> usize {
        self.traversal_ordinal
    }

    pub const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    pub fn field(&self) -> &AspectFieldLocator {
        &self.field
    }

    pub const fn comparison(&self) -> RelationalAuthorizationFieldComparison {
        self.comparison
    }

    pub fn expected(&self) -> &AspectValue {
        &self.expected
    }

    pub(crate) fn matches(&self, observed: &AspectValue) -> bool {
        if observed.value_family() != self.expected.value_family() {
            return false;
        }
        match self.comparison {
            RelationalAuthorizationFieldComparison::Equal => observed == &self.expected,
            RelationalAuthorizationFieldComparison::AtMost => observed <= &self.expected,
            RelationalAuthorizationFieldComparison::AtLeast => observed >= &self.expected,
            RelationalAuthorizationFieldComparison::StrictlyLess => observed < &self.expected,
            RelationalAuthorizationFieldComparison::StrictlyGreater => observed > &self.expected,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationFieldOperand {
    traversal_ordinal: usize,
    entity_kind: KindId,
    field: AspectFieldLocator,
}

impl RelationalAuthorizationFieldOperand {
    pub fn new(traversal_ordinal: usize, entity_kind: KindId, field: AspectFieldLocator) -> Self {
        Self {
            traversal_ordinal,
            entity_kind,
            field,
        }
    }

    pub const fn traversal_ordinal(&self) -> usize {
        self.traversal_ordinal
    }

    pub const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    pub const fn field(&self) -> &AspectFieldLocator {
        &self.field
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationFieldConstraint {
    left: RelationalAuthorizationFieldOperand,
    comparison: RelationalAuthorizationFieldComparison,
    right: RelationalAuthorizationFieldOperand,
}

impl RelationalAuthorizationFieldConstraint {
    pub fn new(
        left: RelationalAuthorizationFieldOperand,
        comparison: RelationalAuthorizationFieldComparison,
        right: RelationalAuthorizationFieldOperand,
    ) -> Self {
        Self {
            left,
            comparison,
            right,
        }
    }

    pub const fn left(&self) -> &RelationalAuthorizationFieldOperand {
        &self.left
    }

    pub const fn comparison(&self) -> RelationalAuthorizationFieldComparison {
        self.comparison
    }

    pub const fn right(&self) -> &RelationalAuthorizationFieldOperand {
        &self.right
    }

    pub(crate) fn matches(&self, left: &AspectValue, right: &AspectValue) -> bool {
        if left.value_family() != right.value_family() {
            return false;
        }
        match self.comparison {
            RelationalAuthorizationFieldComparison::Equal => left == right,
            RelationalAuthorizationFieldComparison::AtMost => left <= right,
            RelationalAuthorizationFieldComparison::AtLeast => left >= right,
            RelationalAuthorizationFieldComparison::StrictlyLess => left < right,
            RelationalAuthorizationFieldComparison::StrictlyGreater => left > right,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationEntityAnchor {
    traversal_ordinal: usize,
    entity_kind: KindId,
    entity: EntityId,
}

impl RelationalAuthorizationEntityAnchor {
    pub const fn new(traversal_ordinal: usize, entity_kind: KindId, entity: EntityId) -> Self {
        Self {
            traversal_ordinal,
            entity_kind,
            entity,
        }
    }

    pub const fn traversal_ordinal(&self) -> usize {
        self.traversal_ordinal
    }

    pub const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationRelatedEntityConstraint {
    traversal_ordinal: usize,
    traversal: RelationalAuthorizationTraversal,
    entity: EntityId,
}

impl RelationalAuthorizationRelatedEntityConstraint {
    pub fn new(
        traversal_ordinal: usize,
        traversal: RelationalAuthorizationTraversal,
        entity: EntityId,
    ) -> Self {
        Self {
            traversal_ordinal,
            traversal,
            entity,
        }
    }

    pub const fn traversal_ordinal(&self) -> usize {
        self.traversal_ordinal
    }

    pub const fn traversal(&self) -> &RelationalAuthorizationTraversal {
        &self.traversal
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }
}

/// Requires the complete visible adjacency at one path entity to equal an
/// expected multiset of related entities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalAuthorizationExactAdjacencyConstraint {
    traversal_ordinal: usize,
    traversal: RelationalAuthorizationTraversal,
    expected_entities: Vec<EntityId>,
}

impl RelationalAuthorizationExactAdjacencyConstraint {
    pub fn new(
        traversal_ordinal: usize,
        traversal: RelationalAuthorizationTraversal,
        expected_entities: impl IntoIterator<Item = EntityId>,
    ) -> Self {
        let mut expected_entities = expected_entities.into_iter().collect::<Vec<_>>();
        expected_entities.sort_unstable();
        Self {
            traversal_ordinal,
            traversal,
            expected_entities,
        }
    }

    pub const fn traversal_ordinal(&self) -> usize {
        self.traversal_ordinal
    }

    pub const fn traversal(&self) -> &RelationalAuthorizationTraversal {
        &self.traversal
    }

    pub fn expected_entities(&self) -> &[EntityId] {
        &self.expected_entities
    }
}
