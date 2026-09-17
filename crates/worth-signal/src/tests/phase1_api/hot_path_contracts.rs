use super::source_corpus::{
    HOT_APPLY_SOURCE, HOT_CONTEXT_SOURCE, HOT_EFFECT_SOURCE, HOT_INVALIDATION_ROUTING_SOURCE,
    HOT_INVALIDATION_SUBSCRIPTION_SOURCE, HOT_PLANNING_SOURCE, HOT_PRECOMPUTE_SOURCE,
    HOT_PREPARED_APPLY_SOURCE, HOT_REUSE_CONTEXT_SOURCE, HOT_SEMANTIC_FINALIZE_SOURCE,
    HOT_SERIAL_BATCH_SOURCE, HOT_STAGE_SOURCE, HOT_VALIDATION_SOURCE, PERFORMANCE_PROFILES_SOURCE,
    PERFORMANCE_SUPPORT_SOURCE,
};

#[test]
fn hot_apply_modules_do_not_use_broad_entry_accessors_for_reads() {
    for (name, source) in [
        ("apply", HOT_APPLY_SOURCE),
        ("prepared_apply", HOT_PREPARED_APPLY_SOURCE),
        ("semantic_finalize", HOT_SEMANTIC_FINALIZE_SOURCE),
        ("serial_batch", HOT_SERIAL_BATCH_SOURCE),
        ("planning", HOT_PLANNING_SOURCE),
        ("planning_validation", HOT_VALIDATION_SOURCE),
        ("precompute", HOT_PRECOMPUTE_SOURCE),
    ] {
        assert!(
            !source.contains("get_entry("),
            "{name} should use narrowed graph accessors instead of broad get_entry reads"
        );
        assert!(
            !source.contains("get_entry_mut("),
            "{name} should not require broad mutable entry access on the read-path seam"
        );
    }
}

#[test]
fn perf_harness_supports_hot_family_access_counter_budgets() {
    assert!(
        PERFORMANCE_SUPPORT_SOURCE.contains("access_counter_maxima"),
        "performance support should encode explicit access-counter budgets for hot families"
    );
    assert!(
        PERFORMANCE_SUPPORT_SOURCE
            .contains("for (counter, maximum) in contract.access_counter_maxima"),
        "performance support should certify access-counter maxima as part of perf-case enforcement"
    );
}

#[test]
fn hot_perf_families_forbid_broad_entry_access() {
    for suite in [
        "topology_rewiring_churn",
        "topology_rewiring_rotating_window",
        "chain_10k_bootstrap",
        "suppression_wide_fanout",
    ] {
        assert!(
            PERFORMANCE_PROFILES_SOURCE.contains(&format!("\"{suite}\"")),
            "{suite} perf family should remain source-visible in the cert profile file"
        );
    }
    assert!(
        PERFORMANCE_PROFILES_SOURCE.contains("ZERO_BROAD_ENTRY_ACCESS"),
        "perf profiles should define an explicit zero-broad-entry budget for narrowed hot families"
    );
    assert!(
        PERFORMANCE_PROFILES_SOURCE.contains("ZERO_BROAD_AND_ARTIFACT_ACCESS"),
        "perf profiles should define an explicit zero-broad-and-artifact budget for already-clean topology families"
    );
    assert!(
        PERFORMANCE_PROFILES_SOURCE.contains("hot_family_contract("),
        "hot perf families should use explicit hot-family contracts instead of generic perf contracts"
    );
    assert!(
        PERFORMANCE_PROFILES_SOURCE.contains("\"suppression_wide_fanout\"")
            && PERFORMANCE_PROFILES_SOURCE.contains("SignalRuntimePolicy::operational().with_history_limit(4)"),
        "suppression perf cert should run under explicit operational policy instead of paying development-mode diagnostic retention by default"
    );
}

#[test]
fn observability_perf_profiles_use_structural_only_certification() {
    assert!(
        PERFORMANCE_SUPPORT_SOURCE.contains("PerfTimingPolicy::StructuralOnly"),
        "performance support should expose a structural-only cert mode for rich observability workloads"
    );
    assert!(
        PERFORMANCE_SUPPORT_SOURCE
            .contains("if !matches!(contract.timing_policy, PerfTimingPolicy::StructuralOnly)"),
        "structural-only perf cases should skip timing-phase regression gates"
    );
    assert!(
        PERFORMANCE_PROFILES_SOURCE.contains("\"harness_observability_profile\"")
            && PERFORMANCE_PROFILES_SOURCE.contains("PerfTimingPolicy::StructuralOnly"),
        "observability perf profiles should certify structural/resource behavior without hard timing gating"
    );
}

#[test]
fn maybe_stale_validation_path_uses_narrowed_hot_accessors() {
    assert!(
        !HOT_VALIDATION_SOURCE.contains("get_entry("),
        "maybe-stale validation should not reintroduce broad entry reads"
    );
    assert!(
        !HOT_VALIDATION_SOURCE.contains("RuntimeArtifactState"),
        "maybe-stale validation should rely on hot artifact truth rather than broad runtime artifact state"
    );
    assert!(
        HOT_VALIDATION_SOURCE.contains("node_runtime_artifact_hot("),
        "maybe-stale validation should inspect changed scopes through the hot artifact lane"
    );
}

#[test]
fn hot_effect_runtime_path_avoids_broad_entry_reads() {
    assert!(
        !HOT_EFFECT_SOURCE.contains("get_entry("),
        "runtime effect hot path should not use broad get_entry reads"
    );
    assert!(
        !HOT_EFFECT_SOURCE.contains("node_runtime_artifact_state("),
        "runtime effect hot path should inspect partition scope changes through the hot artifact lane"
    );
    assert_eq!(
        HOT_EFFECT_SOURCE.matches("get_entry_mut(").count(),
        0,
        "runtime effect hot path should mutate through named graph transitions instead of broad mutable entry access"
    );
    assert!(
        HOT_EFFECT_SOURCE.contains("node_runtime_artifact_structural_state("),
        "runtime effect should derive previous lineage/hash/reuse truth through a narrowed graph accessor"
    );
    assert!(
        HOT_EFFECT_SOURCE.contains("apply_artifact_write("),
        "runtime effect should publish runtime and retained artifact writes through a named graph operation"
    );
    assert!(
        HOT_EFFECT_SOURCE.contains("transition_clean("),
        "runtime effect suppression should clean nodes through a named graph transition"
    );
}

#[test]
fn invalidation_subscription_path_uses_narrowed_config_access() {
    assert!(
        !HOT_INVALIDATION_SUBSCRIPTION_SOURCE.contains("get_entry("),
        "subscription invalidation should not materialize broad node entries for partition policy checks"
    );
    assert!(
        HOT_INVALIDATION_SUBSCRIPTION_SOURCE.contains("node_eval_config("),
        "subscription invalidation should inspect partitioned-output policy through narrowed config access"
    );
}

#[test]
fn execution_context_and_reuse_paths_use_narrowed_graph_accessors() {
    for (name, source) in [
        ("context", HOT_CONTEXT_SOURCE),
        ("reuse_context", HOT_REUSE_CONTEXT_SOURCE),
    ] {
        assert!(
            !source.contains("get_entry("),
            "{name} should not rely on broad entry reads for execution-time version or config access"
        );
    }
    assert!(
        HOT_CONTEXT_SOURCE.contains("node_aspect_version("),
        "evaluation context should read aspect versions through the narrowed graph accessor"
    );
    assert!(
        HOT_CONTEXT_SOURCE.contains("node_partitioned_aspect_version("),
        "evaluation context should read partitioned versions through the narrowed graph accessor"
    );
    assert!(
        HOT_REUSE_CONTEXT_SOURCE.contains("node_eval_config("),
        "reuse boundary resolution should derive comparator/config truth through the named graph accessor"
    );
}

#[test]
fn invalidation_routing_uses_named_node_transitions() {
    assert!(
        !HOT_INVALIDATION_ROUTING_SOURCE.contains("get_entry("),
        "invalidation routing should not use broad entry reads"
    );
    assert!(
        !HOT_INVALIDATION_ROUTING_SOURCE.contains("get_entry_mut("),
        "invalidation routing should mutate node state through named graph transitions"
    );
    assert!(
        HOT_INVALIDATION_ROUTING_SOURCE.contains("transition_node_dirty("),
        "invalidation routing should use the named dirty transition"
    );
    assert!(
        !HOT_INVALIDATION_ROUTING_SOURCE.contains("transition_node_pending_revalidation("),
        "source routing must not pre-mark subscribers pending before a committed output delta"
    );
}

#[test]
fn hot_stage_path_avoids_broad_entry_reads() {
    assert!(
        !HOT_STAGE_SOURCE.contains("get_entry("),
        "stage lowering should use narrowed graph accessors instead of broad get_entry reads"
    );
    assert!(
        !HOT_STAGE_SOURCE.contains("get_entry_mut("),
        "stage lowering should not require broad mutable entry access on the read-path seam"
    );
}
