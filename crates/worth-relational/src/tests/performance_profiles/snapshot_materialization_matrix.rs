use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_snapshot_materialization_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let suite = "snapshot_materialization_matrix";

    let snapshot_read_samples = capture_perf_samples(suite, "snapshot_read_view_current", || {
        let runtime = runtime_with_test_schema();
        for index in 0..128 {
            let _ = create_entity_in_partition(
                execution,
                &runtime,
                &format!("entity-{index}"),
                PartitionId(1 + (index % 4) as u32),
            );
        }
        let snapshot = runtime.visibility_authority().snapshot();

        runtime.performance_access().reset_counters();
        let started_at = Instant::now();
        let read = runtime
            .read_truth()
            .read_snapshot(&snapshot, execution)
            .expect("read request remains live")
            .expect("snapshot read");
        let elapsed_micros = started_at.elapsed().as_micros();
        let counters = runtime.performance_access().counters();

        PerfMeasurement {
            elapsed_micros,
            metrics: perf_metrics!({
                "entities": read.entities().len(),
                "relations": read.relations().len(),
                "counters": counters,
            }),
        }
    });
    assert!(snapshot_read_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &snapshot_read_samples,
        "snapshot reads should remain cache-local and materialize only the live entity surface",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "snapshot_pin_full_rebuilds") == 0
                && counter_u64(metrics, "visible_authoritative_entity_records_materialized") == 128
                && metrics["entities"].as_u64() == Some(128)
        },
    );

    let historical_read_samples =
        capture_perf_samples(suite, "version_read_view_historical", || {
            let runtime = runtime_with_test_schema();
            for index in 0..96 {
                let _ = create_entity_in_partition(
                    execution,
                    &runtime,
                    &format!("before-{index}"),
                    PartitionId(1 + (index % 3) as u32),
                );
            }
            let pinned_snapshot = runtime.visibility_authority().snapshot();
            for index in 0..24 {
                let entity_id = create_entity_in_partition(
                    execution,
                    &runtime,
                    &format!("after-{index}"),
                    PartitionId(9 + index as u32),
                );
                let _ = update_entity(
                    execution,
                    &runtime,
                    entity_id,
                    &format!("after-updated-{index}"),
                );
            }

            runtime.performance_access().reset_counters();
            let started_at = Instant::now();
            let read = runtime
                .read_truth()
                .read_version(execution, pinned_snapshot.version_id)
                .expect("read request remains live");
            let elapsed_micros = started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "entities": read.entities().len(),
                    "relations": read.relations().len(),
                    "counters": counters,
                }),
            }
        });
    assert!(historical_read_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &historical_read_samples,
        "historical reads should reconstruct only the pinned version surface",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "snapshot_pin_full_rebuilds") == 0
                && counter_u64(metrics, "visible_authoritative_entity_records_materialized") == 96
                && metrics["entities"].as_u64() == Some(96)
        },
    );

    let projection_samples =
        capture_perf_samples(suite, "projection_entity_identity_surface", || {
            let runtime = runtime_with_test_schema();
            for index in 0..128 {
                let _ = create_entity_in_partition(
                    execution,
                    &runtime,
                    &format!("projection-{index}"),
                    PartitionId(1 + (index % 4) as u32),
                );
            }
            let snapshot = runtime.visibility_authority().snapshot();

            runtime.performance_access().reset_counters();
            let started_at = Instant::now();
            let projected = runtime
                .read_truth()
                .project_snapshot(&snapshot, execution)
                .expect("projection snapshot")
                .entities::<EntityIdentityProjection>(execution)
                .expect("read request remains live");
            let elapsed_micros = started_at.elapsed().as_micros();
            let counters = runtime.performance_access().counters();

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "projected_entities": projected.len(),
                    "counters": counters,
                }),
            }
        });
    assert!(projection_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &projection_samples,
        "identity projection should remain narrow and allocation-light",
        |metrics| {
            counter_u64(metrics, "full_state_clones") == 0
                && counter_u64(metrics, "snapshot_pin_full_rebuilds") == 0
                && counter_u64(metrics, "visible_authoritative_entity_records_materialized") == 0
                && metrics["projected_entities"].as_u64() == Some(128)
        },
    );
}
