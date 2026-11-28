use crate::facade::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeCommittedPatchEnvelope,
    BridgeMappingId, BridgeMappingRegistration, BridgeProducerMetadata, CoarseRoutingMode,
    InvalidationSink, MappingSelector, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SliceWideningPolicy, SubscriptionSliceKind, TruthBranchIdentity,
    TruthCommitIdentity, TruthDeltaSurfaceKind, TruthPatchIdentity, TruthPatchScope,
    TruthSnapshotIdentity,
};
use crate::truth_identity_fixtures::truth_branch;

pub(super) fn registration() -> BridgeMappingRegistration {
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

pub(super) fn surface_widening_registration() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("profile-surface-widening"),
        TruthPatchScope::for_target(
            MappingSelector::exact("user"),
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            crate::facade::TruthPatchTargetSelector::any(),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
        SignalInvalidationScope::admit_bridge_owned("signal.profile.widening"),
        CoarseRoutingMode::Direct,
    )
}

pub(super) fn committed_patch(
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    field_key: worth_foundational::facade::FieldKey,
) -> BridgeCommittedPatchEnvelope {
    committed_patch_on_branch(
        truth_branch("main"),
        commit_identity,
        patch_identity,
        snapshot_identity,
        field_key,
    )
}

pub(super) fn committed_patch_on_branch(
    branch_identity: TruthBranchIdentity,
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    field_key: worth_foundational::facade::FieldKey,
) -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
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
                worth_foundational::facade::CanonicalFieldPath::single(field_key),
            ),
        )],
    )
    .expect("harness committed patch envelope should construct")
}

pub(super) fn committed_region_patch(
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
) -> BridgeCommittedPatchEnvelope {
    committed_patch_items(
        commit_identity,
        patch_identity,
        snapshot_identity,
        vec![crate::facade::BridgeCommittedPatchItem::with_target(
            "user",
            crate::facade::BridgeCommittedPatchTarget::entity_region(
                worth_foundational::facade::AspectLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid bridge patch aspect key"),
                ),
            ),
        )],
    )
}

pub(super) fn committed_partition_patch(
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
) -> BridgeCommittedPatchEnvelope {
    committed_patch_items(
        commit_identity,
        patch_identity,
        snapshot_identity,
        vec![crate::facade::BridgeCommittedPatchItem::with_target(
            "user",
            crate::facade::BridgeCommittedPatchTarget::entity_partition(
                worth_foundational::facade::AspectLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid bridge patch aspect key"),
                ),
            ),
        )],
    )
}

pub(super) fn committed_patch_items(
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    items: Vec<crate::facade::BridgeCommittedPatchItem>,
) -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            BridgeProducerMetadata::bridge_harness_fixture(),
            commit_identity,
            patch_identity,
            snapshot_identity,
            truth_branch("main"),
        ),
        items,
    )
    .expect("harness committed patch envelope should construct")
}

pub(super) fn build_runtime_with_snapshot_source(
    patch_source: impl crate::facade::CommittedPatchSource,
    snapshot_source: impl crate::facade::SnapshotReadSource,
    sink: impl InvalidationSink,
    mapping: BridgeMappingRegistration,
    aspect: Option<BridgeAspectRegistration>,
) -> crate::facade::RuntimeBridge {
    let mut builder = RuntimeBridgeBuilder::new()
        .with_committed_patch_source(patch_source)
        .with_snapshot_read_source(snapshot_source)
        .with_signal_sink(sink)
        .register_mapping(mapping);
    if let Some(aspect) = aspect {
        builder = builder.register_aspect_mapping(aspect);
    }
    builder.build().expect("snapshot protocol fixture builds")
}

pub(super) fn build_runtime<S>(
    source: impl crate::adapter::RelationalBridgeSource + crate::adapter::TruthBranchHeadSource + Clone,
    sink: S,
    mappings: Vec<BridgeMappingRegistration>,
) -> crate::facade::RuntimeBridge
where
    S: InvalidationSink,
{
    build_runtime_with_aspects(source, sink, mappings, vec![])
}

pub(super) fn build_runtime_with_aspects<S>(
    source: impl crate::adapter::RelationalBridgeSource + crate::adapter::TruthBranchHeadSource + Clone,
    sink: S,
    mappings: Vec<BridgeMappingRegistration>,
    aspect_mappings: Vec<BridgeAspectRegistration>,
) -> crate::facade::RuntimeBridge
where
    S: InvalidationSink,
{
    let builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source.clone())
        .with_truth_branch_head_source(source)
        .with_signal_sink(sink);
    let mut mappings = mappings.into_iter();
    let first_mapping = mappings
        .next()
        .expect("bridge harness tests require at least one mapping");
    let mut builder = builder.register_mapping(first_mapping);
    for mapping in mappings {
        builder = builder.register_mapping(mapping);
    }
    for aspect_mapping in aspect_mappings {
        builder = builder.register_aspect_mapping(aspect_mapping);
    }
    builder
        .build()
        .expect("bridge runtime should build for harness tests")
}

#[derive(Debug, Clone, Default)]
pub(super) struct RejectingSignalSink;

impl InvalidationSink for RejectingSignalSink {
    fn deliver_invalidation(
        &self,
        _delivery: crate::facade::BridgeSignalInvalidationDelivery,
        _lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::facade::BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Err(SignalBridgeSinkError::new(
            crate::adapter::SignalBridgeSinkErrorKind::ExternalSinkFailure,
            "forced sink rejection",
        ))
    }
}

pub(super) fn field_aspect_registration() -> BridgeAspectRegistration {
    BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::admit_bridge_owned("profile-name-field"),
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
        TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    )
}
