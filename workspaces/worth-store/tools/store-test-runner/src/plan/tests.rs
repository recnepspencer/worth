use std::path::Path;

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
fn scenario_plan_names_real_targets_without_absorbing_special_lanes() {
    let plan = ci_plan(CiTestLane::Scenario, None);
    let command = nextest(&plan).arguments().join(" ");

    assert!(command.contains("binary_id(=worth-store::physical_record_journeys)"));
    assert!(command.contains("binary_id(=worth-store-recovery-runtime::phase_five_faults)"));
    assert!(!command.contains("phase_eight_process"));
    assert!(!command.contains("physical_runtime_authority_ui"));
    assert!(!command.contains("backend_assumptions"));
}

#[test]
fn ui_and_formal_products_map_to_their_real_targets() {
    let ui = TestPlan::build(&TestProduct::Ui, workspace_root()).unwrap();
    let ui_command = nextest(&ui).arguments().join(" ");
    assert!(ui_command.contains("physical_runtime_authority_ui"));

    let formal = ci_plan(CiTestLane::Formal, None);
    let formal_command = nextest(&formal).arguments().join(" ");
    assert!(formal_command.contains("backend_assumptions"));
    assert!(formal_command.contains("compaction_visibility_owner_execution"));
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
    for product in [
        TestProduct::Ui,
        TestProduct::Ci {
            lane: CiTestLane::Scenario,
            shard: None,
        },
    ] {
        let plan = TestPlan::build(&product, workspace_root()).unwrap();
        assert!(nextest(&plan)
            .arguments()
            .windows(2)
            .any(|pair| { pair == ["--features", "worth-store/certification-test-authority"] }));
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
    let store = plan
        .units()
        .iter()
        .find(|unit| unit.identity() == "smoke::worth-store")
        .unwrap();
    let command = store.arguments().join(" ");

    assert!(command.contains("binary_id(=worth-store::physical_record_journeys)"));
    assert!(
        command.contains("test(=baseline_admission::empty_bootstrap_create_and_reopen_converge)")
    );
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
