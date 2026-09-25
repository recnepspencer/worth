use std::collections::BTreeMap;
use std::sync::Arc;

use crate::indexes::data::DerivedIndexDefinition;

/// Detached restored-definition inventory for bulk exact semantic matching.
/// Numeric IDs come from the owner, never from the caller's expected shape.
pub struct DerivedIndexDefinitionLookup {
    by_name: BTreeMap<String, Vec<Arc<DerivedIndexDefinition>>>,
    definition_count: usize,
}

impl DerivedIndexDefinitionLookup {
    pub(super) fn new(definitions: Vec<Arc<DerivedIndexDefinition>>) -> Self {
        let definition_count = definitions.len();
        let mut by_name = BTreeMap::new();
        for definition in definitions {
            by_name
                .entry(definition.name.clone())
                .or_insert_with(Vec::new)
                .push(definition);
        }
        Self {
            by_name,
            definition_count,
        }
    }

    pub fn definition_count(&self) -> usize {
        self.definition_count
    }

    pub fn candidate_count_for_name(&self, name: &str) -> usize {
        self.by_name.get(name).map_or(0, Vec::len)
    }

    pub fn matching_definition(
        &self,
        expected: &DerivedIndexDefinition,
    ) -> Option<DerivedIndexDefinition> {
        self.by_name
            .get(&expected.name)?
            .iter()
            .find_map(|candidate| {
                (candidate.kind == expected.kind
                    && candidate.branch_scoped == expected.branch_scoped)
                    .then(|| candidate.as_ref().clone())
            })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::DerivedIndexDefinitionLookup;
    use crate::indexes::data::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};
    use crate::tests::support::{aspect_field_locator, aspect_key, field_key};

    #[test]
    fn exact_lookup_preserves_restored_numeric_identity_and_disambiguates_name() {
        let kind = DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        };
        let wrong_kind = DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("other")),
        };
        let installed = |id, kind, branch_scoped| DerivedIndexDefinition {
            index_id: DerivedIndexId(id),
            name: "same-name".to_owned(),
            kind,
            branch_scoped,
        };
        let lookup = DerivedIndexDefinitionLookup::new(vec![
            Arc::new(installed(11, wrong_kind.clone(), false)),
            Arc::new(installed(12, kind.clone(), true)),
            Arc::new(installed(13, kind.clone(), false)),
            Arc::new(installed(14, kind.clone(), false)),
        ]);
        assert_eq!(lookup.definition_count(), 4);
        assert_eq!(lookup.candidate_count_for_name("same-name"), 4);
        assert_eq!(lookup.candidate_count_for_name("missing"), 0);

        assert_eq!(
            lookup
                .matching_definition(&installed(0, kind.clone(), false))
                .unwrap()
                .index_id,
            DerivedIndexId(13)
        );
        assert_eq!(
            lookup
                .matching_definition(&installed(0, kind.clone(), true))
                .unwrap()
                .index_id,
            DerivedIndexId(12)
        );
        assert!(lookup
            .matching_definition(&installed(0, wrong_kind, true))
            .is_none());
        assert!(lookup
            .matching_definition(&DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: "missing".to_owned(),
                kind,
                branch_scoped: false,
            })
            .is_none());
    }

    #[test]
    fn lookup_snapshot_is_detached_from_subsequent_owner_registration() {
        let runtime = crate::tests::support::persisted_runtime_with_test_schema();
        let expected = DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "snapshot-index".to_owned(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: false,
        };
        let installed = runtime.index_authority().register(expected.clone());
        let snapshot = runtime.index_access().definition_lookup_snapshot();
        runtime.index_authority().register(DerivedIndexDefinition {
            name: "later-index".to_owned(),
            ..expected.clone()
        });

        assert_eq!(snapshot.definition_count(), 1);
        assert_eq!(snapshot.candidate_count_for_name("snapshot-index"), 1);
        assert_eq!(snapshot.candidate_count_for_name("later-index"), 0);
        assert_eq!(
            snapshot.matching_definition(&expected).unwrap().index_id,
            installed.index_id
        );
    }
}
