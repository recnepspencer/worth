use worth_foundational::facade::{AspectKey, CanonicalFieldPath};
use worth_relational::facade::{
    identity::{EntityId, KindId, LineageId},
    indexes::DerivedIndexId,
    storage::AuthoritativeFieldComparisonKey,
};

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;

/// Exact reverse-posting selectors. These are derived addresses, not facts or
/// revision evidence; a match identifies the retained fact to verify.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FactPostingKey {
    SourcePublication(
        std::sync::Arc<
            crate::domain_computation::primary_graph::application_contribution::OutputSourceAddress,
        >,
    ),
    /// A head of this source's output family changed: every `current_output`
    /// selection from the family re-resolves (public #429).
    FamilySelection(crate::domain_computation::primary_graph::output_lineage::SemanticSource),
    EntityLifecycle(EntityId),
    AspectRevision {
        entity: EntityId,
        aspect: AspectKey,
    },
    FieldRevision {
        entity: EntityId,
        aspect: AspectKey,
        path: CanonicalFieldPath,
    },
    RelationMembership {
        kind: KindId,
        from: EntityId,
        to: EntityId,
    },
    Adjacency {
        kind: KindId,
        anchor: EntityId,
        direction: WorthQueryApplicationAdjacencyDirection,
    },
    IndexMembership {
        index: DerivedIndexId,
        kind: KindId,
        aspect: AspectKey,
        path: CanonicalFieldPath,
        key: AuthoritativeFieldComparisonKey,
    },
    IndexDefinition(DerivedIndexId),
    /// A committed lineage event ended, split, merged away, replaced or
    /// restored from this lineage.
    PersistentNameLineage(LineageId),
    /// A committed lineage event birthed the name with this digest.
    PersistentNameBirth([u8; 32]),
}
