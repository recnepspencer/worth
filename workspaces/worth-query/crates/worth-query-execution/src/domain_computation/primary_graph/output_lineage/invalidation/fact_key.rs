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

impl FactPostingKey {
    /// Owned key payload copied into a posting/index cell. Callers admit the
    /// field-path traversal before asking for this bound, then admit the
    /// returned bytes before cloning the key.
    pub(super) fn owned_payload_capacity_bytes(&self) -> Option<u64> {
        let (aspect, path, value) = match self {
            Self::AspectRevision { aspect, .. } => (Some(aspect), None, None),
            Self::FieldRevision { aspect, path, .. } => (Some(aspect), Some(path), None),
            Self::IndexMembership {
                aspect, path, key, ..
            } => (Some(aspect), Some(path), Some(key)),
            Self::EntityLifecycle(_)
            | Self::RelationMembership { .. }
            | Self::Adjacency { .. }
            | Self::IndexDefinition(_) => (None, None, None),
        };
        let aspect = aspect.map_or(0, AspectKey::owned_allocation_capacity_bytes);
        let path = path.map_or(0, CanonicalFieldPath::owned_allocation_capacity_bytes);
        let value = value.map_or(
            0,
            AuthoritativeFieldComparisonKey::owned_allocation_capacity_bytes,
        );
        u64::try_from(aspect)
            .ok()?
            .checked_add(u64::try_from(path).ok()?)?
            .checked_add(value)
    }

    /// Bound one ordered comparison by the fields it can visit and the
    /// initialized bytes it can compare. Spare String/Vec capacity and the
    /// enum's largest variant are retained memory, not comparison work.
    /// The caller admits field-path traversal before computing this bound.
    pub(super) fn comparison_work_bound(&self) -> Option<u64> {
        fn text(value: &str) -> Option<u64> {
            u64::try_from(value.len()).ok()?.checked_add(1)
        }

        fn path(value: &CanonicalFieldPath) -> Option<u64> {
            value
                .fields()
                .iter()
                .try_fold(1_u64, |work, field| work.checked_add(text(field.as_str())?))
        }

        // One visit for the discriminant and each fixed scalar. EntityId has
        // partition, slot and generation; its phantom domain has no work.
        match self {
            Self::EntityLifecycle(_) => Some(4),
            Self::IndexDefinition(_) => Some(2),
            Self::RelationMembership { .. } => Some(8),
            Self::Adjacency { .. } => Some(6),
            Self::AspectRevision { aspect, .. } => 4_u64.checked_add(text(aspect.as_str())?),
            Self::FieldRevision {
                aspect,
                path: fields,
                ..
            } => 4_u64
                .checked_add(text(aspect.as_str())?)?
                .checked_add(path(fields)?),
            Self::IndexMembership {
                aspect,
                path: fields,
                key,
                ..
            } => 3_u64
                .checked_add(text(aspect.as_str())?)?
                .checked_add(path(fields)?)?
                .checked_add(u64::try_from(key.canonical_value_bytes().len()).ok()?)?
                .checked_add(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};
    use worth_relational::facade::identity::{EntityId, PartitionId};

    use super::FactPostingKey;

    #[test]
    fn fixed_entity_lifecycle_does_not_pay_for_unvisited_variant_storage() {
        let key = FactPostingKey::EntityLifecycle(EntityId::new(PartitionId::main(), 1, 1));
        assert_eq!(key.comparison_work_bound(), Some(4));
        assert!(std::mem::size_of::<FactPostingKey>() > 4);
    }

    #[test]
    fn spare_field_key_capacity_does_not_inflate_comparison_work() {
        let entity = EntityId::new(PartitionId::main(), 1, 1);
        let tight = FactPostingKey::FieldRevision {
            entity,
            aspect: AspectKey::new("a").unwrap(),
            path: CanonicalFieldPath::single(FieldKey::new("b").unwrap()),
        };
        let mut spare_aspect = String::with_capacity(4096);
        spare_aspect.push('a');
        let mut spare_field = String::with_capacity(4096);
        spare_field.push('b');
        let spare = FactPostingKey::FieldRevision {
            entity,
            aspect: AspectKey::new(spare_aspect).unwrap(),
            path: CanonicalFieldPath::single(FieldKey::new(spare_field).unwrap()),
        };
        assert_eq!(tight, spare);
        assert_eq!(tight.comparison_work_bound(), spare.comparison_work_bound());
        assert!(
            spare.owned_payload_capacity_bytes().unwrap()
                > tight.owned_payload_capacity_bytes().unwrap()
        );
    }
}
