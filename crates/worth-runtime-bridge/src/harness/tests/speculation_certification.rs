use worth_harness::facade::{
    parity_suite, ExecutionProfile, ExecutionRequest, HarnessAdapter, RunRecord, ScenarioPlan,
};

use crate::facade::BridgeRuntimePolicy;
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth, ScenarioRelationalTruth};

fn speculation_fixture(
    name: &str,
    policy: BridgeRuntimePolicy,
) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    let (truth, _commit) = ScenarioRelationalTruth::named("alice", false);
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()])
            .with_policy(policy)
            .with_relational_truth(truth),
    )
    .declare_input("speculation")
    .declare_observation("speculation")
    .compile()
}

fn direct_host_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(name)
}

fn wrapped_host_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(format!("{name}-wrapped-host"))
        .with_metadata("source_adapter_shape", "wrapped")
}

fn forensic_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::forensic(name)
}

fn execute_speculation_run(
    fixture_name: &str,
    policy: BridgeRuntimePolicy,
    profile: ExecutionProfile,
    request_name: &str,
    target: BridgeHarnessTargetId,
) -> RunRecord<BridgeHarnessTargetId> {
    let adapter = BridgeHarnessAdapter;
    let fixture = speculation_fixture(fixture_name, policy);
    let mut runtime = adapter
        .create_runtime()
        .expect("speculation harness runtime");
    adapter
        .prepare_runtime(&mut runtime, &profile)
        .expect("speculation harness prepare");
    adapter
        .load_fixture(&mut runtime, &fixture)
        .expect("speculation harness load fixture");
    adapter
        .execute(
            &mut runtime,
            &fixture,
            &ExecutionRequest::target(request_name, target),
            &profile,
        )
        .expect("speculation harness execution")
}

fn assert_speculation_parity_suite_matches(
    fixture_name: &str,
    request_name: &str,
    target: BridgeHarnessTargetId,
) {
    let report = parity_suite(
        BridgeHarnessAdapter,
        speculation_fixture(fixture_name, BridgeRuntimePolicy::development()),
        ExecutionRequest::target(request_name, target),
        direct_host_profile("baseline-direct-host"),
    )
    .candidates([wrapped_host_profile("candidate")])
    .compare()
    .expect("speculation certification parity suite should compare cleanly");

    assert!(report.matched);
    assert_eq!(report.results.len(), 1);
}

fn assert_terminal_exports_stay_stable_between_hosts(
    target: BridgeHarnessTargetId,
    request_name: &str,
) {
    let control_run = execute_speculation_run(
        &format!("{request_name}-direct"),
        BridgeRuntimePolicy::development(),
        direct_host_profile("baseline-direct-host"),
        request_name,
        target.clone(),
    );
    let replay_run = execute_speculation_run(
        &format!("{request_name}-wrapped"),
        BridgeRuntimePolicy::development(),
        wrapped_host_profile("candidate"),
        request_name,
        target,
    );

    assert_eq!(control_run.summary, replay_run.summary);
    assert_eq!(control_run.extensions, replay_run.extensions);
}

fn assert_terminal_exports_keep_forensic_policy_shape(
    target: BridgeHarnessTargetId,
    request_name: &str,
) {
    let control_run = execute_speculation_run(
        &format!("{request_name}-control"),
        BridgeRuntimePolicy::development(),
        direct_host_profile("baseline-direct-host"),
        request_name,
        target.clone(),
    );
    let forensic_run = execute_speculation_run(
        &format!("{request_name}-forensic"),
        BridgeRuntimePolicy::forensic(),
        forensic_profile("hostile-forensic"),
        request_name,
        target,
    );

    assert_eq!(
        control_run.requested_targets,
        forensic_run.requested_targets
    );
    assert_eq!(control_run.status, forensic_run.status);
    assert_eq!(control_run.outcome, forensic_run.outcome);
    assert!(control_run
        .extensions
        .contains_key("bridge_speculation_certification_bundle"));
    assert!(forensic_run
        .extensions
        .contains_key("bridge_speculation_certification_bundle"));
}

#[test]
fn speculative_discard_certification_exports_are_host_parity_safe() {
    assert_speculation_parity_suite_matches(
        "bridge-speculation-discard-certification",
        "speculation-discard-control",
        BridgeHarnessTargetId::speculation_discard_certification(),
    );
    assert_terminal_exports_stay_stable_between_hosts(
        BridgeHarnessTargetId::speculation_discard_certification(),
        "speculation-discard-control",
    );
    assert_terminal_exports_keep_forensic_policy_shape(
        BridgeHarnessTargetId::speculation_discard_certification(),
        "speculation-discard-forensic",
    );
}

#[test]
fn speculative_promotion_certification_exports_are_host_parity_safe() {
    assert_speculation_parity_suite_matches(
        "bridge-speculation-promotion-certification",
        "speculation-promotion-control",
        BridgeHarnessTargetId::speculation_promotion_certification(),
    );
    assert_terminal_exports_stay_stable_between_hosts(
        BridgeHarnessTargetId::speculation_promotion_certification(),
        "speculation-promotion-control",
    );
    assert_terminal_exports_keep_forensic_policy_shape(
        BridgeHarnessTargetId::speculation_promotion_certification(),
        "speculation-promotion-forensic",
    );
}

#[test]
fn preview_lifecycle_churn_certification_exports_are_host_parity_safe() {
    assert_speculation_parity_suite_matches(
        "bridge-speculation-churn-certification",
        "speculation-churn-control",
        BridgeHarnessTargetId::speculation_churn_certification(),
    );
    assert_terminal_exports_stay_stable_between_hosts(
        BridgeHarnessTargetId::speculation_churn_certification(),
        "speculation-churn-control",
    );
    assert_terminal_exports_keep_forensic_policy_shape(
        BridgeHarnessTargetId::speculation_churn_certification(),
        "speculation-churn-forensic",
    );
}
