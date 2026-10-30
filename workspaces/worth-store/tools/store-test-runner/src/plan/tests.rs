use std::{collections::BTreeSet, path::Path};

use super::{TestExecutionUnit, TestPlan};
use crate::product::{CiTestLane, TestProduct};

#[test]
fn owner_product_maps_the_selected_package_directly() {
    let plan = TestPlan::build(
        &TestProduct::Owner {
            package: "worth-store".into(),
        },
        workspace_root(),
    )
    .unwrap();

    assert!(plan.units()[0]
        .arguments()
        .windows(2)
        .any(|pair| pair == ["-p", "worth-store"]));
}

#[test]
fn scenario_sharding_uses_nextests_stable_hash_partition() {
    let plan = ci_plan(CiTestLane::Scenario, Some((1, 3)));

    assert!(nextest(&plan)
        .arguments()
        .windows(2)
        .any(|pair| pair == ["--partition", "hash:2/3"]));
}

#[test]
fn scenario_plan_selects_ordinary_process_and_determinism_courts_by_prefix() {
    let plan = ci_plan(CiTestLane::Scenario, None);
    let command = nextest(&plan).arguments().join(" ");

    for selector in [
        "binary_id(=worth-store-court-store-replication-physical-record-journeys-durability-admission::worth_store_court_store_replication_physical_record_journeys_durability_admission)",
        "binary_id(=worth-store-court-recovery-runtime-replication::worth_store_court_recovery_runtime_replication)",
        "binary_id(=worth-store-court-relational::worth_store_court_relational)",
    ] {
        assert!(command.contains(selector), "missing court selector {selector}");
    }
    assert!(command.contains(" & (test(/^process::/))"));
    assert!(command.contains(" & (test(/^determinism::/))"));
    assert!(command.contains(
        "not (test(/^process::/) + test(/^oracle::/) + test(/^cost::/) + test(/^determinism::/) + test(/^compile_fail::/))"
    ));
    for category in ["compile_fail", "oracle", "cost"] {
        assert!(
            !command.contains(&format!(" & (test(/^{category}::/))")),
            "scenario absorbed the {category} category"
        );
    }
    assert!(!command.contains("binary_id(=worth-store::physical_record_journeys)"));
}

#[test]
fn nonignored_cost_contracts_have_their_own_landing_partition() {
    let plan = ci_plan(CiTestLane::Cost, None);
    let arguments = nextest(&plan).arguments();
    assert!(arguments.join(" ").contains(" & (test(/^cost::/))"));
    assert!(!arguments
        .iter()
        .any(|argument| argument == "--include-ignored"));
}

#[test]
fn ui_and_formal_products_select_compile_fail_and_oracle_prefixes() {
    let ui = TestPlan::build(&TestProduct::Ui, workspace_root()).unwrap();
    let ui_command = nextest(&ui).arguments().join(" ");
    assert!(ui_command.contains("binary_id(=worth-store-court-store-replication-physical-record-journeys-physical-work::worth_store_court_store_replication_physical_record_journeys_physical_work)"));
    assert!(ui_command.contains(" & (test(/^compile_fail::/))"));
    assert!(!ui_command.contains(" & (test(/^process::/))"));

    let formal = ci_plan(CiTestLane::Formal, None);
    let formal_command = nextest(&formal).arguments().join(" ");
    for selector in [
        "binary_id(=worth-store-court-recovery-runtime-replication-phase-eight-process::worth_store_court_recovery_runtime_replication_phase_eight_process)",
        "binary_id(=worth-store-court-recovery-runtime-replication-production-entry-production-custody-serving::worth_store_court_recovery_runtime_replication_production_entry_production_custody_serving)",
    ] {
        assert!(
            formal_command.contains(selector),
            "missing formal court selector {selector}"
        );
    }
    assert!(formal_command.contains(" & (test(/^oracle::/))"));
    assert!(!formal_command.contains(" & (test(/^compile_fail::/))"));
}

#[test]
fn every_nextest_product_fails_when_its_selection_is_empty() {
    for product in [
        TestProduct::Owner {
            package: "worth-store".into(),
        },
        TestProduct::Smoke,
        TestProduct::Ui,
        TestProduct::Ci {
            lane: CiTestLane::Scenario,
            shard: None,
        },
    ] {
        let plan = TestPlan::build(&product, workspace_root()).unwrap();
        for unit in plan.units().iter().filter(|unit| {
            unit.arguments()
                .starts_with(&["nextest".into(), "run".into()])
        }) {
            assert!(unit
                .arguments()
                .iter()
                .any(|argument| argument == "--no-tests=fail"));
        }
    }
}

#[test]
fn duplicate_execution_unit_identities_are_rejected() {
    let product = TestProduct::Smoke;
    let unit = TestExecutionUnit::command("duplicate", workspace_root().into(), "cargo", &["test"]);
    let denial = super::reject_duplicate_units(&product, &[unit.clone(), unit]).unwrap_err();

    assert!(denial.contains("smoke"));
    assert!(denial.contains("duplicate"));
}

#[test]
fn fresh_process_recovery_keeps_its_direct_phase_eight_dispatcher() {
    let process = ci_plan(CiTestLane::ProcessScenario, None);

    assert_eq!(process.units().len(), 1);
    assert!(process.units()[0]
        .arguments()
        .iter()
        .any(|argument| argument == "store_process_scenario"));
}

#[test]
fn guarded_store_targets_enable_their_real_feature() {
    for (product, category) in [
        (TestProduct::Ui, "compile_fail"),
        (
            TestProduct::Ci {
                lane: CiTestLane::Scenario,
                shard: None,
            },
            "process",
        ),
    ] {
        let plan = TestPlan::build(&product, workspace_root()).unwrap();
        let arguments = nextest(&plan).arguments();
        let command = arguments.join(" ");
        // This court declares certification-test-authority on its Store
        // dependency. Selecting it preserves that graph without a second
        // command-line feature configuration.
        assert!(command.contains(&format!(
            "(binary_id(=worth-store-court-store-replication-physical-record-journeys-physical-work::worth_store_court_store_replication_physical_record_journeys_physical_work) & (test(/^{category}::/)))"
        )));
        assert!(!arguments.iter().any(|argument| argument == "--features"));
    }
}

#[test]
fn ci_products_apply_the_ci_profiles_to_cargo_commands() {
    let plan = ci_plan(CiTestLane::Scenario, None);
    let arguments = nextest(&plan).arguments();

    assert!(arguments.windows(2).any(|pair| pair == ["--profile", "ci"]));
    assert!(arguments
        .windows(2)
        .any(|pair| pair == ["--cargo-profile", "ci-test"]));
}

#[test]
fn smoke_plan_uses_exact_real_test_selectors() {
    let plan = TestPlan::build(&TestProduct::Smoke, workspace_root()).unwrap();
    let cases = [
        (
            "worth-store-court-maintenance-recovery-runtime-replication",
            "worth_store_court_maintenance_recovery_runtime_replication",
            "certification::io_scheduling::access_policy::certification_materializes_successful_execution_receipts",
        ),
        (
            "worth-store-court-maintenance-recovery-runtime-replication",
            "worth_store_court_maintenance_recovery_runtime_replication",
            "certification::layout_access::layout_corruption_rebuild::root_manifest_rebuild_reaches_exact_parity_through_production_owners",
        ),
        (
            "worth-store-court-maintenance-recovery-runtime-replication",
            "worth_store_court_maintenance_recovery_runtime_replication",
            "certification::operational_security::security_scope_propagation::stable_read_scope_survives_protection_observation_and_decode_entry",
        ),
        (
            "worth-store-court-maintenance-recovery-runtime-replication",
            "worth_store_court_maintenance_recovery_runtime_replication",
            "certification::physical_isolation::stable_read_plan_admission::proof_bearing_read_plan_admits_before_execution_handle",
        ),
        (
            "worth-store-court-store-replication-physical-blob-journeys",
            "worth_store_court_store_replication_physical_blob_journeys",
            "process::store::physical_record_journeys::baseline_admission::empty_bootstrap_create_and_reopen_converge",
        ),
        (
            "worth-store-court-store-replication-physical-blob-journeys",
            "worth_store_court_store_replication_physical_blob_journeys",
            "process::store::physical_record_journeys::scan_journeys::scan_batch_widths_converge_to_one_physical_sequence",
        ),
        (
            "worth-store-court-recovery-runtime-replication-store-physical-record-journeys-durability-admission",
            "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_durability_admission",
            "process::store::physical_record_journeys::physical_work::failure::indeterminate_write_obligation_survives_orderly_close_and_reopen",
        ),
        (
            "worth-store-court-store-replication-physical-record-journeys-physical-work",
            "worth_store_court_store_replication_physical_record_journeys_physical_work",
            "process::store::physical_record_journeys::physical_work::serving_frame_residency::pins_distinguish_faults_hits_overpin_and_refault_without_another_runtime",
        ),
        (
            "worth-store-court-recovery-runtime-replication-store-physical-record-journeys-record-chunk-views",
            "worth_store_court_recovery_runtime_replication_store_physical_record_journeys_record_chunk_views",
            "process::store::physical_record_journeys::record_chunk_views::borrowed_access::inline_view_exposes_only_the_record_payload_and_observational_basis",
        ),
        (
            "worth-store-court-store-replication-physical-blob-journeys",
            "worth_store_court_store_replication_physical_blob_journeys",
            "process::store::physical_record_journeys::record_chunk_views::bounded_copy::bounded_copies_and_views_share_one_cursor_with_exact_copy_evidence",
        ),
        (
            "worth-store-court-store-replication-physical-record-journeys-physical-work",
            "worth_store_court_store_replication_physical_record_journeys_physical_work",
            "process::store::physical_record_journeys::physical_work::residency_writeback_retry::no_effect_writeback_retains_dirty_ownership_through_signal_retry",
        ),
        (
            "worth-store-court-store-replication-physical-record-journeys-physical-work",
            "worth_store_court_store_replication_physical_record_journeys_physical_work",
            "process::store::physical_record_journeys::physical_work::speculative_residency::outcomes::cold_hot_and_mixed_speculation_reconcile_work_media_and_residency_truth",
        ),
    ];
    let expected = cases
        .iter()
        .map(|(package, target, name)| format!("(binary_id(={package}::{target}) & test(={name}))"))
        .collect::<BTreeSet<_>>();
    let observed = plan
        .units()
        .iter()
        .flat_map(|unit| {
            unit.arguments()
                .windows(2)
                .filter(|pair| pair[0] == "--filterset")
                .flat_map(|pair| pair[1].split(" + "))
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();

    assert_eq!(observed.len(), cases.len());
    assert_eq!(observed.into_iter().collect::<BTreeSet<_>>(), expected);
    assert_eq!(plan.units().len(), 5);
    for unit in plan.units() {
        let package = unit.identity().strip_prefix("smoke::").unwrap();
        let target = cases.iter().find(|case| case.0 == package).unwrap().1;
        assert!(unit
            .arguments()
            .windows(2)
            .any(|pair| pair == ["-p", package]));
        assert!(unit
            .arguments()
            .windows(2)
            .any(|pair| pair == ["--test", target]));
        assert!(!unit
            .arguments()
            .iter()
            .any(|argument| argument == "--features"));
    }
}

fn ci_plan(lane: CiTestLane, shard: Option<(usize, usize)>) -> TestPlan {
    TestPlan::build(&TestProduct::Ci { lane, shard }, workspace_root()).unwrap()
}

fn nextest(plan: &TestPlan) -> &TestExecutionUnit {
    plan.units()
        .iter()
        .find(|unit| {
            unit.arguments()
                .starts_with(&["nextest".into(), "run".into()])
        })
        .expect("product must own one nextest command")
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}
