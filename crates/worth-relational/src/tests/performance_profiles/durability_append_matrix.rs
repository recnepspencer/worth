use super::*;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_durability_append_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let suite = "durability_append_matrix";

    let fresh_append_samples =
        capture_perf_samples(suite, "append_canonical_envelope_fresh_medium", || {
            let runtime = durable_runtime_with_test_schema();

            let outcome = create_entity_outcome(execution, &runtime, "fresh-source");
            let elapsed_micros = outcome.execution().phase_timing.durable_append_micros as u128;

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "medium_head": attached_test_medium(&runtime).head().0,
                    "durable_log_len": runtime.durable_log().len(),
                }),
            }
        });
    assert!(fresh_append_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &fresh_append_samples,
        "a fresh durable commit is the medium's first position and the log's one envelope",
        |metrics| {
            metrics["medium_head"].as_u64() == Some(1)
                && metrics["durable_log_len"].as_u64() == Some(1)
        },
    );

    let warm_append_samples =
        capture_perf_samples(suite, "append_canonical_envelope_existing_medium", || {
            let runtime = durable_runtime_with_test_schema();
            create_entity_outcome(execution, &runtime, "warm-source-a");

            let outcome = create_entity_outcome(execution, &runtime, "warm-source-b");
            let elapsed_micros = outcome.execution().phase_timing.durable_append_micros as u128;

            PerfMeasurement {
                elapsed_micros,
                metrics: perf_metrics!({
                    "medium_head": attached_test_medium(&runtime).head().0,
                    "durable_log_len": runtime.durable_log().len(),
                }),
            }
        });
    assert!(warm_append_samples
        .iter()
        .all(|sample| sample.elapsed_micros > 0));
    assert_budget(
        &warm_append_samples,
        "a warm durable commit extends the medium and the log by exactly one",
        |metrics| {
            metrics["medium_head"].as_u64() == Some(2)
                && metrics["durable_log_len"].as_u64() == Some(2)
        },
    );
}
