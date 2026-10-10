use worth_foundational::facade::{
    AspectLocator, CanonicalFieldPath, LocatorAuthority, ScalarAspectType,
};
use worth_runtime_bridge::facade::{
    AdmittedBridgeSubscription, AspectKeySelector, BridgeAspectRegistration,
    BridgeAspectRegistrationId, BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity,
    BridgeCommittedPatchItem, BridgeCommittedPatchTarget, BridgeContinuityAuthorityBasis,
    BridgeDeliveredContinuityResult, BridgeLineageContext, BridgeMappingId,
    BridgeMappingRegistration, BridgeProducerMetadata, CoarseRoutingMode, MappingSelector,
    NormalizedSubscriptionSliceIntent, RuntimeBridge, RuntimeBridgeBuilder,
    SignalInvalidationScope, SliceWideningPolicy, SnapshotReadContract, SnapshotReadRecord,
    SubscriptionSliceKind, TruthPatchScope, TruthPatchTargetSelector,
};

use super::identity::{
    aspect_key, field_key, fixture_branch_identity, fixture_commit_identity,
    fixture_patch_identity, fixture_snapshot_identity, COMMIT_A, PHASE_SIX_MAIN_BRANCH, SNAPSHOT_A,
};
use super::{FixedLineageSource, NoopSignalSink, TestRelationalSource};

pub(in crate::lower_runtime_routing::certification::surface::fixtures::phase_six) fn observation_runtime(
) -> RuntimeBridge {
    build_runtime(base_source(), false)
}

pub(in crate::lower_runtime_routing::certification::surface::fixtures::phase_six) fn continuity_runtime(
) -> RuntimeBridge {
    build_runtime(base_source_with_field_slice(), true)
}

pub(in crate::lower_runtime_routing::certification::surface::fixtures::phase_six) fn subscription_runtime(
) -> RuntimeBridge {
    build_runtime(base_source(), false)
}

pub(in crate::lower_runtime_routing::certification::surface::fixtures::phase_six) fn detail_subscription(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> AdmittedBridgeSubscription {
    let declaration = runtime
        .declare_subscription(
            worth_runtime_bridge::facade::BridgeSubscriptionDeclarationFamilyKind::DetailExact,
            vec![NormalizedSubscriptionSliceIntent::try_new_entity_field(
                "entity-1",
                aspect_key("profile"),
                field_key("name"),
                SubscriptionSliceKind::SignalField,
            )
            .expect("detail slice intent should validate")],
            worth_runtime_bridge::facade::BridgeSubscriptionDeliveryIntentClass::None,
        )
        .expect("detail declaration should admit");
    runtime
        .admit_subscription(
            &declaration,
            worth_runtime_bridge::facade::BridgeSubscriptionBasisRequest::branch_head(
                fixture_branch_identity(PHASE_SIX_MAIN_BRANCH),
            ),
            resource_request,
        )
        .expect("branch-head subscription basis should admit")
}

pub(in crate::lower_runtime_routing::certification::surface::fixtures::phase_six) fn delivered_continuity(
    runtime: &RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeDeliveredContinuityResult {
    let route = runtime
        .plan_committed_patch_with_mapping_context(
            worth_runtime_bridge::facade::BridgeRouteRequest::for_commit(fixture_commit_identity(
                COMMIT_A,
            )),
            worth_runtime_bridge::facade::BridgeMappingContext::default().with_lineage_context(
                BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
                    fixture_branch_identity(PHASE_SIX_MAIN_BRANCH),
                    fixture_snapshot_identity(SNAPSHOT_A),
                )),
            ),
            resource_request,
        )
        .expect("continuity route should plan");
    runtime
        .deliver_invalidation(route, resource_request)
        .expect("continuity route should deliver");
    let route_record = runtime
        .diagnostics()
        .last_route_record()
        .expect("continuity route record should be retained");
    runtime
        .deliver_continuity(&route_record)
        .expect("continuity should deliver")
}

fn registration() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("profile-name"),
        TruthPatchScope::new(
            MappingSelector::exact("user"),
            AspectKeySelector::exact(aspect_key("profile")),
            TruthPatchTargetSelector::entity_field(field_key("name")),
        ),
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        SignalInvalidationScope::from_stable_name("signal.profile"),
        CoarseRoutingMode::Direct,
    )
}

fn field_aspect_registration() -> BridgeAspectRegistration {
    BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::from_stable_name("profile-name-field"),
        TruthPatchScope::new(
            MappingSelector::exact("user"),
            AspectKeySelector::exact(aspect_key("profile")),
            TruthPatchTargetSelector::entity_field(field_key("name")),
        ),
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        worth_runtime_bridge::facade::TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    )
}

fn build_runtime(source: TestRelationalSource, with_continuity: bool) -> RuntimeBridge {
    let builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source)
        .with_signal_sink(NoopSignalSink)
        .register_mapping(registration());
    let builder = if with_continuity {
        builder
            .with_continuity_lineage_source(FixedLineageSource)
            .register_aspect_mapping(field_aspect_registration())
    } else {
        builder
    };
    builder
        .build()
        .expect("fixture bridge runtime should build")
}

fn base_source() -> TestRelationalSource {
    let source = TestRelationalSource::default();
    source.insert_committed_patch(committed_patch(
        PHASE_SIX_MAIN_BRANCH,
        COMMIT_A,
        SNAPSHOT_A,
        "name",
    ));
    source.insert_snapshot(SNAPSHOT_A, snapshot_records("user:profile", "alice"));
    source
}

fn base_source_with_field_slice() -> TestRelationalSource {
    let source = TestRelationalSource::default();
    source.insert_committed_patch(committed_patch(
        PHASE_SIX_MAIN_BRANCH,
        COMMIT_A,
        SNAPSHOT_A,
        "name",
    ));
    source.insert_snapshot(
        SNAPSHOT_A,
        snapshot_records("user:profile:signal-field:name", "alice"),
    );
    source
}

fn committed_patch(
    branch: &str,
    commit: &str,
    snapshot: &str,
    surface: &str,
) -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            BridgeProducerMetadata::bridge_harness_fixture(),
            fixture_commit_identity(commit),
            fixture_patch_identity(commit),
            fixture_snapshot_identity(snapshot),
            fixture_branch_identity(branch),
        ),
        vec![BridgeCommittedPatchItem::with_target(
            "user",
            BridgeCommittedPatchTarget::entity_field_path(
                AspectLocator::new(LocatorAuthority::Authoritative, aspect_key("profile")),
                CanonicalFieldPath::single(field_key(surface)),
            ),
        )],
    )
    .expect("readmission fixture committed patch envelope should construct")
}

fn snapshot_records(_key: &str, value: &str) -> Vec<SnapshotReadRecord> {
    let read = worth_runtime_bridge::facade::SnapshotReadRequest::for_coarse(
        "user",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
    );
    vec![SnapshotReadRecord::for_request(
        &read,
        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(value),
    )]
}
