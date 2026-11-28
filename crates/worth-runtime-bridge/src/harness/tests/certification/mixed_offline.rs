use worth_harness::facade::ExecutionProfile;

use crate::harness::adapter::BridgeHarnessTargetId;
use crate::source::SourceDeclarationIdentity;
use crate::structural::StructuralIdentityDeclarationIdentity;

use super::execution_harness::execute_harness_run;
use super::mixed_family_fixtures::{
    mixed_policy_fixture, mixed_source_fixture, mixed_speculation_fixture, mixed_stream_fixture,
    mixed_structural_fixture, mixed_writeback_fixture,
};

#[test]
fn bridge_m13_mixed_offline_certification_exports_every_family_without_json_proof_shortcuts() {
    let baseline = ExecutionProfile::development("baseline");

    let stream_control = execute_harness_run(
        mixed_stream_fixture("bridge-m13-mixed-stream"),
        baseline.clone(),
        "mixed-stream-routing",
        BridgeHarnessTargetId::stream_routing([
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        ]),
    );
    let stream_replay = execute_harness_run(
        mixed_stream_fixture("bridge-m13-mixed-stream"),
        baseline.clone(),
        "mixed-stream-replay",
        BridgeHarnessTargetId::stream_replay_audit([
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        ]),
    );
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
    let structural_control = execute_harness_run(
        mixed_structural_fixture("bridge-m13-mixed-structural"),
        baseline.clone(),
        "mixed-structural-control",
        BridgeHarnessTargetId::structural_remap_exact(
            StructuralIdentityDeclarationIdentity::admit_bridge_owned("structural:analysis-remap"),
        ),
    );
    let structural_replay = execute_harness_run(
        mixed_structural_fixture("bridge-m13-mixed-structural"),
        baseline.clone(),
        "mixed-structural-replay",
        BridgeHarnessTargetId::structural_remap_replay(
            StructuralIdentityDeclarationIdentity::admit_bridge_owned("structural:analysis-remap"),
        ),
    );
    let structural_hostile = execute_harness_run(
        mixed_structural_fixture("bridge-m13-mixed-structural"),
        baseline.clone(),
        "mixed-structural-hostile",
        BridgeHarnessTargetId::structural_remap_ambiguous(
            StructuralIdentityDeclarationIdentity::admit_bridge_owned("structural:analysis-remap"),
        ),
    );
    let preview_control = execute_harness_run(
        mixed_speculation_fixture("bridge-m13-mixed-preview"),
        baseline.clone(),
        "mixed-preview-control",
        BridgeHarnessTargetId::speculation_discard_certification(),
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
    let writeback_control = execute_harness_run(
        mixed_writeback_fixture("bridge-m13-mixed-writeback"),
        baseline.clone(),
        "mixed-writeback-control",
        BridgeHarnessTargetId::writeback_duplicate_certification(),
    );
    let writeback_replay = execute_harness_run(
        mixed_writeback_fixture("bridge-m13-mixed-writeback"),
        baseline.clone(),
        "mixed-writeback-replay",
        BridgeHarnessTargetId::writeback_feedback_loop_certification(),
    );
    let writeback_hostile = execute_harness_run(
        mixed_writeback_fixture("bridge-m13-mixed-writeback"),
        baseline,
        "mixed-writeback-hostile",
        BridgeHarnessTargetId::writeback_authority_denial_certification(),
    );

    assert!(stream_control.summary.is_object());
    assert!(stream_replay.summary.is_object());
    assert!(source_control.summary.is_object());
    assert!(source_replay.summary.is_object());
    assert!(structural_control.summary.is_object());
    assert!(structural_replay.summary.is_object());
    assert_eq!(policy_control.summary, policy_replay.summary);

    assert!(stream_control
        .extensions
        .contains_key("bridge_stream_certification_bundle"));
    assert!(source_hostile
        .extensions
        .contains_key("bridge_source_rejection"));
    assert!(structural_hostile
        .extensions
        .contains_key("bridge_structural_certification_bundle"));
    assert!(preview_control
        .extensions
        .contains_key("bridge_speculation_certification_bundle"));
    assert!(policy_hostile
        .extensions
        .contains_key("bridge_policy_certification_bundle"));
    assert!(writeback_control
        .extensions
        .contains_key("bridge_writeback_certification_bundle"));
    assert!(writeback_replay
        .extensions
        .contains_key("bridge_writeback_certification_bundle"));
    assert!(writeback_hostile
        .extensions
        .contains_key("bridge_writeback_certification_bundle"));
}
