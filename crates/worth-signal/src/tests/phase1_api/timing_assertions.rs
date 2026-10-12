mod scan;

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
