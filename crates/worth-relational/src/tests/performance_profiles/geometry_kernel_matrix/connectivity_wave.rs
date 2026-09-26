use super::*;

pub(super) fn certify_topology_bridge_connectivity_wave(suite: &'static str) {
    let topology_bridge_samples =
        capture_perf_samples(suite, "topology_bridge_connectivity_wave", || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            let mut cluster_a = Vec::new();
            let mut cluster_b = Vec::new();
            for index in 0..6 {
                cluster_a.push(create_entity_in_partition(
                    &runtime,
                    &format!("cluster-a-{index}"),
                    PartitionId((index % 3) as u32 + 1),
                ));
                cluster_b.push(create_entity_in_partition(
                    &runtime,
                    &format!("cluster-b-{index}"),
                    PartitionId((index % 3) as u32 + 5),
                ));
            }
            for index in 0..(cluster_a.len() - 1) {
                create_relation_in_partition(
                    &runtime,
                    cluster_a[index],
                    cluster_a[index + 1],
                    &format!("a-link-{index}"),
                    PartitionId(11),
                );
                create_relation_in_partition(
                    &runtime,
                    cluster_b[index],
                    cluster_b[index + 1],
                    &format!("b-link-{index}"),
                    PartitionId(12),
                );
            }

            runtime.performance_access().reset_counters();
            let bridge_started_at = Instant::now();
            let bridge_outcome = create_relation_outcome(
                &runtime,
                cluster_a[2],
                cluster_b[2],
                "bridge-topology-wave",
            );
            let bridge_commit_micros = bridge_started_at.elapsed().as_micros();

            let connectivity_started_at = Instant::now();
            let summary = runtime.inspect_what_happened().connectivity_summary(
                &ConnectivityInspectionRequest {
                    scope: InspectionScope::Current,
                    partition_scope: None,
                    relation_kind_scope: Some(vec![KindId(2)]),
                    include_members: false,
                    budget: ConnectivityInspectionBudget {
                        max_entities: 64,
                        max_relations: 64,
                        max_frontier: 64,
                        max_components: 8,
                        max_work_units: 256,
                    },
                },
            );
            let connectivity_summary_micros = connectivity_started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros: bridge_commit_micros + connectivity_summary_micros,
                metrics: perf_metrics!({
                    "bridge_commit_micros": bridge_commit_micros,
                    "connectivity_summary_micros": connectivity_summary_micros,
                    "bridge_changed_records": bridge_outcome.changed_records.len(),
                    "component_count": summary.component_count,
                    "largest_component_size": summary.largest_component_size,
                    "enumerated_entity_count": summary.enumerated_entity_count,
                    "availability": format!("{:?}", summary.availability),
                    "degradation_count": summary.degradations.len(),
                    "counters": counters,
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "topology_bridge_connectivity_wave",
        &topology_bridge_samples,
        &[
            ("bridge_commit_micros", &["bridge_commit_micros"]),
            (
                "connectivity_summary_micros",
                &["connectivity_summary_micros"],
            ),
            ("component_count", &["component_count"]),
            ("largest_component_size", &["largest_component_size"]),
            ("enumerated_entity_count", &["enumerated_entity_count"]),
        ],
    );
    assert!(topology_bridge_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &topology_bridge_samples,
        "geometry topology bridge should collapse two local components into one bounded connectivity surface",
        |metrics| {
            metrics["bridge_changed_records"].as_u64() == Some(1)
                && metrics["component_count"].as_u64() == Some(1)
                && metrics["largest_component_size"].as_u64() == Some(12)
                && metrics["enumerated_entity_count"].as_u64() == Some(12)
                && metrics["availability"].as_str() == Some("Direct")
                && metrics["degradation_count"].as_u64() == Some(1)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "relation_slots_touched_by_commit") == 1
                && counter_u64(metrics, "inspection_connectivity_summary_requests") == 1
                && counter_u64(metrics, "inspection_connectivity_components_evaluated") == 1
                && counter_u64(metrics, "inspection_connectivity_frontier_expansions") >= 1
                && counter_u64(metrics, "inspection_connectivity_entity_scans") == 12
                && counter_u64(metrics, "inspection_connectivity_relation_scans") == 11
        },
    );

    let topology_bridge_rich_geometry_samples = capture_perf_samples(
        suite,
        "topology_bridge_connectivity_wave_rich_geometry_profile",
        || {
            let runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();
            let mut cluster_a = Vec::new();
            let mut cluster_b = Vec::new();
            for index in 0..6 {
                cluster_a.push(create_entity_in_partition(
                    &runtime,
                    &format!("rich-cluster-a-{index}"),
                    PartitionId((index % 3) as u32 + 1),
                ));
                cluster_b.push(create_entity_in_partition(
                    &runtime,
                    &format!("rich-cluster-b-{index}"),
                    PartitionId((index % 3) as u32 + 5),
                ));
            }
            for index in 0..(cluster_a.len() - 1) {
                create_relation_in_partition(
                    &runtime,
                    cluster_a[index],
                    cluster_a[index + 1],
                    &format!("rich-a-link-{index}"),
                    PartitionId(11),
                );
                create_relation_in_partition(
                    &runtime,
                    cluster_b[index],
                    cluster_b[index + 1],
                    &format!("rich-b-link-{index}"),
                    PartitionId(12),
                );
            }

            runtime.performance_access().reset_counters();
            let bridge_started_at = Instant::now();
            let bridge_outcome = create_relation_outcome(
                &runtime,
                cluster_a[2],
                cluster_b[2],
                "bridge-topology-wave-rich",
            );
            let bridge_commit_micros = bridge_started_at.elapsed().as_micros();

            let connectivity_started_at = Instant::now();
            let summary = runtime.inspect_what_happened().connectivity_summary(
                &ConnectivityInspectionRequest {
                    scope: InspectionScope::Current,
                    partition_scope: None,
                    relation_kind_scope: Some(vec![KindId(2)]),
                    include_members: false,
                    budget: ConnectivityInspectionBudget {
                        max_entities: 64,
                        max_relations: 64,
                        max_frontier: 64,
                        max_components: 8,
                        max_work_units: 256,
                    },
                },
            );
            let connectivity_summary_micros = connectivity_started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();
            let (diagnostic_artifact_count, detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);

            PerfMeasurement {
                elapsed_micros: bridge_commit_micros + connectivity_summary_micros,
                metrics: perf_metrics!({
                    "bridge_commit_micros": bridge_commit_micros,
                    "connectivity_summary_micros": connectivity_summary_micros,
                    "bridge_changed_records": bridge_outcome.changed_records.len(),
                    "component_count": summary.component_count,
                    "largest_component_size": summary.largest_component_size,
                    "enumerated_entity_count": summary.enumerated_entity_count,
                    "diagnostic_artifact_count": diagnostic_artifact_count,
                    "detailed_trace_entries": detailed_trace_entries,
                    "availability": format!("{:?}", summary.availability),
                    "degradation_count": summary.degradations.len(),
                    "counters": counters,
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "topology_bridge_connectivity_wave_rich_geometry_profile",
        &topology_bridge_rich_geometry_samples,
        &[
            ("bridge_commit_micros", &["bridge_commit_micros"]),
            (
                "connectivity_summary_micros",
                &["connectivity_summary_micros"],
            ),
            ("component_count", &["component_count"]),
            ("largest_component_size", &["largest_component_size"]),
            ("enumerated_entity_count", &["enumerated_entity_count"]),
            ("diagnostic_artifact_count", &["diagnostic_artifact_count"]),
            ("detailed_trace_entries", &["detailed_trace_entries"]),
        ],
    );
    assert!(topology_bridge_rich_geometry_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &topology_bridge_rich_geometry_samples,
        "geometry rich topology bridge should preserve the same connectivity truth while deferring hot detailed traces",
        |metrics| {
            metrics["bridge_changed_records"].as_u64() == Some(1)
                && metrics["component_count"].as_u64() == Some(1)
                && metrics["largest_component_size"].as_u64() == Some(12)
                && metrics["enumerated_entity_count"].as_u64() == Some(12)
                && metrics["availability"].as_str() == Some("Direct")
                && metrics["degradation_count"].as_u64() == Some(1)
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "relation_slots_touched_by_commit") == 1
                && counter_u64(metrics, "inspection_connectivity_summary_requests") == 1
                && counter_u64(metrics, "inspection_connectivity_entity_scans") == 12
                && counter_u64(metrics, "inspection_connectivity_relation_scans") == 11
        },
    );

    let topology_bridge_zero_diag_samples = capture_perf_samples(
        suite,
        "topology_bridge_connectivity_wave_zero_diagnostics",
        || {
            let mut runtime =
                runtime_with_test_schema_profile(RelationalRuntimeProfile::GeometryKernel);
            runtime.configure_diagnostics_for_test(|profile| {
                profile.detailed_traces_enabled = false;
                profile.max_entries_per_artifact = 0;
            });
            let diagnostics_start = runtime.publication().diagnostic_artifacts().len();
            let mut cluster_a = Vec::new();
            let mut cluster_b = Vec::new();
            for index in 0..6 {
                cluster_a.push(create_entity_in_partition(
                    &runtime,
                    &format!("zero-cluster-a-{index}"),
                    PartitionId((index % 3) as u32 + 1),
                ));
                cluster_b.push(create_entity_in_partition(
                    &runtime,
                    &format!("zero-cluster-b-{index}"),
                    PartitionId((index % 3) as u32 + 5),
                ));
            }
            for index in 0..(cluster_a.len() - 1) {
                create_relation_in_partition(
                    &runtime,
                    cluster_a[index],
                    cluster_a[index + 1],
                    &format!("zero-a-link-{index}"),
                    PartitionId(11),
                );
                create_relation_in_partition(
                    &runtime,
                    cluster_b[index],
                    cluster_b[index + 1],
                    &format!("zero-b-link-{index}"),
                    PartitionId(12),
                );
            }

            runtime.performance_access().reset_counters();
            let bridge_started_at = Instant::now();
            let bridge_outcome = create_relation_outcome(
                &runtime,
                cluster_a[2],
                cluster_b[2],
                "bridge-topology-wave-zero",
            );
            let bridge_commit_micros = bridge_started_at.elapsed().as_micros();

            let connectivity_started_at = Instant::now();
            let summary = runtime.inspect_what_happened().connectivity_summary(
                &ConnectivityInspectionRequest {
                    scope: InspectionScope::Current,
                    partition_scope: None,
                    relation_kind_scope: Some(vec![KindId(2)]),
                    include_members: false,
                    budget: ConnectivityInspectionBudget {
                        max_entities: 64,
                        max_relations: 64,
                        max_frontier: 64,
                        max_components: 8,
                        max_work_units: 256,
                    },
                },
            );
            let connectivity_summary_micros = connectivity_started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();
            let (diagnostic_artifact_count, detailed_trace_entries) =
                fresh_diagnostics_metrics(&runtime, diagnostics_start);

            PerfMeasurement {
                elapsed_micros: bridge_commit_micros + connectivity_summary_micros,
                metrics: perf_metrics!({
                    "bridge_commit_micros": bridge_commit_micros,
                    "connectivity_summary_micros": connectivity_summary_micros,
                    "bridge_changed_records": bridge_outcome.changed_records.len(),
                    "component_count": summary.component_count,
                    "largest_component_size": summary.largest_component_size,
                    "enumerated_entity_count": summary.enumerated_entity_count,
                    "diagnostic_artifact_count": diagnostic_artifact_count,
                    "detailed_trace_entries": detailed_trace_entries,
                    "availability": format!("{:?}", summary.availability),
                    "degradation_count": summary.degradations.len(),
                    "counters": counters,
                }),
            }
        },
    );
    emit_metric_summaries(
        suite,
        "topology_bridge_connectivity_wave_zero_diagnostics",
        &topology_bridge_zero_diag_samples,
        &[
            ("bridge_commit_micros", &["bridge_commit_micros"]),
            (
                "connectivity_summary_micros",
                &["connectivity_summary_micros"],
            ),
            ("component_count", &["component_count"]),
            ("largest_component_size", &["largest_component_size"]),
            ("enumerated_entity_count", &["enumerated_entity_count"]),
            ("diagnostic_artifact_count", &["diagnostic_artifact_count"]),
            ("detailed_trace_entries", &["detailed_trace_entries"]),
        ],
    );
    assert!(topology_bridge_zero_diag_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &topology_bridge_zero_diag_samples,
        "geometry zero-diagnostics topology bridge should preserve connectivity truth while eliminating trace entries",
        |metrics| {
            metrics["bridge_changed_records"].as_u64() == Some(1)
                && metrics["component_count"].as_u64() == Some(1)
                && metrics["largest_component_size"].as_u64() == Some(12)
                && metrics["enumerated_entity_count"].as_u64() == Some(12)
                && metrics["availability"].as_str() == Some("Direct")
                && metrics["degradation_count"].as_u64() == Some(1)
                && metrics["diagnostic_artifact_count"].as_u64().unwrap_or(0) >= 1
                && metrics["detailed_trace_entries"].as_u64() == Some(0)
                && counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "relation_slots_touched_by_commit") == 1
                && counter_u64(metrics, "inspection_connectivity_summary_requests") == 1
                && counter_u64(metrics, "inspection_connectivity_entity_scans") == 12
                && counter_u64(metrics, "inspection_connectivity_relation_scans") == 11
        },
    );
}
