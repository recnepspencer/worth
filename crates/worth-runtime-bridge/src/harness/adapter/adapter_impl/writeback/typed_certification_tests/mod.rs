mod duplicate_authority;
mod family_extension;
mod feedback_loop;
mod mapper_parity;
mod other_certifications;
mod replay_loop;

use worth_harness::facade::{ExecutionProfile, HarnessAdapter, ScenarioPlan};

use crate::facade::{
    BridgeCommittedPatchEnvelope, BridgeMappingId, BridgeMappingRegistration,
    BridgeProducerMetadata, BridgeRuntimePolicy, MappingSelector, SignalInvalidationScope,
    SnapshotReadRecord, SnapshotReadRequest, TruthPatchScope,
};
use crate::harness::adapter::BridgeHarnessAdapter;
use crate::harness::fixtures::{BridgeHarnessFixture, SnapshotFixture};

use super::*;

fn certified_execution(
    target: WritebackHarnessTarget,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WritebackHarnessExecution {
    let adapter = BridgeHarnessAdapter;
    let fixture = writeback_fixture("typed-writeback-certification");
    let mut runtime = adapter
        .create_runtime()
        .expect("writeback typed certification runtime");
    adapter
        .prepare_runtime(&mut runtime, &ExecutionProfile::development("typed-host"))
        .expect("writeback typed certification prepare");
    adapter
        .load_fixture(&mut runtime, &fixture)
        .expect("writeback typed certification fixture");
    let runtime_bridge = runtime
        .runtime
        .as_ref()
        .expect("writeback typed certification bridge");
    execute_writeback_request(
        &runtime,
        runtime_bridge,
        &fixture.fixture,
        target,
        resource_request,
    )
    .expect("writeback typed certification execution")
}

fn writeback_fixture(name: &str) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(BridgeRuntimePolicy::development())
            .with_committed_patch(committed_patch())
            .with_snapshot(snapshot()),
    )
    .declare_input("writeback")
    .declare_observation("writeback")
    .compile()
}

fn registration() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("typed-profile-name"),
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
        SignalInvalidationScope::admit_bridge_owned("signal.profile.typed"),
        crate::facade::CoarseRoutingMode::Direct,
    )
}

fn committed_patch() -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            BridgeProducerMetadata::bridge_harness_fixture(),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
        ),
        vec![crate::facade::BridgeCommittedPatchItem::with_target(
            "user",
            crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                worth_foundational::facade::AspectLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid typed certification aspect key"),
                ),
                worth_foundational::facade::CanonicalFieldPath::single(
                    worth_foundational::facade::FieldKey::new("name".to_owned())
                        .expect("valid foundational field key"),
                ),
            ),
        )],
    )
    .expect("typed certification committed patch")
}

fn snapshot() -> SnapshotFixture {
    SnapshotFixture::new(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        vec![SnapshotReadRecord::for_request(
            &SnapshotReadRequest::for_coarse(
                "user",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid writeback snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            ),
            worth_foundational::facade::AspectValue::String("alice".into()),
        )],
    )
}
