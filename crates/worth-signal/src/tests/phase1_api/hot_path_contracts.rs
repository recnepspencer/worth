//! Hot-path access rules that no behavioral test enforces in the default lane:
//! the runtime access-counter budgets that would catch these run only in the
//! ignored performance profiles. Each guard forbids a broad accessor on a named
//! hot-path owner; none pins the owner's ordinary spelling.
use super::source_corpus::{
    ENTRY_TRANSITIONS_SOURCE, HOT_APPLY_SOURCE, HOT_CONTEXT_SOURCE, HOT_EFFECT_SOURCE,
    HOT_INVALIDATION_ROUTING_SOURCE, HOT_INVALIDATION_SUBSCRIPTION_SOURCE, HOT_PLANNING_SOURCE,
    HOT_PRECOMPUTE_SOURCE, HOT_PREPARED_APPLY_SOURCE, HOT_REUSE_CONTEXT_SOURCE,
    HOT_SEMANTIC_FINALIZE_SOURCE, HOT_SERIAL_BATCH_SOURCE, HOT_STAGE_SOURCE,
    HOT_TRANSACTION_OBSERVATION_MUTATION_SOURCE, HOT_VALIDATION_SOURCE,
    INVALIDATION_AUTHORITY_SOURCE, INVALIDATION_REVALIDATION_SOURCE,
    ORDINARY_GRAPH_MUTATION_ACCESS_SOURCE, ORDINARY_INVALIDATION_ACCESS_SOURCE,
};

const BROAD_ENTRY: &[&str] = &["get_entry(", "get_entry_mut("];
const BROAD_ARTIFACT_STATE: &[&str] = &["RuntimeArtifactState", "node_runtime_artifact_state("];

fn assert_forbids(owner: &str, source: &str, forbidden: &[&str]) {
    for token in forbidden {
        assert!(
            !source.contains(token),
            "{owner} must not use broad accessor {token} on the hot path"
        );
    }
}

#[test]
fn hot_path_owners_forbid_broad_entry_access() {
    for (owner, source) in [
        ("apply", HOT_APPLY_SOURCE),
        ("prepared_apply", HOT_PREPARED_APPLY_SOURCE),
        ("semantic_finalize", HOT_SEMANTIC_FINALIZE_SOURCE),
        ("serial_batch", HOT_SERIAL_BATCH_SOURCE),
        ("stage", HOT_STAGE_SOURCE),
        ("planning", HOT_PLANNING_SOURCE),
        ("planning_validation", HOT_VALIDATION_SOURCE),
        ("precompute", HOT_PRECOMPUTE_SOURCE),
        ("runtime_effect", HOT_EFFECT_SOURCE),
        (
            "invalidation_subscription",
            HOT_INVALIDATION_SUBSCRIPTION_SOURCE,
        ),
        ("invalidation_routing", HOT_INVALIDATION_ROUTING_SOURCE),
        ("evaluation_context", HOT_CONTEXT_SOURCE),
        ("reuse_context", HOT_REUSE_CONTEXT_SOURCE),
    ] {
        assert_forbids(owner, source, BROAD_ENTRY);
    }
}

#[test]
fn hot_finalize_and_validation_owners_forbid_broad_runtime_artifact_state() {
    for (owner, source) in [
        ("semantic_finalize", HOT_SEMANTIC_FINALIZE_SOURCE),
        ("serial_batch", HOT_SERIAL_BATCH_SOURCE),
        ("stage", HOT_STAGE_SOURCE),
        ("planning_validation", HOT_VALIDATION_SOURCE),
    ] {
        assert_forbids(owner, source, BROAD_ARTIFACT_STATE);
    }
    assert_forbids(
        "runtime_effect",
        HOT_EFFECT_SOURCE,
        &["node_runtime_artifact_state("],
    );
}

#[test]
fn source_routing_does_not_premark_subscribers_pending() {
    assert!(
        !HOT_INVALIDATION_ROUTING_SOURCE.contains("transition_node_pending_revalidation("),
        "source routing must not pre-mark subscribers pending before a committed output delta"
    );
}

#[test]
fn transaction_observation_staging_does_not_snapshot_matching_observers() {
    assert!(
        !HOT_TRANSACTION_OBSERVATION_MUTATION_SOURCE.contains("matching_observers_for_node(node)"),
        "transaction observation staging should not allocate matching-observer snapshots on the hot path"
    );
}

#[test]
fn reported_stage_outcomes_never_read_the_physical_high_water_mark() {
    // A shared authority's high-water mark moves with other leases, so it is
    // evidence only; the resolved posture alone decides what a stage reports.
    for (owner, source) in [
        ("semantic_finalize", HOT_SEMANTIC_FINALIZE_SOURCE),
        ("stage", HOT_STAGE_SOURCE),
        ("precompute", HOT_PRECOMPUTE_SOURCE),
    ] {
        assert!(
            !source.contains("active_workers_high_watermark"),
            "{owner} should decide parallel outcomes from the resolved posture"
        );
    }
}

#[test]
fn ordinary_graph_mutation_and_invalidation_sources_forbid_broad_entry_materialization() {
    for (owner, source) in [
        ("invalidation_authority", INVALIDATION_AUTHORITY_SOURCE),
        ("graph_mutation", ORDINARY_GRAPH_MUTATION_ACCESS_SOURCE),
        ("invalidation", ORDINARY_INVALIDATION_ACCESS_SOURCE),
    ] {
        assert_forbids(owner, source, BROAD_ENTRY);
    }
    assert!(!INVALIDATION_AUTHORITY_SOURCE.contains("NodeEntry"));
    assert!(!INVALIDATION_AUTHORITY_SOURCE.contains("materialize_entry"));

    // Ratchet: the named residual broad seams may shrink, never grow.
    for (owner, source, token, retained) in [
        (
            "entry transitions",
            ENTRY_TRANSITIONS_SOURCE,
            "get_entry_mut(",
            1,
        ),
        (
            "entry transitions",
            ENTRY_TRANSITIONS_SOURCE,
            "get_entry(",
            0,
        ),
        (
            "revalidation",
            INVALIDATION_REVALIDATION_SOURCE,
            "get_entry(",
            2,
        ),
        (
            "revalidation",
            INVALIDATION_REVALIDATION_SOURCE,
            "get_entry_mut(",
            1,
        ),
    ] {
        assert!(
            source.matches(token).count() <= retained,
            "{owner} gained a broad {token} seam beyond its {retained} named residuals"
        );
    }
}
