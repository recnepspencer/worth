use super::source_corpus::{
    ENTRIES_SOURCE, ENTRY_TRANSITIONS_SOURCE, INVALIDATION_AUTHORITY_SOURCE,
    INVALIDATION_REVALIDATION_SOURCE, ORDINARY_EXPLANATION_ACCESS_SOURCE,
    ORDINARY_GRAPH_MUTATION_ACCESS_SOURCE, ORDINARY_INVALIDATION_ACCESS_SOURCE,
};

#[test]
fn ordinary_graph_mutation_and_invalidation_sources_forbid_broad_entry_materialization() {
    for (name, source) in [
        ("invalidation_authority", INVALIDATION_AUTHORITY_SOURCE),
        ("graph_mutation", ORDINARY_GRAPH_MUTATION_ACCESS_SOURCE),
        ("invalidation", ORDINARY_INVALIDATION_ACCESS_SOURCE),
    ] {
        assert!(
            !source.contains("get_entry("),
            "{name} must not assemble NodeEntry values for ordinary reads"
        );
        assert!(
            !source.contains("get_entry_mut("),
            "{name} must not write through materialized NodeEntry guards"
        );
    }

    assert_eq!(
        ENTRY_TRANSITIONS_SOURCE.matches("get_entry_mut(").count(),
        1,
        "entry replacement is the only retained broad transition"
    );
    assert_eq!(
        ENTRY_TRANSITIONS_SOURCE.matches("get_entry(").count(),
        0,
        "ordinary transitions must not retain a broad read"
    );
    assert!(ENTRY_TRANSITIONS_SOURCE.contains("pub(crate) fn replace_entry("));
    assert_eq!(
        INVALIDATION_REVALIDATION_SOURCE
            .matches("get_entry(")
            .count(),
        2,
        "checkpoint readmission and the test-only injector are the named residual reads"
    );
    assert_eq!(
        INVALIDATION_REVALIDATION_SOURCE
            .matches("get_entry_mut(")
            .count(),
        1,
        "only the test-only malformed-cause injector retains broad mutation"
    );
    assert!(INVALIDATION_REVALIDATION_SOURCE.contains("fn readmit_checkpoint_causes("));
    assert!(
        INVALIDATION_REVALIDATION_SOURCE.contains("fn inject_pending_causes_unchecked_for_test(")
    );
}

#[test]
fn invalidation_authority_stays_component_owned_and_responsibility_specific() {
    for accessor in [
        "node_dependency_revision(",
        "node_pending_cause_set_id(",
        "node_direct_invalidation_basis(",
        "node_direct_invalidation_generation(",
        "node_dirty_partition_scope_payload(",
        "node_pending_revalidation(",
        "set_node_pending_cause_set_id(",
        "advance_node_dependency_revision(",
        "replace_node_invalidation_cache(",
        "install_node_dependency_revalidation(",
    ] {
        let definition = format!("fn {accessor}");
        assert!(
            INVALIDATION_AUTHORITY_SOURCE.contains(definition.as_str()),
            "component owner must retain named seam {accessor}"
        );
        assert!(
            ORDINARY_GRAPH_MUTATION_ACCESS_SOURCE.contains(accessor)
                || ORDINARY_INVALIDATION_ACCESS_SOURCE.contains(accessor)
                || INVALIDATION_REVALIDATION_SOURCE.contains(accessor)
                || ENTRY_TRANSITIONS_SOURCE.contains(accessor),
            "a real graph path must consume named component seam {accessor}"
        );
    }
    assert!(!INVALIDATION_AUTHORITY_SOURCE.contains("NodeEntry"));
    assert!(!INVALIDATION_AUTHORITY_SOURCE.contains("materialize_entry"));
}

#[test]
fn retained_explanation_validation_forbids_broad_entry_materialization() {
    assert!(ENTRIES_SOURCE.contains("fn node_explanation_storage_view("));
    assert!(
        ORDINARY_EXPLANATION_ACCESS_SOURCE.contains("graph.node_explanation_storage_view(node)?")
    );
    assert!(!ORDINARY_EXPLANATION_ACCESS_SOURCE.contains("get_entry("));
    assert!(!ORDINARY_EXPLANATION_ACCESS_SOURCE.contains("materialize_entry"));
}
