use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, CanonicalAspectValueIdentityBasis,
};

use super::WorthQueryApplicationObservedFact;

/// Storage identity for retained observations. Descriptive legacy locators
/// alone do not distinguish separate predicates against the same index.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryApplicationFactStorageKey {
    locator: String,
    predicate: Option<(CanonicalAspectValueIdentityBasis, usize)>,
}

impl WorthQueryApplicationObservedFact {
    pub fn dependency_locator_byte_bound(&self) -> Option<u64> {
        if let Self::DerivedSource { observation } = self {
            let key = observation.key();
            return u64::try_from(key.relation().len())
                .ok()?
                .checked_add(key.identity().schema.0.len() as u64)?
                .checked_add(key.identity().name.len() as u64)?
                .checked_add(512);
        }
        if let Self::StructuralWalk { observation } = self {
            // Decimal components of one initialized native ID fit within128bytes;
            // count the bound without an uncontrolled preliminary root scan.
            return u64::try_from(observation.key().roots().len())
                .ok()?
                .checked_mul(128)?
                .checked_add(observation.key().relation().len() as u64)?
                .checked_add(512);
        }
        struct Count(usize);
        impl std::fmt::Write for Count {
            fn write_str(&mut self, text: &str) -> std::fmt::Result {
                self.0 = self.0.checked_add(text.len()).ok_or(std::fmt::Error)?;
                Ok(())
            }
        }
        let mut count = Count(0);
        self.write_dependency_locator(&mut count).ok()?;
        let base = u64::try_from(count.0).ok()?;
        match self {
            Self::IndexedEntitySelection { value, .. } => base
                .checked_add(
                    (value
                        .owned_allocation_capacity_bytes()
                        .max(value.semantic_byte_width()) as u64)
                        .checked_mul(16)?,
                )?
                .checked_add(512),
            _ => base.checked_add(128),
        }
    }

    /// Exact predicate-aware key for provider ports that carry string locators.
    /// The descriptive locator and canonical dependency encoding stay unchanged.
    pub fn dependency_locator_identity(&self) -> String {
        match self {
            Self::IndexedEntitySelection {
                value,
                candidate_limit,
                ..
            } => {
                let predicate = prepare_aspect_value_identity_basis(value);
                format!(
                    "{}:predicate#{}:{}:limit#{}",
                    self.locator_identity(),
                    predicate.as_str().len(),
                    predicate.as_str(),
                    candidate_limit
                )
            }
            _ => self.locator_identity(),
        }
    }

    pub fn dependency_key(&self) -> WorthQueryApplicationFactStorageKey {
        let predicate = match self {
            Self::IndexedEntitySelection {
                value,
                candidate_limit,
                ..
            } => Some((prepare_aspect_value_identity_basis(value), *candidate_limit)),
            _ => None,
        };
        WorthQueryApplicationFactStorageKey {
            locator: self.locator_identity(),
            predicate,
        }
    }

    /// Allocation-free structural key comparison for `merge_same_source_fact`.
    /// Matches same-variant dependency identity without formatting locator strings.
    pub fn same_source_merge_key(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::SourceReadAuthorization { observation: a },
                Self::SourceReadAuthorization { observation: b },
            ) => a.key() == b.key(),
            (
                Self::ManagedIndexSource { observation: a },
                Self::ManagedIndexSource { observation: b },
            ) => a.key() == b.key(),
            (
                Self::ComponentSelection { observation: a },
                Self::ComponentSelection { observation: b },
            ) => a.key() == b.key(),
            (Self::Aggregate { observation: a }, Self::Aggregate { observation: b }) => {
                a.key() == b.key()
            }
            (Self::DerivedSource { observation: a }, Self::DerivedSource { observation: b }) => {
                a.key() == b.key()
            }
            (Self::StructuralWalk { observation: a }, Self::StructuralWalk { observation: b }) => {
                a.key() == b.key()
            }
            (
                Self::SourcePublication { observation: a },
                Self::SourcePublication { observation: b },
            ) => a.address() == b.address(),
            (
                Self::FamilySelection { observation: a },
                Self::FamilySelection { observation: b },
            ) => a.producer == b.producer,
            (
                Self::RetiredOutputEntity {
                    read_locator: a, ..
                },
                Self::RetiredOutputEntity {
                    read_locator: b, ..
                },
            ) => a == b,
            (Self::SourceEntity { entity_id: a }, Self::SourceEntity { entity_id: b }) => a == b,
            (
                Self::SourceAspectRevision {
                    entity_id: ae,
                    aspect: aa,
                    ..
                },
                Self::SourceAspectRevision {
                    entity_id: be,
                    aspect: ba,
                    ..
                },
            )
            | (
                Self::AspectPresence {
                    entity_id: ae,
                    aspect: aa,
                    ..
                },
                Self::AspectPresence {
                    entity_id: be,
                    aspect: ba,
                    ..
                },
            ) => ae == be && aa == ba,
            (
                Self::SourceFieldRevision {
                    entity_id: ae,
                    locator: al,
                    ..
                },
                Self::SourceFieldRevision {
                    entity_id: be,
                    locator: bl,
                    ..
                },
            )
            | (
                Self::Field {
                    entity_id: ae,
                    locator: al,
                    ..
                },
                Self::Field {
                    entity_id: be,
                    locator: bl,
                    ..
                },
            )
            | (
                Self::AbsentField {
                    entity_id: ae,
                    locator: al,
                    ..
                },
                Self::AbsentField {
                    entity_id: be,
                    locator: bl,
                    ..
                },
            )
            | (
                Self::OptionalAspectAbsent {
                    entity_id: ae,
                    locator: al,
                    ..
                },
                Self::OptionalAspectAbsent {
                    entity_id: be,
                    locator: bl,
                    ..
                },
            ) => ae == be && same_field_locator_bytes(al, bl),
            (
                Self::SourceAdjacencyRevision {
                    relation_kind: ar,
                    anchor: aa,
                    direction: ad,
                    ..
                },
                Self::SourceAdjacencyRevision {
                    relation_kind: br,
                    anchor: ba,
                    direction: bd,
                    ..
                },
            ) => ar == br && aa == ba && ad == bd,
            (
                Self::Entity {
                    entity_id: ae,
                    kind: ak,
                },
                Self::Entity {
                    entity_id: be,
                    kind: bk,
                },
            ) => ae == be && ak == bk,
            (
                Self::Relation {
                    relation_kind: ar,
                    from: af,
                    to: at,
                    ..
                },
                Self::Relation {
                    relation_kind: br,
                    from: bf,
                    to: bt,
                    ..
                },
            ) => ar == br && af == bf && at == bt,
            (
                Self::Adjacency {
                    relation_kind: ar,
                    anchor: aa,
                    direction: ad,
                    ..
                },
                Self::Adjacency {
                    relation_kind: br,
                    anchor: ba,
                    direction: bd,
                    ..
                },
            ) => ar == br && aa == ba && ad == bd,
            (
                Self::IndexedEntitySelection {
                    index_id: ai,
                    value: av,
                    candidate_limit: al,
                    ..
                },
                Self::IndexedEntitySelection {
                    index_id: bi,
                    value: bv,
                    candidate_limit: bl,
                    ..
                },
            ) => ai == bi && al == bl && av == bv,
            (
                Self::WorkflowDefinitionPredecessor {
                    relation_kind: ar,
                    lineage: al,
                    expected_definition: ad,
                    ..
                },
                Self::WorkflowDefinitionPredecessor {
                    relation_kind: br,
                    lineage: bl,
                    expected_definition: bd,
                    ..
                },
            ) => ar == br && al == bl && ad == bd,
            (
                Self::WorkflowDefinitionCurrent {
                    relation_kind: ar,
                    lineage: al,
                    expected_definition: ad,
                    ..
                },
                Self::WorkflowDefinitionCurrent {
                    relation_kind: br,
                    lineage: bl,
                    expected_definition: bd,
                    ..
                },
            ) => ar == br && al == bl && ad == bd,
            (
                Self::WorkflowInstanceCapacity {
                    relation_kind: ar,
                    lineage: al,
                    ..
                },
                Self::WorkflowInstanceCapacity {
                    relation_kind: br,
                    lineage: bl,
                    ..
                },
            ) => ar == br && al == bl,
            (
                Self::WorkflowHistoryBasis { instance: a, .. },
                Self::WorkflowHistoryBasis { instance: b, .. },
            ) => a == b,
            (
                Self::PersistentNameHistory { read: a, .. },
                Self::PersistentNameHistory { read: b, .. },
            ) => a == b,
            _ => false,
        }
    }
}

fn same_field_locator_bytes(
    left: &worth_foundational::facade::AspectFieldLocator,
    right: &worth_foundational::facade::AspectFieldLocator,
) -> bool {
    fn parts(loc: &worth_foundational::facade::AspectFieldLocator) -> Option<[&[u8]; 3]> {
        let field = loc.field_path().fields().first()?;
        Some([
            loc.aspect().aspect_key().as_str().as_bytes(),
            b"/",
            field.as_str().as_bytes(),
        ])
    }
    let (Some(left_parts), Some(right_parts)) = (parts(left), parts(right)) else {
        return false;
    };
    left_parts
        .into_iter()
        .flatten()
        .eq(right_parts.into_iter().flatten())
}

#[cfg(test)]
mod tests {
    use super::WorthQueryApplicationObservedFact as Fact;
    use worth_foundational::facade::{
        AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
    };
    use worth_relational::facade::{
        identity::{EntityId, KindId, PartitionId, VersionId},
        runtime::RelationalAdjacencyDirection,
    };

    fn field_fact(entity_id: EntityId, aspect: &str, field: &str) -> Fact {
        Fact::SourceFieldRevision {
            entity_id,
            locator: AspectFieldLocator::new(
                LocatorAuthority::Authoritative,
                AspectKey::new(aspect).expect("valid aspect"),
                CanonicalFieldPath::single(FieldKey::new(field).expect("valid field")),
            ),
            native_revision: None,
        }
    }

    #[test]
    fn structural_source_merge_key_matches_dependency_key_for_same_variant_facts() {
        let e1 = EntityId::new(PartitionId::main(), 1, 1);
        let e2 = EntityId::new(PartitionId::main(), 2, 1);
        let (f1, f2, f3) = (
            field_fact(e1, "task", "status"),
            field_fact(e1, "task", "status"),
            field_fact(e1, "task", "due"),
        );
        assert!(f1.same_source_merge_key(&f2) && f1.dependency_key() == f2.dependency_key());
        assert!(!f1.same_source_merge_key(&f3) && f1.dependency_key() != f3.dependency_key());
        let (sep_l, sep_r) = (field_fact(e1, "a/b", "c"), field_fact(e1, "a", "b/c"));
        assert_eq!(sep_l.dependency_key(), sep_r.dependency_key());
        assert!(sep_l.same_source_merge_key(&sep_r));

        let adj = |anchor, rev, limit| {
            Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(9),
            anchor,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(rev)),
            comparison_work_limit: limit,
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
        }
        };
        assert!(adj(e1, 1, 4).same_source_merge_key(&adj(e1, 2, 8)));
        assert!(!adj(e1, 1, 4).same_source_merge_key(&adj(e2, 1, 4)));
        assert!(!adj(e1, 1, 4).same_source_merge_key(&f1));
    }
}
