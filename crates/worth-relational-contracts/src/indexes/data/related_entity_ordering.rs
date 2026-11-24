use serde::{Deserialize, Serialize};
use worth_foundational::facade::AspectFieldLocator;

use crate::identity::data::{EntityId, RelationId};
use worth_foundational::facade::AspectValue;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RelatedEntityEndpoint {
    SourceParent,
    TargetParent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RelatedEntityOrderingDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelatedEntityOrderingField {
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
    locator: AspectFieldLocator,
    direction: RelatedEntityOrderingDirection,
}

impl RelatedEntityOrderingField {
    pub fn new(locator: AspectFieldLocator, direction: RelatedEntityOrderingDirection) -> Self {
        Self { locator, direction }
    }

    pub fn locator(&self) -> &AspectFieldLocator {
        &self.locator
    }

    pub const fn direction(&self) -> RelatedEntityOrderingDirection {
        self.direction
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelatedEntityOrderingValue {
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_value")]
    value: AspectValue,
}

impl RelatedEntityOrderingValue {
    pub const fn value(&self) -> &AspectValue {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RelatedEntityOrderingEntry {
    ordering_values: Vec<RelatedEntityOrderingValue>,
    child_entity_id: EntityId,
    relation_id: RelationId,
}

impl RelatedEntityOrderingEntry {
    pub fn ordering_values(&self) -> &[RelatedEntityOrderingValue] {
        &self.ordering_values
    }

    pub const fn child_entity_id(&self) -> EntityId {
        self.child_entity_id
    }

    pub const fn relation_id(&self) -> RelationId {
        self.relation_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelatedEntityOrderingBoundary {
    entry: RelatedEntityOrderingEntry,
}

/// Operations used by the Relational owners.
pub trait RelatedEntityOrderingEntryOwnerPort: Sized {
    fn new(
        ordering_values: Vec<AspectValue>,
        child_entity_id: EntityId,
        relation_id: RelationId,
    ) -> Self;
    fn owned_allocation_capacity_bytes(&self) -> u64;
}

impl RelatedEntityOrderingEntryOwnerPort for RelatedEntityOrderingEntry {
    fn new(
        ordering_values: Vec<AspectValue>,
        child_entity_id: EntityId,
        relation_id: RelationId,
    ) -> Self {
        Self {
            ordering_values: ordering_values
                .into_iter()
                .map(|value| RelatedEntityOrderingValue { value })
                .collect(),
            child_entity_id,
            relation_id,
        }
    }

    fn owned_allocation_capacity_bytes(&self) -> u64 {
        (self.ordering_values.capacity() as u64)
            .saturating_mul(std::mem::size_of::<RelatedEntityOrderingValue>() as u64)
            .saturating_add(
                self.ordering_values
                    .iter()
                    .map(|value| value.value.owned_allocation_capacity_bytes() as u64)
                    .sum(),
            )
    }
}

/// Operations used by the Relational owners.
pub trait RelatedEntityOrderingBoundaryOwnerPort: Sized {
    fn from_entry(entry: &RelatedEntityOrderingEntry) -> Self;
    fn entry(&self) -> &RelatedEntityOrderingEntry;
}

impl RelatedEntityOrderingBoundaryOwnerPort for RelatedEntityOrderingBoundary {
    fn from_entry(entry: &RelatedEntityOrderingEntry) -> Self {
        Self {
            entry: entry.clone(),
        }
    }

    fn entry(&self) -> &RelatedEntityOrderingEntry {
        &self.entry
    }
}
