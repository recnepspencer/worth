struct DirtyEvidenceBoundary {
    owner: &'static str,
    source: &'static str,
    required_symbols: &'static [&'static str],
}

const BOUNDARIES: &[DirtyEvidenceBoundary] = &[
    DirtyEvidenceBoundary {
        owner: "async admission evidence model",
        source: include_str!("../../../../data/async_node/admission.rs"),
        required_symbols: &["dirty_aspects", "dirty_partition_scope_count"],
    },
    DirtyEvidenceBoundary {
        owner: "source-only invalidation routing",
        source: include_str!("../../../../logic/invalidation/routing/application.rs"),
        required_symbols: &["execute_invalidation_frontier", "mark_source_seed"],
    },
    DirtyEvidenceBoundary {
        owner: "planner topology admission",
        source: include_str!("../../../../logic/planner/planning/topology.rs"),
        required_symbols: &["NodeState::Dirty", "NodeState::MaybeStale"],
    },
    DirtyEvidenceBoundary {
        owner: "ordinary condition eligibility",
        source: include_str!("../../../../logic/planner/precompute/eligibility.rs"),
        required_symbols: &["node_invalidation_input", "NodeInvalidationInput"],
    },
    DirtyEvidenceBoundary {
        owner: "installed condition resolution",
        source: include_str!("../../../../data/conditional_execution/condition_resolution.rs"),
        required_symbols: &["node_invalidation_input", "NodeInvalidationInput"],
    },
    DirtyEvidenceBoundary {
        owner: "node storage layout",
        source: include_str!("../../../../data/node/entry/layout.rs"),
        required_symbols: &["dirty_aspects", "dirty_partition_scope_payload"],
    },
    DirtyEvidenceBoundary {
        owner: "node entry transitions",
        source: include_str!("../../../../data/node/entry/state_transitions.rs"),
        required_symbols: &["dirty_aspects", "merge_dirty_partition_scopes"],
    },
    DirtyEvidenceBoundary {
        owner: "node contract projection",
        source: include_str!("../../../../data/node/contract.rs"),
        required_symbols: &["reads_dirty_aspects"],
    },
    DirtyEvidenceBoundary {
        owner: "graph storage access and capture",
        source: include_str!("../../../../data/graph/storage/entries/access.rs"),
        required_symbols: &["node_dirty_aspects", "dirty_partition_scopes"],
    },
    DirtyEvidenceBoundary {
        owner: "graph storage transitions",
        source: include_str!("../../../../data/graph/storage/entries/transitions.rs"),
        required_symbols: &["dirty_aspects", "merge_dirty_partition_scopes"],
    },
    DirtyEvidenceBoundary {
        owner: "graph invalidation authority",
        source: include_str!("../../../../data/graph/storage/entries/invalidation_authority.rs"),
        required_symbols: &[
            "node_direct_invalidation_basis",
            "node_dirty_partition_scope_payload",
            "replace_node_invalidation_cache",
            "install_node_dependency_revalidation",
        ],
    },
    DirtyEvidenceBoundary {
        owner: "graph diagnostic scan",
        source: include_str!("../../../../data/graph/storage/diagnostic_scan.rs"),
        required_symbols: &[
            "diagnostic_nodes",
            "node_runtime_artifact_state_present",
            "execution_record_present",
            "causality_present",
        ],
    },
    DirtyEvidenceBoundary {
        owner: "node explanation storage view",
        source: include_str!("../../../../data/graph/storage/entries/diagnostic_artifacts.rs"),
        required_symbols: &[
            "node_explanation_storage_view",
            "historical_artifact_record",
            "trace_summary",
        ],
    },
    DirtyEvidenceBoundary {
        owner: "graph deserialization shape validation",
        source: include_str!("../../../../data/graph/runtime/graph.rs"),
        required_symbols: &["validate_deserialized_lane_alignment"],
    },
    DirtyEvidenceBoundary {
        owner: "diagnostic graph summary projection",
        source: include_str!("../../../../diagnostics/model/summary/graph.rs"),
        required_symbols: &["diagnostic_nodes", "execution_record_present"],
    },
    DirtyEvidenceBoundary {
        owner: "node checkpoint conversion",
        source: include_str!("../../../../data/node/entry/checkpoint.rs"),
        required_symbols: &["dirty_aspects", "dirty_partition_scopes"],
    },
    DirtyEvidenceBoundary {
        owner: "checkpoint image persistence",
        source: include_str!("../../../../data/node/checkpoint_image.rs"),
        required_symbols: &["dirty_aspects", "dirty_partition_scopes"],
    },
    DirtyEvidenceBoundary {
        owner: "graph checkpoint authority",
        source: include_str!("../../../../data/graph/runtime/graph/checkpoint.rs"),
        required_symbols: &[
            "capture_checkpoint_authority",
            "restore_from_checkpoint_authority",
        ],
    },
    DirtyEvidenceBoundary {
        owner: "transaction rollback patching",
        source: include_str!("../../../../logic/transaction/patch_buffer.rs"),
        required_symbols: &["stage_original", "rollback_and_clear"],
    },
    DirtyEvidenceBoundary {
        owner: "diagnostic history inspection",
        source: include_str!("../../../../diagnostics/inspection/history.rs"),
        required_symbols: &["nodes_with_dirty_aspect", "node_dirty_aspects"],
    },
    DirtyEvidenceBoundary {
        owner: "diagnostic fact projection",
        source: include_str!("../../../../diagnostics/model/facts/projection.rs"),
        required_symbols: &["dirty_aspects"],
    },
    DirtyEvidenceBoundary {
        owner: "diagnostic summary projection",
        source: include_str!("../../../../diagnostics/model/summary/explanation.rs"),
        required_symbols: &["dirty_aspect_count"],
    },
    DirtyEvidenceBoundary {
        owner: "easy runtime dirty batching",
        source: include_str!("../../../../easy/runtime.rs"),
        required_symbols: &["mark_dirty_batch", "batched_dirty_nodes"],
    },
    DirtyEvidenceBoundary {
        owner: "evaluation condition context",
        source: include_str!("../../../../logic/evaluation/condition/context.rs"),
        required_symbols: &["dirty_aspects"],
    },
    DirtyEvidenceBoundary {
        owner: "explain analysis projection",
        source: include_str!("../../../../logic/explain/analysis.rs"),
        required_symbols: &["dirty_aspects"],
    },
    DirtyEvidenceBoundary {
        owner: "explain cause assembly",
        source: include_str!("../../../../logic/explain/resolver/assembly.rs"),
        required_symbols: &["node_explanation_storage_view", "dirty_aspects"],
    },
    DirtyEvidenceBoundary {
        owner: "explain evidence model",
        source: include_str!("../../../../logic/explain/types.rs"),
        required_symbols: &["dirty_aspects"],
    },
    DirtyEvidenceBoundary {
        owner: "explain presentation",
        source: include_str!("../../../../logic/explain/types/presentation.rs"),
        required_symbols: &["dirty_aspects"],
    },
    DirtyEvidenceBoundary {
        owner: "planner task evidence model",
        source: include_str!("../../../../logic/planner/model/task.rs"),
        required_symbols: &["dirty_partition_scopes_present"],
    },
    DirtyEvidenceBoundary {
        owner: "planner context admission",
        source: include_str!("../../../../logic/planner/planning/admission.rs"),
        required_symbols: &["node_dirty_partition_scopes_present"],
    },
    DirtyEvidenceBoundary {
        owner: "branch snapshot capture",
        source: include_str!(
            "../../../../logic/transaction/runtime/state/branching/snapshotting/capture.rs"
        ),
        required_symbols: &["capture_snapshot", "capture_checkpoint_authority"],
    },
    DirtyEvidenceBoundary {
        owner: "async admission",
        source: include_str!(
            "../../../../logic/transaction/runtime/state/async_capability/admission.rs"
        ),
        required_symbols: &["node_invalidation_input", "NodeInvalidationInput"],
    },
    DirtyEvidenceBoundary {
        owner: "async admission outcome projection",
        source: include_str!("../../../../logic/transaction/runtime/state/async_capability/mod.rs"),
        required_symbols: &["dirty_aspects", "dirty_partition_scope_count"],
    },
    DirtyEvidenceBoundary {
        owner: "dot presentation",
        source: include_str!("../../../../presentation/outputs/dot.rs"),
        required_symbols: &["node_dirty_aspects"],
    },
];

#[test]
fn phase_5_inventory_confirms_operational_dirty_readers_cut_over() {
    assert_eq!(BOUNDARIES.len(), 34, "inventory changed without review");
    for boundary in BOUNDARIES {
        for symbol in boundary.required_symbols {
            assert!(
                boundary.source.contains(symbol),
                "{} no longer exposes inventoried symbol {symbol}",
                boundary.owner
            );
        }
    }
}
