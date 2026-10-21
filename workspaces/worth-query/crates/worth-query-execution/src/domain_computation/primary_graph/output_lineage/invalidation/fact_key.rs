use worth_foundational::facade::{AspectKey, CanonicalFieldPath};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    indexes::DerivedIndexId,
    storage::AuthoritativeFieldComparisonKey,
};

use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection;

/// Exact reverse-posting selectors. These are derived addresses, not facts or
/// revision evidence; a match identifies the retained fact to verify.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum FactPostingKey {
    SourcePublication(
        std::sync::Arc<
            crate::domain_computation::primary_graph::application_contribution::OutputSourceAddress,
        >,
    ),
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
}
