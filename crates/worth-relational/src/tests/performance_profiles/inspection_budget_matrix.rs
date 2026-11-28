use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_inspection_budget_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let suite = "inspection_budget_matrix";

    let graph_kind_connectivity_samples =
        capture_perf_samples(suite, "graph_kind_connectivity_bundle", || {
            let runtime = runtime_with_test_schema();
            let left_a = create_entity_in_partition(execution, &runtime, "left-a", PartitionId(7));
            let _left_b = create_entity_in_partition(execution, &runtime, "left-b", PartitionId(7));
            let _isolated =
                create_entity_in_partition(execution, &runtime, "isolated", PartitionId(11));
            let right = create_entity_in_partition(execution, &runtime, "right", PartitionId(13));
            let _relation = create_relation_in_partition(
                execution,
                &runtime,
                left_a,
                right,
                "rel",
                PartitionId(17),
            );

            runtime.performance_access().reset_counters();

            let graph_started_at = Instant::now();
            let graph = runtime
                .inspect_what_happened()
                .graph_summary(execution, &current_graph_request(None, None, true))
                .expect("read request remains live");
            let graph_micros = graph_started_at.elapsed().as_micros();

            let kind_started_at = Instant::now();
            let kind = runtime
                .inspect_what_happened()
                .kind_summary(
                    execution,
                    &KindInspectionRequest {
                        scope: InspectionScope::Current,
                        partition_scope: Some(vec![PartitionId(7)]),
                        kind_id: KindId(1),
                        record_class: InspectionRecordClass::Entity,
                    },
                )
                .expect("read request remains live");
            let kind_micros = kind_started_at.elapsed().as_micros();

            let connectivity_started_at = Instant::now();
            let connectivity = runtime
                .inspect_what_happened()
                .connectivity_summary(
                    execution,
                    &connectivity_request(InspectionScope::Current, None, None, false),
                )
                .expect("read request remains live");
            let connectivity_micros = connectivity_started_at.elapsed().as_micros();

            PerfMeasurement {
                elapsed_micros: graph_micros + kind_micros + connectivity_micros,
                metrics: perf_metrics!({
                    "graph_micros": graph_micros,
                    "kind_micros": kind_micros,
                    "connectivity_micros": connectivity_micros,
                    "graph_entity_count": graph.entity_count,
                    "graph_relation_count": graph.relation_count,
                    "kind_count": kind.count,
                    "connectivity_component_count": connectivity.component_count,
                    "connectivity_largest_component_size": connectivity.largest_component_size,
                    "graph_access_path": format!("{:?}", graph.access_path),
                    "kind_access_path": format!("{:?}", kind.access_path),
                    "connectivity_access_path": format!("{:?}", connectivity.access_path),
                    "counters": runtime.performance_access().counters(),
                }),
            }
        });
    emit_metric_summaries(
        suite,
        "graph_kind_connectivity_bundle",
        &graph_kind_connectivity_samples,
        &[
            ("graph_micros", &["graph_micros"]),
            ("kind_micros", &["kind_micros"]),
            ("connectivity_micros", &["connectivity_micros"]),
            ("graph_entity_count", &["graph_entity_count"]),
            (
                "connectivity_component_count",
                &["connectivity_component_count"],
            ),
        ],
    );
    assert!(graph_kind_connectivity_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &graph_kind_connectivity_samples,
        "inspection bundles should stay request-shaped and avoid visibility materialization",
        |metrics| {
            metric_u64(metrics, "graph_entity_count") == 4
                && metric_u64(metrics, "graph_relation_count") == 1
                && metric_u64(metrics, "kind_count") == 2
                && metric_u64(metrics, "connectivity_component_count") == 3
                && metric_u64(metrics, "connectivity_largest_component_size") == 2
                && counter_u64(metrics, "inspection_graph_summary_requests") == 1
                && counter_u64(metrics, "inspection_kind_summary_requests") == 1
                && counter_u64(metrics, "inspection_connectivity_summary_requests") == 1
                && counter_u64(metrics, "visible_authoritative_entity_records_materialized") == 4
                && counter_u64(
                    metrics,
                    "visible_authoritative_relation_records_materialized",
                ) == 1
                && counter_u64(metrics, "visibility_entity_slot_scans") == 0
                && counter_u64(metrics, "visibility_relation_slot_scans") == 0
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );

    let retention_commit_samples = capture_perf_samples(suite, "retention_commit_window", || {
        let runtime = runtime_with_test_schema();
        let left = create_entity(execution, &runtime, "left");
        let right = create_entity(execution, &runtime, "right");
        let _relation = create_relation(execution, &runtime, left, right, "rel");
        let latest_commit = runtime
            .history()
            .latest_commit()
            .map(|commit| commit.commit_id)
            .expect("latest commit");

        runtime.performance_access().reset_counters();

        let retention_started_at = Instant::now();
        let retention = runtime
            .inspect_what_happened()
            .retention_summary(execution, &default_retention_request())
            .unwrap();
        let retention_micros = retention_started_at.elapsed().as_micros();

        let commit_started_at = Instant::now();
        let commit = runtime
            .inspect_what_happened()
            .inspect_commit(execution, latest_commit)
            .unwrap()
            .expect("commit inspection");
        let commit_micros = commit_started_at.elapsed().as_micros();

        let recent_started_at = Instant::now();
        let recent = runtime
            .inspect_what_happened()
            .inspect_recent_commits(
                execution,
                &RecentCommitInspectionRequest {
                    branch_id: Some(BranchId("main".to_string())),
                    limit: 3,
                },
            )
            .unwrap();
        let recent_micros = recent_started_at.elapsed().as_micros();

        PerfMeasurement {
            elapsed_micros: retention_micros + commit_micros + recent_micros,
            metrics: perf_metrics!({
                "retention_micros": retention_micros,
                "commit_micros": commit_micros,
                "recent_micros": recent_micros,
                "retention_availability": format!("{:?}", retention.availability),
                "commit_changed_records": commit.changed_records.len(),
                "recent_commit_count": recent.commits.len(),
                "counters": runtime.performance_access().counters(),
            }),
        }
    });
    emit_metric_summaries(
        suite,
        "retention_commit_window",
        &retention_commit_samples,
        &[
            ("retention_micros", &["retention_micros"]),
            ("commit_micros", &["commit_micros"]),
            ("recent_micros", &["recent_micros"]),
            ("recent_commit_count", &["recent_commit_count"]),
        ],
    );
    assert!(retention_commit_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &retention_commit_samples,
        "retention and commit inspection windows should stay index-backed and bounded",
        |metrics| {
            metrics["retention_availability"].as_str() == Some("Direct")
                && metric_u64(metrics, "commit_changed_records") == 1
                && metric_u64(metrics, "recent_commit_count") == 3
                && counter_u64(metrics, "inspection_commit_reads") == 4
                && counter_u64(metrics, "inspection_retention_entity_slot_scans") >= 2
                && counter_u64(metrics, "inspection_retention_relation_slot_scans") >= 1
                && counter_u64(metrics, "visible_authoritative_entity_records_materialized") == 0
                && counter_u64(
                    metrics,
                    "visible_authoritative_relation_records_materialized",
                ) == 0
                && counter_u64(metrics, "full_state_clones") == 0
        },
    );
}
