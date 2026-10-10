use super::*;
use crate::facade::{
    BridgeMappingId, BridgeMappingRegistration, BridgeProducerMetadata, BridgeTruthViewSelector,
    CoarseRoutingMode, MappingSelector, SignalInvalidationScope, SnapshotReadRecord,
    SnapshotReadRequest, StructuralFingerprintEquivalenceContract, StructuralFingerprintFamily,
    StructuralFingerprintNormalizationRule, StructuralFingerprintOmissionPolicy,
    StructuralFingerprintOrderingRule, StructuralIdentityDeclaration,
    StructuralIdentityDeclarationIdentity, StructuralSchemaIdentity, StructuralTruthViewBasis,
    TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity, TruthPatchScope,
    TruthSnapshotIdentity,
};
use crate::harness::fixtures::{BridgeHarnessFixture, SnapshotFixture};
use std::collections::BTreeSet;
use worth_harness::facade::{ExecutionProfile, ScenarioPlan};
use worth_harness::runtime::HarnessAdapter;

fn execute(
    target: StructuralHarnessTarget,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> StructuralHarnessExecution {
    let adapter = crate::harness::adapter::BridgeHarnessAdapter;
    let fixture = structural_fixture();
    let mut runtime = adapter.create_runtime().expect("structural runtime");
    let profile = ExecutionProfile::development("typed-structural-certification");
    adapter
        .prepare_runtime(&mut runtime, &profile)
        .expect("structural prepare");
    adapter
        .load_fixture(&mut runtime, &fixture)
        .expect("structural fixture");
    let runtime_bridge = runtime.runtime.as_ref().expect("runtime bridge");
    execute_structural_request(runtime_bridge, &fixture.fixture, target, resource_request)
        .expect("typed structural execution")
}

#[test]
fn structural_remap_certification_is_typed_before_terminal_export() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let execution = execute(
        StructuralHarnessTarget::RemapExact {
            declaration_identity:
                crate::structural::StructuralIdentityDeclarationIdentity::admit_bridge_owned(
                    "structural:analysis-remap",
                ),
        },
        resource_request,
    );
    let summary = execution.summary();
    let bundle = execution.certification_bundle();

    assert!(summary.structural_match_digest.is_some());
    assert_eq!(
        bundle.structural_match_digest,
        summary.structural_match_digest
    );
    assert_eq!(
        bundle.remap_artifact_digest,
        summary.structural_reuse_digest
    );
    assert_eq!(
        bundle.structural_reuse_digest,
        summary.structural_reuse_digest
    );
    assert!(bundle.ambiguity_report.is_none());
    assert!(bundle.identity_separation_report.is_none());
    assert_eq!(bundle.counter_snapshot.structural_replay_mismatch_count, 0);
}

fn structural_fixture() -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    ScenarioPlan::new(
        "typed-structural-certification",
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_structural_declaration(remap_declaration())
            .with_structural_declaration(branch_declaration())
            .with_committed_patch(committed_patch_on_branch(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ))
            .with_committed_patch(committed_patch_on_branch(
                crate::truth_identity_fixtures::truth_branch_fixture("left"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-left-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-left-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ))
            .with_committed_patch(committed_patch_on_branch(
                crate::truth_identity_fixtures::truth_branch_fixture("right"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-right-b"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-right-b"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
            ))
            .with_snapshot(snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "alice",
            ))
            .with_snapshot(snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
                "bob",
            )),
    )
    .declare_input("structural")
    .declare_observation("structural")
    .compile()
}

fn registration() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("profile-name"),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact("user"),
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            worth_foundational::facade::FieldKey::new("name".to_owned())
                .expect("valid native field key"),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
        SignalInvalidationScope::admit_bridge_owned("signal.profile"),
        CoarseRoutingMode::Direct,
    )
}

fn committed_patch_on_branch(
    branch_identity: TruthBranchIdentity,
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
) -> crate::facade::BridgeCommittedPatchEnvelope {
    crate::facade::BridgeCommittedPatchEnvelope::new(
        crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            BridgeProducerMetadata::bridge_harness_fixture(),
            commit_identity,
            patch_identity,
            snapshot_identity,
            branch_identity,
        ),
        vec![crate::facade::BridgeCommittedPatchItem::with_target(
            "user",
            crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                worth_foundational::facade::AspectLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid bridge patch aspect key"),
                ),
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("name".to_owned())
                        .expect("valid foundational field key"),
                ),
            ),
        )],
    )
    .expect("harness committed patch envelope should construct")
}

fn snapshot(snapshot_identity: TruthSnapshotIdentity, value: &str) -> SnapshotFixture {
    let mismatch_identity = snapshot_identity.as_str().to_owned();
    SnapshotFixture::new(
        snapshot_identity,
        vec![
            structural_snapshot_record(
                "entity-1",
                worth_foundational::facade::AspectValue::String((value).into()),
            ),
            structural_snapshot_record(
                "entity-2",
                worth_foundational::facade::AspectValue::String((value).into()),
            ),
            structural_snapshot_record(
                "entity-3",
                worth_foundational::facade::AspectValue::String(
                    (format!("shape-mismatch-{mismatch_identity}")).into(),
                ),
            ),
        ],
    )
}

fn structural_snapshot_record(
    entity_identity: &str,
    value: worth_foundational::facade::AspectValue,
) -> SnapshotReadRecord {
    SnapshotReadRecord::for_request(
        &SnapshotReadRequest::for_coarse(
            entity_identity,
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid typed structural aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
        ),
        value,
    )
}

fn remap_declaration() -> StructuralIdentityDeclaration {
    StructuralIdentityDeclaration::advisory_remap(
        StructuralIdentityDeclarationIdentity::admit_bridge_owned("structural:analysis-remap"),
        StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
        fingerprint_contract(StructuralFingerprintFamily::TopologyFingerprint),
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    )
}

fn branch_declaration() -> StructuralIdentityDeclaration {
    StructuralIdentityDeclaration::branch_comparison(
        StructuralIdentityDeclarationIdentity::admit_bridge_owned(
            "structural:analysis-branch-compare",
        ),
        StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
        fingerprint_contract(StructuralFingerprintFamily::BranchComparisonFingerprint),
        StructuralTruthViewBasis::explicit_branch_pair(
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("left"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("right"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
            ),
        ),
    )
}

fn fingerprint_contract(
    family: StructuralFingerprintFamily,
) -> StructuralFingerprintEquivalenceContract {
    StructuralFingerprintEquivalenceContract::new(
        StructuralSchemaIdentity::admit_bridge_owned("schema:geometry"),
        family,
        "geometry-typed-certification-v1",
        StructuralFingerprintNormalizationRule::SchemaDeclaredCanonicalForm,
        StructuralFingerprintOrderingRule::SchemaDeclaredCanonicalOrder,
        StructuralFingerprintOmissionPolicy::SchemaDeclaredOmissionPolicy,
    )
}

#[test]
fn structural_ambiguity_certification_retains_typed_rejection_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let execution = execute(
        StructuralHarnessTarget::RemapAmbiguous {
            declaration_identity:
                crate::structural::StructuralIdentityDeclarationIdentity::admit_bridge_owned(
                    "structural:analysis-remap",
                ),
        },
        resource_request,
    );
    let summary = execution.summary();
    let bundle = execution.certification_bundle();
    let ambiguity = bundle
        .ambiguity_report
        .as_ref()
        .expect("typed ambiguity report");

    assert_eq!(bundle.failure_digest, summary.failure_digest);
    assert_eq!(
        ambiguity.outcome_class,
        crate::structural::StructuralMatchOutcomeClass::RejectedAmbiguousStructuralMatch
    );
    assert_eq!(
        ambiguity.retained_candidates.len(),
        bundle.counter_snapshot.structural_candidate_count
    );
    assert!(
        ambiguity
            .retained_candidates
            .candidates()
            .iter()
            .all(|candidate| !candidate.identity().is_empty()),
        "ambiguity report must retain non-empty typed structural candidate identities before terminal export"
    );
    assert_eq!(
        retained_candidate_identity_count(&ambiguity.retained_candidates),
        ambiguity.retained_candidates.len(),
        "ambiguity report must retain a unique typed candidate set, not a duplicated presentation list"
    );
    assert_eq!(bundle.counter_snapshot.structural_ambiguity_count, 1);
    assert!(bundle.remap_artifact_digest.is_none());
}

#[test]
fn structural_branch_certification_retains_typed_diff_and_replay_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let execution = execute(
        StructuralHarnessTarget::BranchReplay {
            declaration_identity:
                crate::structural::StructuralIdentityDeclarationIdentity::admit_bridge_owned(
                    "structural:analysis-branch-compare",
                ),
        },
        resource_request,
    );
    let summary = execution.summary();
    let bundle = execution.certification_bundle();
    let diff = bundle
        .structural_diff_report
        .as_ref()
        .expect("typed structural diff report");

    assert_eq!(bundle.branch_compare_digest, summary.branch_compare_digest);
    assert_eq!(bundle.replay_digest, summary.replay_digest);
    assert_eq!(diff.branch_diff_count, 1);
    assert!(!diff.retained_candidates.is_empty());
    assert_eq!(
        diff.retained_candidates.len(),
        bundle.counter_snapshot.structural_candidate_count
    );
    assert!(
        diff.retained_candidates
            .candidates()
            .iter()
            .all(|candidate| candidate.identity().starts_with("branch-diff:")),
        "branch diff report must retain typed structural candidate identities before terminal export"
    );
    assert_eq!(
        retained_candidate_identity_count(&diff.retained_candidates),
        diff.retained_candidates.len(),
        "branch diff report must retain a unique typed candidate set, not a duplicated presentation list"
    );
    assert_eq!(bundle.counter_snapshot.structural_replay_request_count, 1);
}

fn retained_candidate_identity_count(
    candidates: &super::certification_bundle::StructuralRetainedCandidateSet,
) -> usize {
    candidates
        .candidates()
        .iter()
        .map(|candidate| candidate.identity())
        .collect::<BTreeSet<_>>()
        .len()
}
