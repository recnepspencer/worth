use super::fixtures::*;

#[test]
fn harness_phase8_certification_matrix_reports_parallel_lane_diagnostics() {
    let (fixture, batch, request) = harness_phase8_fixture_batch_request();

    let report = worth_harness::facade::certification_matrix(
        RelationalHarnessAdapter,
        fixture,
        request,
        ExecutionProfile::serial("serial"),
    )
    .mutate(batch)
    .candidates([
        ExecutionProfile::staged_parallel("staged"),
        ExecutionProfile::full_parallel("post-commit"),
    ])
    .certify()
    .unwrap();

    assert!(report.matched);
    assert_eq!(report.baseline_profile, "serial");
    assert_eq!(
        harness_summary_field(
            report.baseline_diagnostics_summary.as_ref().unwrap(),
            "runtime_execution_model"
        ),
        Some("SingleLaneExecution")
    );
    assert_eq!(report.cases.len(), 2);
    assert_eq!(report.cases[0].candidate_profile, "staged");
    assert_eq!(report.cases[1].candidate_profile, "post-commit");
    assert_eq!(
        harness_summary_field(
            report.cases[0].diagnostics_summary.as_ref().unwrap(),
            "runtime_execution_model"
        ),
        Some("ParallelPreparation")
    );
    assert_eq!(
        harness_summary_field(
            report.cases[1].diagnostics_summary.as_ref().unwrap(),
            "runtime_execution_model"
        ),
        Some("ParallelPostCommitConsumption")
    );
    assert!(report
        .cases
        .iter()
        .all(|case| case.comparison.mismatches.is_empty()));
}

#[test]
fn harness_phase8_certification_matrix_closes_out_supported_runtime_lanes() {
    let (fixture, batch, request) = harness_phase8_fixture_batch_request();

    let report = worth_harness::facade::certification_matrix(
        RelationalHarnessAdapter,
        fixture,
        request,
        ExecutionProfile::serial("serial"),
    )
    .mutate(batch)
    .candidates([
        ExecutionProfile::staged_parallel("staged"),
        ExecutionProfile::full_parallel("post-commit"),
    ])
    .certify()
    .unwrap();

    assert!(report.matched);
    assert_eq!(report.baseline_profile, "serial");
    assert_eq!(
        harness_summary_field(
            report.baseline_diagnostics_summary.as_ref().unwrap(),
            "execution_mode"
        ),
        Some("Serial")
    );
    assert_eq!(
        harness_summary_field(
            report.baseline_diagnostics_summary.as_ref().unwrap(),
            "runtime_execution_model"
        ),
        Some("SingleLaneExecution")
    );

    let staged = certification_case(&report, "staged");
    let post_commit = certification_case(&report, "post-commit");

    assert!(staged.comparison.matched);
    assert!(post_commit.comparison.matched);
    assert_eq!(
        harness_summary_field(
            staged.diagnostics_summary.as_ref().unwrap(),
            "runtime_execution_model"
        ),
        Some("ParallelPreparation")
    );
    assert_eq!(
        harness_summary_field(
            post_commit.diagnostics_summary.as_ref().unwrap(),
            "runtime_execution_model"
        ),
        Some("ParallelPostCommitConsumption")
    );
    assert!(harness_summary_counter(
        staged.diagnostics_summary.as_ref().unwrap(),
        "preparation_packet_count"
    )
    .is_some());
    assert!(harness_summary_counter(
        post_commit.diagnostics_summary.as_ref().unwrap(),
        "post_commit_consumer_packet_count"
    )
    .is_some());
}

#[test]
fn harness_phase8_observed_matrix_exposes_mode_specific_metadata() {
    let (fixture, batch, request) = harness_phase8_fixture_batch_request();

    let bundles = worth_harness::facade::run_matrix(RelationalHarnessAdapter, fixture, request)
        .mutate(batch)
        .profiles([
            ExecutionProfile::serial("serial"),
            ExecutionProfile::staged_parallel("staged"),
            ExecutionProfile::full_parallel("post-commit"),
        ])
        .diagnose()
        .unwrap();

    assert_eq!(bundles.len(), 3);

    let serial = &bundles[0];
    let staged = &bundles[1];
    let post_commit = &bundles[2];

    assert_eq!(serial.core.run.profile_name, "serial");
    assert_eq!(staged.core.run.profile_name, "staged");
    assert_eq!(post_commit.core.run.profile_name, "post-commit");

    let serial_summary = &serial.diagnostics.as_ref().unwrap().summary;
    let staged_summary = &staged.diagnostics.as_ref().unwrap().summary;
    let post_commit_summary = &post_commit.diagnostics.as_ref().unwrap().summary;

    assert_eq!(
        harness_summary_field(serial_summary, "execution_mode"),
        Some("Serial")
    );
    assert_eq!(
        harness_summary_field(serial_summary, "runtime_execution_model"),
        Some("SingleLaneExecution")
    );
    assert_eq!(
        harness_summary_field(staged_summary, "execution_mode"),
        Some("StagedParallel")
    );
    assert_eq!(
        harness_summary_field(staged_summary, "runtime_execution_model"),
        Some("ParallelPreparation")
    );
    assert_eq!(
        harness_summary_field(post_commit_summary, "execution_mode"),
        Some("FullParallel")
    );
    assert_eq!(
        harness_summary_field(post_commit_summary, "runtime_execution_model"),
        Some("ParallelPostCommitConsumption")
    );

    assert!(harness_summary_counter(serial_summary, "preparation_packet_count").is_some());
    assert!(harness_summary_counter(serial_summary, "preparation_packet_item_count").is_some());
    assert!(harness_summary_counter(serial_summary, "preparation_scope_unit_count").is_some());
    assert!(harness_summary_counter(staged_summary, "preparation_packet_count").is_some());
    assert!(
        harness_summary_counter(staged_summary, "preparation_packet_peak_width_total").is_some()
    );
    assert!(
        harness_summary_counter(post_commit_summary, "post_commit_consumer_packet_count").is_some()
    );
    assert!(
        harness_summary_counter(post_commit_summary, "post_commit_consumer_peak_width_total")
            .is_some()
    );
}

#[test]
fn harness_diagnostics_expose_execution_mode_and_performance_counters() {
    let adapter = RelationalHarnessAdapter;
    let (fixture, batch, request) = harness_phase8_fixture_batch_request();
    let profile = ExecutionProfile::staged_parallel("staged");
    let mut runtime = adapter.create_runtime().unwrap();
    adapter.prepare_runtime(&mut runtime, &profile).unwrap();
    adapter.load_fixture(&mut runtime, &fixture).unwrap();
    adapter.apply_mutation_batch(&mut runtime, &batch).unwrap();
    let _ = adapter
        .execute(&mut runtime, &fixture, &request, &profile)
        .unwrap();
    let diagnostics = adapter
        .capture_diagnostics(&runtime, &fixture, &profile)
        .unwrap();

    let summary = diagnostics.summary;
    assert_eq!(
        harness_summary_field(&summary, "execution_mode"),
        Some("StagedParallel")
    );
    assert_eq!(
        harness_summary_field(&summary, "runtime_execution_model"),
        Some("ParallelPreparation")
    );
    assert!(harness_summary_counter(&summary, "preparation_packet_count").is_some());
    assert!(harness_summary_counter(&summary, "preparation_packet_item_count").is_some());
    assert!(harness_summary_counter(&summary, "preparation_packet_peak_width_total").is_some());
    assert!(harness_summary_counter(&summary, "preparation_scope_unit_count").is_some());
}

#[test]
fn harness_phase8_serial_strategy_selection_is_harness_visible_and_still_parity_safe() {
    let adapter = InvariantHarnessAdapter::new(InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::MaxMergedIntents(16),
        )],
    });
    let (fixture, batch, request) = harness_phase8_fixture_batch_request();

    let report = worth_harness::facade::parity_suite(
        adapter,
        fixture,
        request,
        ExecutionProfile::serial("serial"),
    )
    .mutate(batch)
    .candidate(ExecutionProfile::staged_parallel("staged"))
    .compare()
    .unwrap();

    assert!(report.matched);
    assert_eq!(report.results.len(), 1);
    assert!(report.results[0].comparison.mismatches.is_empty());

    let adapter = InvariantHarnessAdapter::new(InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::MaxMergedIntents(16),
        )],
    });
    let (fixture, batch, request) = harness_phase8_fixture_batch_request();
    let bundles = worth_harness::facade::run_matrix(adapter, fixture, request)
        .mutate(batch)
        .profile(ExecutionProfile::staged_parallel("staged"))
        .diagnose()
        .unwrap();
    let summary = bundles[0].diagnostics.as_ref().unwrap().summary.clone();

    let serial_entries = harness_diagnostic_entries(&summary, "SerialPreparationSelected");
    let failure_entries = harness_diagnostic_entries(&summary, "PreparationFailure");

    assert!(serial_entries.iter().any(|entry| {
        harness_diagnostic_field_matches(entry, "reason", "insufficient_packet_breadth")
    }));
    assert!(failure_entries.iter().any(|entry| {
        harness_diagnostic_field_matches(entry, "failure_class", "serial_strategy_selected")
    }));
    assert_eq!(
        harness_summary_field(&summary, "execution_mode"),
        Some("StagedParallel")
    );
    assert_eq!(
        harness_summary_field(&summary, "runtime_execution_model"),
        Some("ParallelPreparation")
    );
    assert!(
        harness_summary_counter(&summary, "preparation_serial_strategy_count")
            .is_some_and(|count| count >= 1)
    );
    assert_eq!(
        harness_summary_counter(&summary, "preparation_staged_parallel_strategy_count"),
        Some(0)
    );
}
