use worth_harness::facade::ExecutionProfile;

use crate::harness::adapter::BridgeHarnessTargetId;
use crate::source::SourceDeclarationIdentity;

use super::execution_harness::execute_harness_run;
use super::mixed_family_fixtures::{mixed_policy_fixture, mixed_source_fixture};

#[test]
fn bridge_m13_mixed_offline_certification_exports_every_family_without_json_proof_shortcuts() {
    let baseline = ExecutionProfile::development("baseline");

    let source_control = execute_harness_run(
        mixed_source_fixture("bridge-m13-mixed-source"),
        baseline.clone(),
        "mixed-source-control",
        BridgeHarnessTargetId::source_materialize(SourceDeclarationIdentity::admit_bridge_owned(
            "source:analysis-history",
        )),
    );
    let source_replay = execute_harness_run(
        mixed_source_fixture("bridge-m13-mixed-source"),
        baseline.clone(),
        "mixed-source-replay",
        BridgeHarnessTargetId::source_replay(SourceDeclarationIdentity::admit_bridge_owned(
            "source:analysis-history",
        )),
    );
    let source_hostile = execute_harness_run(
        mixed_source_fixture("bridge-m13-mixed-source"),
        baseline.clone(),
        "mixed-source-hostile",
        BridgeHarnessTargetId::source_reject_unregistered(
            SourceDeclarationIdentity::admit_bridge_owned("source:hostile-missing"),
        ),
    );
    let policy_control = execute_harness_run(
        mixed_policy_fixture("bridge-m13-mixed-policy"),
        baseline.clone(),
        "mixed-policy-control",
        BridgeHarnessTargetId::policy_provenance_certification(),
    );
    let policy_replay = execute_harness_run(
        mixed_policy_fixture("bridge-m13-mixed-policy"),
        ExecutionProfile::development("sections-canonical")
            .with_metadata("policy_builder_load_order", "sections_canonical"),
        "mixed-policy-replay",
        BridgeHarnessTargetId::policy_provenance_certification(),
    );
    let policy_hostile = execute_harness_run(
        mixed_policy_fixture("bridge-m13-mixed-policy"),
        baseline.clone(),
        "mixed-policy-hostile",
        BridgeHarnessTargetId::policy_rejection_certification(),
    );

    assert!(source_control.summary.is_object());
    assert!(source_replay.summary.is_object());
    assert_eq!(policy_control.summary, policy_replay.summary);

    assert!(source_hostile
        .extensions
        .contains_key("bridge_source_rejection"));
    assert!(policy_hostile
        .extensions
        .contains_key("bridge_policy_certification_bundle"));
}
