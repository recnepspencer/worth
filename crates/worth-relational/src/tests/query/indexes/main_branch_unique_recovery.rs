use super::*;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::validation::data::{InvariantCatalog, InvariantRegistration, InvariantRule};

#[test]
fn unique_index_rebuild_uses_main_head_when_a_feature_commit_is_globally_newer() {
    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
        )],
    });
    let main_entity = create_entity(&runtime, "main-only");
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .expect("feature branch forks");
    let feature_outcome =
        create_entity_outcome_on_branch(&runtime, "feature-only", BranchId("feature".to_string()));
    let feature_entity = changed_entities(&feature_outcome)[0];

    runtime
        .index_authority()
        .rebuild_unique_entity_aspect_field_indexes()
        .expect("rebuild admits the configured main branch head");

    let name_field = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index_access = runtime.index_access();
    let entries = index_access
        .entity_unique_field_entries(&name_field)
        .expect("unique name entries are rebuilt");
    assert_eq!(
        entries
            .get(&comparison_key("main-only"))
            .cloned()
            .unwrap_or_default(),
        std::collections::BTreeSet::from([main_entity]),
    );
    assert!(!entries.contains_key(&comparison_key("feature-only")));
    assert_ne!(main_entity, feature_entity);
    release_test_commit_snapshot(&runtime, &feature_outcome);
}

fn comparison_key(value: &str) -> AuthoritativeFieldComparisonKey {
    AuthoritativeFieldComparisonKey::from_aspect_value(&string_aspect_value(value))
}
