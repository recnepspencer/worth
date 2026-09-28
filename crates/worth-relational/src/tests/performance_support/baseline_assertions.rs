use super::baseline_records::{perf_baseline_elapsed_rows, perf_baseline_metric_rows};
use super::report_records::{PerfMetricSummaryRecord, PerfSummaryRecord};

#[derive(Debug, Clone, Copy)]
struct PerfCaseContract {
    elapsed_median_tolerance: f64,
    elapsed_max_tolerance: f64,
    metric_median_tolerance: f64,
    enforce_elapsed_max: bool,
    zero_baseline_metric_floor: u128,
}

impl PerfCaseContract {
    const fn standard() -> Self {
        Self {
            elapsed_median_tolerance: 1.20,
            elapsed_max_tolerance: 1.35,
            metric_median_tolerance: 1.25,
            enforce_elapsed_max: true,
            zero_baseline_metric_floor: 4,
        }
    }

    const fn noisy() -> Self {
        Self {
            elapsed_median_tolerance: 1.50,
            elapsed_max_tolerance: 1.75,
            metric_median_tolerance: 1.75,
            enforce_elapsed_max: true,
            zero_baseline_metric_floor: 8,
        }
    }

    const fn bursty() -> Self {
        Self {
            elapsed_median_tolerance: 1.75,
            elapsed_max_tolerance: 2.10,
            metric_median_tolerance: 1.90,
            enforce_elapsed_max: true,
            zero_baseline_metric_floor: 12,
        }
    }

    const fn tiny() -> Self {
        Self {
            elapsed_median_tolerance: 2.25,
            elapsed_max_tolerance: 2.50,
            metric_median_tolerance: 2.00,
            enforce_elapsed_max: true,
            zero_baseline_metric_floor: 16,
        }
    }

    const fn micro() -> Self {
        Self {
            elapsed_median_tolerance: 3.00,
            elapsed_max_tolerance: 3.50,
            metric_median_tolerance: 3.00,
            enforce_elapsed_max: true,
            zero_baseline_metric_floor: 16,
        }
    }

    const fn io_bursty() -> Self {
        Self {
            elapsed_median_tolerance: 1.90,
            elapsed_max_tolerance: 2.50,
            metric_median_tolerance: 2.00,
            enforce_elapsed_max: false,
            zero_baseline_metric_floor: 32,
        }
    }

    const fn volatile() -> Self {
        Self {
            elapsed_median_tolerance: 2.10,
            elapsed_max_tolerance: 2.50,
            metric_median_tolerance: 2.25,
            enforce_elapsed_max: false,
            zero_baseline_metric_floor: 32,
        }
    }

    const fn extreme_volatile() -> Self {
        Self {
            elapsed_median_tolerance: 2.50,
            elapsed_max_tolerance: 3.00,
            metric_median_tolerance: 3.00,
            enforce_elapsed_max: false,
            zero_baseline_metric_floor: 32,
        }
    }
}

pub(super) fn assert_elapsed_against_baseline(
    suite: &str,
    case: &str,
    summary: &PerfSummaryRecord<'_>,
) {
    if skip_baseline_asserts() {
        return;
    }
    let Some(baseline) = perf_baseline_elapsed_rows().get(&(suite.to_string(), case.to_string()))
    else {
        if allow_missing_baseline_rows() {
            return;
        }
        panic!("missing relational perf baseline for {suite}/{case}");
    };
    let contract = perf_case_contract(suite, case);

    let allowed_median = allowed_perf_regression(
        baseline.median_elapsed_micros,
        contract.elapsed_median_tolerance,
    );
    assert!(
        summary.median_elapsed_micros <= allowed_median,
        "elapsed median regressed for {suite}/{case}: observed {} > allowed {} from baseline {}",
        summary.median_elapsed_micros,
        allowed_median,
        baseline.median_elapsed_micros
    );

    if contract.enforce_elapsed_max {
        let allowed_max =
            allowed_perf_regression(baseline.max_elapsed_micros, contract.elapsed_max_tolerance);
        assert!(
            summary.max_elapsed_micros <= allowed_max,
            "elapsed max regressed for {suite}/{case}: observed {} > allowed {} from baseline {}",
            summary.max_elapsed_micros,
            allowed_max,
            baseline.max_elapsed_micros
        );
    }
}

pub(super) fn assert_metric_against_baseline(
    suite: &str,
    case: &str,
    metric_name: &str,
    summary: &PerfMetricSummaryRecord<'_>,
) {
    if skip_baseline_asserts() {
        return;
    }
    let Some(baseline) = perf_baseline_metric_rows().get(&(
        suite.to_string(),
        case.to_string(),
        metric_name.to_string(),
    )) else {
        if allow_missing_baseline_rows() {
            return;
        }
        panic!("missing relational perf baseline metric for {suite}/{case}/{metric_name}");
    };
    if profile_boundary_metric(metric_name) {
        assert!(
            summary.median == baseline.median,
            "profile boundary drift for {suite}/{case}/{metric_name}: observed {} != baseline {}",
            summary.median,
            baseline.median
        );
        return;
    }
    let contract = perf_case_contract(suite, case);
    let allowed_median = allowed_metric_regression(baseline.median, contract);
    assert!(
        summary.median <= allowed_median,
        "phase metric median regressed for {suite}/{case}/{metric_name}: observed {} > allowed {} from baseline {}",
        summary.median,
        allowed_median,
        baseline.median
    );
}

fn allow_missing_baseline_rows() -> bool {
    std::env::var("RELATIONAL_PERF_ALLOW_MISSING_BASELINE")
        .ok()
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "1" || normalized == "true" || normalized == "yes"
        })
        .unwrap_or(false)
}

fn skip_baseline_asserts() -> bool {
    std::env::var("RELATIONAL_PERF_SKIP_BASELINE_ASSERTS")
        .ok()
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "1" || normalized == "true" || normalized == "yes"
        })
        .unwrap_or(false)
}

fn allowed_perf_regression(baseline: u128, tolerance: f64) -> u128 {
    ((baseline as f64) * tolerance).ceil() as u128
}

fn allowed_metric_regression(baseline: u128, contract: PerfCaseContract) -> u128 {
    if baseline == 0 {
        contract.zero_baseline_metric_floor
    } else {
        allowed_perf_regression(baseline, contract.metric_median_tolerance)
    }
}

fn profile_boundary_metric(metric_name: &str) -> bool {
    matches!(
        metric_name,
        "profile_execution_lane_code"
            | "profile_diagnostics_boundary_code"
            | "profile_matches_defaults"
    )
}

fn perf_case_contract(suite: &str, case: &str) -> PerfCaseContract {
    match (suite, case) {
        ("commit_delta_matrix", "persisted_single_entity_create")
        | (
            "artifact_recoverability_matrix",
            "geometry_diagnostics_summary_vs_trace_recoverability",
        )
        | ("artifact_recoverability_matrix", "chip_compiled_artifact_recoverability")
        | ("cad_topology_matrix", "assembly_interface_bridge_wave")
        | ("chip_simulator_matrix", "checkpoint_window_recover_compile_round_trip")
        | ("durability_append_matrix", "append_canonical_envelope_fresh_store")
        | ("durability_append_matrix", "append_canonical_envelope_existing_segment")
        | ("index_parity_matrix", "persisted_recovery_generation_parity")
        | ("invariant_materialization_matrix", "custom_structural_surface_commit_wave")
        | ("hot_cold_path_matrix", "geometry_hot_commit_vs_replay_reconstruction")
        | ("hot_cold_path_matrix", "chip_hot_compile_vs_recovery_compile")
        | ("hot_cold_path_matrix", "geometry_rich_publication_hot_vs_replay_truth")
        | ("hot_cold_path_matrix", "chip_rich_compile_hot_vs_recovery_compile")
        | ("recoverability_policy_matrix", "geometry_hot_truth_vs_deferred_trace_policy")
        | ("recoverability_policy_matrix", "chip_compile_reconstructable_policy")
        | ("rocketship_scale_matrix", "hundred_k_nodes_geometry_profile_propagation_wave")
        | (
            "rocketship_scale_matrix",
            "hundred_k_nodes_pseudorealistic_large_flat_entity_batch_wave",
        )
        | (
            "rocketship_scale_matrix",
            "hundred_k_nodes_pseudorealistic_mixed_entity_relation_batch_wave",
        )
        | ("rocketship_scale_matrix", "hundred_k_nodes_pseudorealistic_propagation_wave")
        | ("rocketship_scale_matrix", "hundred_k_nodes_pseudorealistic_subsystem_round_trip")
        | ("rocketship_scale_matrix", "hundred_k_nodes_zero_diagnostics_narrow_round_trip")
        | ("rocketship_scale_matrix", "hundred_k_nodes_geometry_profile_narrow_round_trip")
        | ("sustained_load_matrix", "rocketship_hot_update_endurance")
        | ("sustained_load_matrix", "rocketship_propagation_endurance")
        | ("sustained_load_matrix", "chip_global_step_endurance")
        | ("replay_recovery_matrix", "durable_replay_lineage_basis")
        | ("sustained_load_matrix", "replay_window_drift_stability") => {
            PerfCaseContract::io_bursty()
        }
        ("commit_delta_matrix", "single_partition_create_burst")
        | ("geometry_kernel_matrix", "topology_identity_survival_recovery_round_trip")
        | ("chip_simulator_matrix", "branch_rollback_compile_step_window")
        | ("index_parity_matrix", "entity_field_equals_warm_generation")
        | ("inspection_budget_matrix", "retention_commit_window")
        | ("sustained_load_matrix", "commit_query_churn_stability")
        | ("snapshot_materialization_matrix", "projection_entity_identity_surface")
        | ("workflow_matrix", "persisted_recovery_replay_round_trip")
        | ("profile_matrix", "certification_core_rich_commit_query_round_trip")
        | ("profile_matrix", "geometry_kernel_rich_commit_query_round_trip") => {
            PerfCaseContract::volatile()
        }
        ("chip_simulator_matrix", "event_wave_compile_churn_window")
        | ("chip_simulator_matrix", "event_wave_compile_churn_rich_diagnostics") => {
            PerfCaseContract::extreme_volatile()
        }
        ("merge_lineage_matrix", "merge_execution_feature_adoption")
        | ("merge_lineage_matrix", "merge_execution_feature_adoption_zero_diagnostics_budget")
        | ("merge_lineage_matrix", "merge_execute_phase_timing_feature_adoption")
        | ("merge_lineage_matrix", "merge_execution_vs_persisted_commit_floor")
        | ("merge_lineage_matrix", "merge_prepare_vs_execute_feature_adoption")
        | (
            "geometry_artifact_decomposition_matrix",
            "hundred_k_nodes_pseudorealistic_rich_artifact_classes",
        )
        | ("geometry_kernel_matrix", "topology_bridge_connectivity_wave")
        | ("chip_simulator_matrix", "dense_fanout_compile_wave")
        | ("chip_simulator_matrix", "dense_fanout_compile_wave_rich_diagnostics")
        | ("downstream_runtime_mock_matrix", "geometry_commit_downstream_wave_operational")
        | ("downstream_runtime_mock_matrix", "geometry_commit_downstream_wave_development")
        | (
            "downstream_runtime_mock_matrix",
            "geometry_commit_downstream_wave_medium_region_operational",
        )
        | (
            "downstream_runtime_mock_matrix",
            "geometry_commit_downstream_wave_medium_region_development",
        )
        | (
            "downstream_runtime_mock_matrix",
            "geometry_commit_downstream_wave_mixed_locality_operational",
        )
        | ("mixed_load_matrix", "concurrent_relation_index_parity_pressure")
        | ("game_engine_matrix", "local_scene_graph_propagation_wave")
        | ("game_engine_matrix", "mixed_read_write_frame_churn_window")
        | ("query_packet_matrix", "explicit_targets_cross_partition")
        | ("sustained_load_matrix", "mixed_topology_query_churn_stability")
        | ("sustained_load_matrix", "retention_pass_drift_stability")
        | ("workflow_matrix", "trade_correction_analysis_round_trip")
        | ("workflow_matrix", "fintech_intraday_risk_branch_round_trip")
        | ("workflow_matrix", "fintech_trade_correction_audit_round_trip")
        | ("profile_matrix", "certification_core_zero_diagnostics_commit_query_round_trip") => {
            PerfCaseContract::bursty()
        }
        ("commit_delta_matrix", "cross_partition_relation_burst")
        | ("geometry_kernel_matrix", "topology_bridge_connectivity_wave_rich_geometry_profile")
        | ("geometry_kernel_matrix", "topology_bridge_connectivity_wave_zero_diagnostics")
        | ("chip_simulator_matrix", "flat_entity_step_batch_compile_window")
        | ("mixed_load_matrix", "concurrent_snapshot_version_read_pressure")
        | ("query_packet_matrix", "connectivity_traversal_cross_partition")
        | ("snapshot_materialization_matrix", "version_read_view_historical")
        | ("replay_recovery_matrix", "checkpoint_recover_suffix_replay") => {
            PerfCaseContract::noisy()
        }
        ("merge_lineage_matrix", "merge_planning_divergent_update")
        | ("index_parity_matrix", "entity_field_equals_build_failed_storage_read")
        | ("inspection_budget_matrix", "structural_identity_historical_window")
        | ("merge_lineage_matrix", "lineage_branch_divergence_breadth")
        | ("query_packet_matrix", "entity_kind_scan_partition_matrix")
        | ("retention_reclaim_matrix", "replay_pin_release_deleted_relation")
        | ("snapshot_materialization_matrix", "snapshot_read_view_current") => {
            PerfCaseContract::tiny()
        }
        ("inspection_budget_matrix", "graph_kind_connectivity_bundle")
        | ("retention_reclaim_matrix", "snapshot_release_to_reclaimable_entity")
        | ("workflow_matrix", "retention_release_reclaim_round_trip") => PerfCaseContract::micro(),
        _ => PerfCaseContract::standard(),
    }
}
