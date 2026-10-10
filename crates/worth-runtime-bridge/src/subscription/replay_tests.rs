use crate::facade::{
    BridgeCommittedPatchEnvelope, BridgeRuntimePolicy, BridgeSubscriptionBasisRequest,
    BridgeSubscriptionDeliveryIntentClass, NormalizedSubscriptionSliceIntent,
};
use crate::input::envelope::{
    BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem, TruthBranchIdentity,
    TruthPatchIdentity,
};
use crate::mapping::SubscriptionSliceKind;
use crate::snapshot::{SnapshotReadPacket, TruthSnapshotIdentity, TruthSnapshotReader};
use crate::subscription::BridgeSubscriptionDeclarationFamilyKind;
use crate::truth_identity_fixtures::{truth_branch, truth_snapshot};

use super::{
    super::BridgeSubscriptionLifecycleRecord, super::BridgeSubscriptionLifecycleStateKind,
    BridgeRetainedSubscriptionBundle, BridgeSubscriptionReplayMismatchKind,
    BridgeSubscriptionReplaySummary,
};

const REPLAY_SNAPSHOT: fn() -> TruthSnapshotIdentity = || truth_snapshot(1, 1);
const REPLAY_BRANCH: fn() -> TruthBranchIdentity = || truth_branch("analysis");

#[derive(Clone)]
struct StaticSource;

struct StaticSnapshotReader;

impl TruthSnapshotReader for StaticSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        REPLAY_SNAPSHOT()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::snapshot::SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError>
    {
        Ok(crate::snapshot::SnapshotReadPacketResult::new(
            REPLAY_SNAPSHOT(),
            request
                .reads()
                .iter()
                .map(|read| {
                    crate::snapshot::SnapshotReadRecord::for_request(
                        read,
                        worth_foundational::facade::AspectValue::String("fixture-value".into()),
                    )
                })
                .collect(),
        ))
    }
}

impl crate::adapter::CommittedPatchSource for StaticSource {
    fn load_committed_patch(
        &self,
        request: crate::adapter::RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, crate::adapter::RelationalBridgeSourceError> {
        BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new(
                request.commit_identity().clone(),
                TruthPatchIdentity::from_relational_patch_position(1),
                REPLAY_SNAPSHOT(),
                REPLAY_BRANCH(),
            ),
            vec![profile_name_patch_item("entity-1")],
        )
        .map_err(|error| {
            crate::adapter::RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                error.to_string(),
            )
        })
    }
}

impl crate::adapter::SnapshotReadSource for StaticSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, crate::adapter::RelationalBridgeSourceError> {
        if snapshot_matches_replay_fixture(identity) {
            Ok(Box::new(StaticSnapshotReader))
        } else {
            Err(crate::adapter::RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                format!("unknown snapshot `{}`", identity.as_str()),
            ))
        }
    }
}

impl crate::adapter::TruthBranchHeadSource for StaticSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, crate::adapter::RelationalBridgeSourceError> {
        BridgeCommittedPatchEnvelope::new(
            BridgeCommittedPatchEnvelopeIdentity::new(
                crate::facade::TruthCommitIdentity::from_relational_commit_id(100),
                TruthPatchIdentity::from_relational_patch_position(100),
                REPLAY_SNAPSHOT(),
                branch_identity.clone(),
            ),
            vec![profile_name_patch_item("entity-1")],
        )
        .map_err(|error| {
            crate::adapter::RelationalBridgeSourceError::new(
                crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                error.to_string(),
            )
        })
    }
}

struct StaticSink;

impl crate::adapter::InvalidationSink for StaticSink {
    fn deliver_invalidation(
        &self,
        delivery: crate::routing::BridgeSignalInvalidationDelivery,
        _lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::delivery::BridgeDeliveryReceipt, crate::adapter::SignalBridgeSinkError> {
        Ok(crate::delivery::BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

fn snapshot_matches_replay_fixture(identity: &TruthSnapshotIdentity) -> bool {
    identity
        .relational_snapshot_parts()
        .is_some_and(|parts| parts.snapshot_id() == 1 && parts.version_id() == 1)
}

fn profile_name_patch_item(entity_identity: &str) -> BridgeCommittedPatchItem {
    BridgeCommittedPatchItem::with_target(
        entity_identity,
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
    )
}

fn runtime(policy: BridgeRuntimePolicy) -> crate::facade::RuntimeBridge {
    crate::builder::RuntimeBridgeBuilder::new()
        .with_policy(policy)
        .with_relational_source(StaticSource)
        .with_truth_branch_head_source(StaticSource)
        .with_signal_sink(StaticSink)
        .register_mapping(crate::mapping::BridgeMappingRegistration::new(
            crate::mapping::BridgeMappingId::admit_bridge_owned("mapping"),
            crate::mapping::TruthPatchScope::for_entity_field(
                crate::mapping::MappingSelector::exact("entity-1"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid native field key"),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            crate::mapping::SignalInvalidationScope::admit_bridge_owned("signal:profile"),
            crate::mapping::CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("runtime should build")
}

fn declare_detail(
    runtime: &crate::facade::RuntimeBridge,
) -> crate::facade::BridgeSubscriptionDeclaration {
    runtime
        .declare_subscription(
            BridgeSubscriptionDeclarationFamilyKind::DetailExact,
            vec![NormalizedSubscriptionSliceIntent::try_new_entity_field(
                "entity-1",
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native subscription aspect key"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid native subscription field key"),
                SubscriptionSliceKind::SignalField,
            )
            .expect("slice intent should validate")],
            BridgeSubscriptionDeliveryIntentClass::None,
        )
        .expect("detail declaration should succeed")
}

fn declare_collection(
    runtime: &crate::facade::RuntimeBridge,
) -> crate::facade::BridgeSubscriptionDeclaration {
    runtime
        .declare_subscription(
            BridgeSubscriptionDeclarationFamilyKind::CollectionMembership,
            vec![NormalizedSubscriptionSliceIntent::try_new_entity_region(
                "entity-1",
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native subscription aspect key"),
                SubscriptionSliceKind::SignalRegion,
            )
            .expect("slice intent should validate")],
            BridgeSubscriptionDeliveryIntentClass::None,
        )
        .expect("collection declaration should succeed")
}

#[test]
fn replay_rejects_lifecycle_admitted_mismatch() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let detail = declare_detail(&runtime);
    let detail_admitted = runtime
        .admit_subscription(
            &detail,
            BridgeSubscriptionBasisRequest::snapshot(REPLAY_SNAPSHOT()),
            execution,
        )
        .expect("detail admission should succeed");
    let collection = declare_collection(&runtime);
    let collection_admitted = runtime
        .admit_subscription(
            &collection,
            BridgeSubscriptionBasisRequest::branch_head(REPLAY_BRANCH()),
            execution,
        )
        .expect("collection admission should succeed");

    let mismatched_lifecycle = BridgeSubscriptionLifecycleRecord::new(
        &collection_admitted,
        BridgeSubscriptionLifecycleStateKind::ActivationReady,
    );
    let bundle = BridgeRetainedSubscriptionBundle::new(
        runtime.subscription_family_registry_identity().clone(),
        &detail_admitted,
        mismatched_lifecycle,
    );

    let mismatch = BridgeSubscriptionReplaySummary::replay(
        runtime.subscription_family_registry_identity(),
        &bundle,
    )
    .expect_err("mismatched lifecycle should reject");

    assert_eq!(
        mismatch.mismatch_kind(),
        BridgeSubscriptionReplayMismatchKind::LifecycleAdmittedMismatch
    );
}
