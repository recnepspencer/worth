//! Replayable descriptors for installed derived indexes.
use super::related_entity_ordering::{RelatedEntityEndpoint, RelatedEntityOrderingField};
use super::relation_join::RelationJoinDefinition;
use serde::{Deserialize, Serialize};
use worth_foundational::facade::AspectFieldLocator;

/// Identifies a derived-index definition within its owning runtime.
/// The coordinate grants no read, mutation or lifecycle authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DerivedIndexId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DerivedIndexGenerationId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivedIndexKind {
    EntityField {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
        field_locator: AspectFieldLocator,
    },
    RelationField {
        #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
        field_locator: AspectFieldLocator,
    },
    RelatedEntityOrdering {
        relation_kind: crate::identity::data::KindId,
        parent_endpoint: RelatedEntityEndpoint,
        child_kind: crate::identity::data::KindId,
        ordering: Vec<RelatedEntityOrderingField>,
    },
    RelationJoin(RelationJoinDefinition),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedIndexDefinition {
    pub index_id: DerivedIndexId,
    pub name: String,
    pub kind: DerivedIndexKind,
    pub branch_scoped: bool,
    /// Declare native uniqueness for one entity kind on this equality index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_entity_kind: Option<crate::identity::data::KindId>,
}

/// Operations used by the Relational owners.
pub trait DerivedIndexDefinitionOwnerPort: Sized {
    fn owned_allocation_capacity_bytes(&self) -> u64;
}

impl DerivedIndexDefinitionOwnerPort for DerivedIndexDefinition {
    fn owned_allocation_capacity_bytes(&self) -> u64 {
        let kind_bytes = match &self.kind {
            DerivedIndexKind::EntityField { field_locator }
            | DerivedIndexKind::RelationField { field_locator } => {
                field_locator.owned_allocation_capacity_bytes() as u64
            }
            DerivedIndexKind::RelatedEntityOrdering { ordering, .. } => (ordering.capacity()
                as u64)
                .saturating_mul(std::mem::size_of::<RelatedEntityOrderingField>() as u64)
                .saturating_add(
                    ordering
                        .iter()
                        .map(|field| field.locator().owned_allocation_capacity_bytes() as u64)
                        .sum::<u64>(),
                ),
            DerivedIndexKind::RelationJoin(_) => 0,
        };
        (self.name.capacity() as u64).saturating_add(kind_bytes)
    }
}
