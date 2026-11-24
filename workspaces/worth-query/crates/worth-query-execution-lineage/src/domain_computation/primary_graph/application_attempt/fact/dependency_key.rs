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
            ) => {
                ae == be
                    && al.aspect().aspect_key() == bl.aspect().aspect_key()
                    && al.field_path().fields().first() == bl.field_path().fields().first()
            }
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
            _ => false,
        }
    }
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

    #[test]
    fn structural_source_merge_key_matches_dependency_key_for_same_variant_facts() {
        let e1 = EntityId::new(PartitionId::main(), 1, 1);
        let e2 = EntityId::new(PartitionId::main(), 2, 1);
        let loc_a = AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            AspectKey::new("task").expect("valid aspect key"),
            CanonicalFieldPath::single(FieldKey::new("status").expect("valid field key")),
        );
        let loc_b = AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            AspectKey::new("task").expect("valid aspect key"),
            CanonicalFieldPath::single(FieldKey::new("due").expect("valid field key")),
        );
        let f1 = Fact::SourceFieldRevision {
            entity_id: e1,
            locator: loc_a.clone(),
            native_revision: None,
        };
        let f2 = Fact::SourceFieldRevision {
            entity_id: e1,
            locator: loc_a,
            native_revision: None,
        };
        let f3 = Fact::SourceFieldRevision {
            entity_id: e1,
            locator: loc_b,
            native_revision: None,
        };
        assert!(f1.same_source_merge_key(&f2));
        assert_eq!(f1.dependency_key(), f2.dependency_key());
        assert!(!f1.same_source_merge_key(&f3));
        assert_ne!(f1.dependency_key(), f3.dependency_key());

        let adj1 = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(9),
            anchor: e1,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(1)),
            comparison_work_limit: 4,
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
        };
        let adj2 = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(9),
            anchor: e1,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(2)),
            comparison_work_limit: 8,
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
        };
        let adj3 = Fact::SourceAdjacencyRevision {
            relation_kind: KindId::new(9),
            anchor: e2,
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: Some(VersionId(1)),
            comparison_work_limit: 4,
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
        };
        assert!(adj1.same_source_merge_key(&adj2));
        assert!(!adj1.same_source_merge_key(&adj3));
        assert!(!adj1.same_source_merge_key(&f1));
    }
}
