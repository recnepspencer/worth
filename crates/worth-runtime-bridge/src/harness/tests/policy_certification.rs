use worth_harness::facade::{
    parity_suite, ExecutionProfile, ExecutionRequest, HarnessAdapter, RunRecord, ScenarioPlan,
};

use crate::facade::BridgeRuntimePolicy;
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth, ScenarioRelationalTruth};

fn policy_fixture(
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
    .declare_input("policy")
    .declare_observation("policy")
    .compile()
}

fn direct_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(name)
}

fn sections_canonical_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(format!("{name}-sections-canonical"))
        .with_metadata("policy_builder_load_order", "sections_canonical")
}

fn sections_reverse_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::development(format!("{name}-sections-reverse"))
        .with_metadata("policy_builder_load_order", "sections_reverse")
}

fn forensic_policy_profile(name: &str) -> ExecutionProfile {
    ExecutionProfile::forensic(format!("{name}-forensic"))
        .with_metadata("policy_builder_load_order", "sections_reverse")
}

fn execute_policy_run(
    fixture_name: &str,
    policy: BridgeRuntimePolicy,
    profile: ExecutionProfile,
    request_name: &str,
    target: BridgeHarnessTargetId,
) -> RunRecord<BridgeHarnessTargetId> {
    let adapter = BridgeHarnessAdapter;
    let fixture = policy_fixture(fixture_name, policy);
    let mut runtime = adapter.create_runtime().expect("policy harness runtime");
    adapter
        .prepare_runtime(&mut runtime, &profile)
        .expect("policy harness prepare");
    adapter
        .load_fixture(&mut runtime, &fixture)
        .expect("policy harness load fixture");
    adapter
        .execute(
            &mut runtime,
            &fixture,
            &ExecutionRequest::target(request_name, target),
            &profile,
        )
        .expect("policy harness execution")
}

fn assert_policy_terminal_export_present(run: &RunRecord<BridgeHarnessTargetId>) {
    assert!(run.summary.is_object());
    assert!(run
        .extensions
        .contains_key("bridge_policy_certification_bundle"));
}

#[test]
fn policy_provenance_equivalence_bundle_is_builder_order_and_replay_stable() {
    let target = BridgeHarnessTargetId::policy_provenance_certification();
    let fixture = policy_fixture(
        "bridge-policy-provenance-certification",
        BridgeRuntimePolicy::development(),
    );
    let request = ExecutionRequest::target("policy-provenance-control", target.clone());

    let report = parity_suite(
        BridgeHarnessAdapter,
        fixture,
        request,
        direct_profile("baseline-direct-host"),
    )
    .candidates([sections_canonical_profile("candidate")])
    .compare()
    .expect("policy provenance parity suite should compare cleanly");

    assert!(report.matched);
    assert_eq!(report.results.len(), 1);

    let control_run = execute_policy_run(
        "bridge-policy-provenance-control",
        BridgeRuntimePolicy::development(),
        direct_profile("baseline-direct-host"),
        "policy-provenance-control",
        target.clone(),
    );
    let replay_run = execute_policy_run(
        "bridge-policy-provenance-replay",
        BridgeRuntimePolicy::development(),
        sections_canonical_profile("candidate"),
        "policy-provenance-replay",
        target.clone(),
    );
    let hostile_run = execute_policy_run(
        "bridge-policy-provenance-hostile",
        BridgeRuntimePolicy::development(),
        forensic_policy_profile("hostile"),
        "policy-provenance-hostile",
        target,
    );

    assert_eq!(control_run.summary, replay_run.summary);
    assert_eq!(control_run.extensions, replay_run.extensions);
    assert_eq!(control_run.summary, hostile_run.summary);
    assert_policy_terminal_export_present(&control_run);
}

#[test]
fn policy_rejection_bundle_stays_typed_and_leaves_zero_authority_escape_residue() {
    let target = BridgeHarnessTargetId::policy_rejection_certification();

    let control_run = execute_policy_run(
        "bridge-policy-rejection-control",
        BridgeRuntimePolicy::development(),
        direct_profile("baseline-direct-host"),
        "policy-rejection-control",
        target.clone(),
    );
    let replay_run = execute_policy_run(
        "bridge-policy-rejection-replay",
        BridgeRuntimePolicy::development(),
        sections_canonical_profile("candidate"),
        "policy-rejection-replay",
        target.clone(),
    );
    let hostile_run = execute_policy_run(
        "bridge-policy-rejection-hostile",
        BridgeRuntimePolicy::development(),
        sections_reverse_profile("hostile"),
        "policy-rejection-hostile",
        target,
    );

    assert_eq!(control_run.summary, replay_run.summary);
    assert_eq!(control_run.extensions, replay_run.extensions);
    assert_eq!(control_run.summary, hostile_run.summary);
    assert_policy_terminal_export_present(&control_run);
}

#[test]
fn ambient_policy_leak_resistance_bundle_preserves_preview_equivalence_under_interleave() {
    let target = BridgeHarnessTargetId::policy_ambient_leak_certification();
    let fixture = policy_fixture(
        "bridge-policy-ambient-leak-certification",
        BridgeRuntimePolicy::development(),
    );
    let request = ExecutionRequest::target("policy-ambient-leak-control", target.clone());

    let report = parity_suite(
        BridgeHarnessAdapter,
        fixture,
        request,
        direct_profile("baseline-direct-host"),
    )
    .candidates([sections_reverse_profile("candidate")])
    .compare()
    .expect("policy ambient leak parity suite should compare cleanly");

    assert!(report.matched);
    assert_eq!(report.results.len(), 1);

    let control_run = execute_policy_run(
        "bridge-policy-ambient-leak-control",
        BridgeRuntimePolicy::development(),
        direct_profile("baseline-direct-host"),
        "policy-ambient-leak-control",
        target.clone(),
    );
    let replay_run = execute_policy_run(
        "bridge-policy-ambient-leak-replay",
        BridgeRuntimePolicy::development(),
        sections_reverse_profile("candidate"),
        "policy-ambient-leak-replay",
        target,
    );

    assert_eq!(control_run.summary, replay_run.summary);
    assert_eq!(control_run.extensions, replay_run.extensions);
    assert_policy_terminal_export_present(&control_run);
}
