mod scan;

#[test]
fn timed_assertion_guard_refuses_every_direct_clock_spelling() {
    let shapes = [
        "Instant::now()",
        "std::time::SystemTime::now()",
        "clock.duration_since(origin)",
        "clock.checked_duration_since(origin)",
        "clock.saturating_duration_since(origin)",
    ];
    let found: Vec<_> = shapes.iter().map(|shape| {
        let source = format!("fn wait() {{ let deadline = {shape} + BOUND; assert!({shape} < deadline); }} #[test] fn caller() {{ wait(); }}");
        scan::violations(&source)
    }).collect();
    assert_eq!(found, vec![vec!["caller".to_owned()]; shapes.len()]);
}

#[test]
fn timed_assertion_guard_refuses_a_clock_bound_to_a_local_or_const() {
    let sources = [
        "fn wait() { let deadline = Instant::now() + BOUND; assert!(current < deadline); } #[test] fn caller() { wait(); }",
        "const BOUND: std::time::Duration = std::time::Duration::from_secs(3); fn wait() { assert!(measured < BOUND); } #[test] fn caller() { wait(); }",
    ];
    let found: Vec<_> = sources
        .iter()
        .map(|source| scan::violations(source))
        .collect();
    assert_eq!(found, vec![vec!["caller".to_owned()]; sources.len()]);
}

#[test]
fn timed_assertion_guard_refuses_a_unit_returning_clock_helper() {
    let source = r#"
        fn assert_budget() {
            assert!(started.elapsed() <= std::time::Duration::from_secs(600));
        }
        #[test] fn through_helper() { assert_budget(); }
    "#;
    assert_eq!(scan::violations(source), ["through_helper"]);
    let caller = "#[test] fn across_files() { support::forward_budget(); }";
    let forwarding = "fn forward_budget() { assert_budget(); }";
    let helpers = scan::clock_helpers(&[("helper.rs", source), ("forward.rs", forwarding)], &[]);
    assert_eq!(
        scan::violations_with_helpers(caller, &helpers),
        ["across_files"]
    );
}

#[test]
fn timed_assertion_guard_refuses_live_elapsed_and_duration_limits() {
    let source = r#"
        #[test] fn elapsed_limit() {
            let measured = started.elapsed();
            assert!(measured.as_millis() < 500);
        }
        #[test] fn duration_limit() {
            let duration = std::time::Duration::from_millis(2);
            assert!(duration <= std::time::Duration::from_millis(3));
        }
        #[test] #[ignore = "quiet timing run"] fn ignored_limit() {
            assert!(started.elapsed() < std::time::Duration::from_secs(1));
        }
        #[test] fn branch_limit() {
            if started.elapsed() > std::time::Duration::from_secs(1) { panic!("late"); }
        }
        #[test] fn method_limit() {
            if started.elapsed().gt(&std::time::Duration::from_secs(1)) { panic!("late"); }
        }
        #[test] fn counted_limit() { assert!(visits < 10); }
    "#;
    assert_eq!(
        scan::violations(source),
        [
            "elapsed_limit",
            "duration_limit",
            "branch_limit",
            "method_limit"
        ]
    );
}

// Exact file/function pairs only. Add a reason when a real hang guard needs one.
const HANG_GUARDS: &[(&str, &str, &str)] = &[
    (
        "src/branch/owner_services/basis_port/tests/owner_postures.rs",
        "retired_rolled_back_closing_and_gone_postures_remain_exact",
        "bounded wait for closing before exact posture assertions",
    ),
    (
        "src/branch/owner_services/lifecycle_port/tests/lifecycle_observation.rs",
        "planning_denies_during_close_and_after_owner_loss_without_hidden_ownership",
        "bounded wait for closing before planning denials",
    ),
    (
        "src/branch/owner_services/lifecycle_port/tests/lifecycle_observation.rs",
        "weak_port_observes_open_closing_closed_and_owner_loss",
        "bounded lifecycle progress observation",
    ),
    (
        "src/branch/owner_services/mutation_port/tests/races.rs",
        "same_basis_advance_race_forces_each_effectful_request_as_legal_winner",
        "bounded wait for real loser cell contention",
    ),
    (
        "src/branch/owner_services/tests/branch_registry.rs",
        "retirement_excludes_new_lookup_then_waits_for_prelooked_cell_work",
        "bounded retirement and prelooked cell rendezvous",
    ),
    (
        "src/branch/owner_services/tests/cancellation.rs",
        "cancellation_while_waiting_for_same_cell_denies_without_movement",
        "bounded wait for the cell waiter before cancellation",
    ),
    (
        "src/branch/owner_services/tests/cell_progress.rs",
        "same_cell_serializes_with_bounded_exact_contact_and_wait_evidence",
        "bounded wait for same-cell serialization evidence",
    ),
    (
        "src/branch/owner_services/tests/fork_contracts.rs",
        "late_fork_cancellation_drops_preconstructed_destination_without_source_movement",
        "bounded source-cell contention before fork cancellation",
    ),
    (
        "src/branch/owner_services/tests/lifecycle.rs",
        "close_drains_admitted_work_and_monotonically_denies_late_admission",
        "bounded closing and drain completion observation",
    ),
    (
        "src/branch/owner_services/tests/lifecycle.rs",
        "lifecycle_poison_policy_preserves_open_status_and_later_close",
        "bounded close completion after poison recovery",
    ),
    (
        "src/branch/owner_services/tests/operation_control/close_unwind.rs",
        "close_batch_fault_releases_cleanup_claim_for_waiting_retry",
        "bounded cleanup-claim waiter and retry completion",
    ),
    (
        "src/branch/owner_services/tests/operation_control/fork_unwind.rs",
        "fork_post_capture_faults_preserve_performed_destination_and_release_source_custody",
        "bounded source-custody release and injected fault completion",
    ),
    (
        "src/branch/owner_services/tests/output_retention.rs",
        "carried_advance_output_handoff_survives_close_fencing_new_work",
        "bounded closing and retained handoff completion",
    ),
    (
        "src/branch/owner_services/tests/retention_lifecycle.rs",
        "prior_admission_retains_acquisition_rights_during_closing_and_closed_denies_fresh_work",
        "bounded closing and final retirement completion",
    ),
    (
        "tests/request_differential.rs",
        "checked_request",
        "five-second worker rendezvous hang guard",
    ),
    (
        "src/tests/leased_execution/support/task_rendezvous.rs",
        "meet",
        "ten-second task rendezvous hang guard",
    ),
];

#[test]
fn default_signal_tests_do_not_assert_wall_clock_limits() {
    fn inspect(
        directory: &std::path::Path,
        base: &std::path::Path,
        sources: &mut Vec<(String, String)>,
    ) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                inspect(&path, base, sources);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let source = std::fs::read_to_string(&path).unwrap();
                let file = path
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                sources.push((file, source));
            }
        }
    }
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    for directory in [base.join("src"), base.join("tests")] {
        inspect(&directory, base, &mut sources);
    }
    let borrowed: Vec<_> = sources
        .iter()
        .map(|(file, source)| (file.as_str(), source.as_str()))
        .collect();
    let helpers = scan::clock_helpers(&borrowed, HANG_GUARDS);
    let mut found = Vec::new();
    for (file, source) in &sources {
        for function in scan::violations_with_helpers(source, &helpers) {
            if !HANG_GUARDS
                .iter()
                .any(|(f, n, reason)| *f == file && *n == function && !reason.is_empty())
            {
                found.push(format!("{file}::{function}"));
            }
        }
    }
    found.sort();
    assert!(
        found.is_empty(),
        "default tests assert wall-clock limits: {found:#?}"
    );
}

#[test]
fn timed_assertion_guard_pins_the_three_original_timing_tests() {
    for (source, expected) in [
        (
            include_str!("../performance_profiles/measured_packet.rs"),
            "measured_idle_versus_balanced_compares_wall_clock_reports",
        ),
        (
            include_str!("../performance_profiles/measured_pressure.rs"),
            "measured_idle_versus_introspective_compares_wall_clock_throughput",
        ),
        (
            include_str!("../performance/performance_scaling.rs"),
            "push_perf_10k_nodes",
        ),
    ] {
        let unguarded = source.replace(
            "#[ignore = \"asserts wall-clock time; run on a quiet machine\"]",
            "",
        );
        assert_eq!(scan::violations(&unguarded), [expected]);
        assert!(scan::violations(source).is_empty());
    }
}
