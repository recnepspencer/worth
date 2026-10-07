use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::facade::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeCommittedPatchEnvelope,
    BridgeMappingId, BridgeMappingRegistration, BridgeProducerMetadata, CoarseRoutingMode,
    InvalidationSink, MappingSelector, RuntimeBridgeBuilder, SignalBridgeSinkError,
    SignalInvalidationScope, SliceWideningPolicy, SnapshotReadRecord, SnapshotReadRequest,
    SnapshotReaderPool, SubscriptionSliceKind, TruthBranchIdentity, TruthCommitIdentity,
    TruthDeltaSurfaceKind, TruthPatchIdentity, TruthPatchScope, TruthSnapshotIdentity,
};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, SnapshotFixture};
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

pub(super) fn snapshot(snapshot_identity: TruthSnapshotIdentity, value: &str) -> SnapshotFixture {
    SnapshotFixture::new(
        snapshot_identity,
        vec![coarse_snapshot_record(
            "user",
            "profile",
            worth_foundational::facade::AspectValue::String((value).into()),
        )],
    )
}

pub(super) fn field_slice_snapshot(
    snapshot_identity: TruthSnapshotIdentity,
    value: &str,
) -> SnapshotFixture {
    let field_slice_request = field_slice_snapshot_read_request("user", "profile", "name");
    SnapshotFixture::new(
        snapshot_identity,
        vec![SnapshotReadRecord::for_request(
            &field_slice_request,
            worth_foundational::facade::AspectValue::String((value).into()),
        )],
    )
}

pub(super) fn coarse_snapshot_record(
    entity: &str,
    aspect: &str,
    value: worth_foundational::facade::AspectValue,
) -> SnapshotReadRecord {
    SnapshotReadRecord::for_request(
        &SnapshotReadRequest::for_coarse(
            entity,
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new(aspect)
                    .expect("valid coarse snapshot aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
        ),
        value,
    )
}

fn field_slice_snapshot_read_request(
    entity: &str,
    aspect: &str,
    field: &str,
) -> SnapshotReadRequest {
    let aspect_key =
        worth_foundational::facade::AspectKey::new(aspect).expect("valid field-slice aspect key");
    let aspect_locator = worth_foundational::facade::AspectLocator::new(
        worth_foundational::facade::LocatorAuthority::Authoritative,
        aspect_key.clone(),
    );
    let field_path = worth_foundational::facade::CanonicalFieldPath::single(
        worth_foundational::facade::FieldKey::new(field.to_owned()).expect("valid field key"),
    );
    let field_locator = worth_foundational::facade::AspectFieldLocator::from_aspect(
        aspect_locator.clone(),
        field_path.clone(),
    );
    SnapshotReadRequest::for_native_subscription_slice(
        entity,
        crate::snapshot::SnapshotReadContract::scalar(
            aspect_key,
            worth_foundational::facade::ScalarAspectType::String,
        ),
        aspect_locator,
        Some(field_locator),
        worth_foundational::facade::AspectMask::new([field_path]),
        SubscriptionSliceKind::SignalField,
    )
}

pub(super) fn build_runtime<S>(
    source: InMemoryRelationalBridgeSource,
    sink: S,
    mappings: Vec<BridgeMappingRegistration>,
) -> crate::facade::RuntimeBridge
where
    S: InvalidationSink,
{
    build_runtime_with_aspects(source, sink, mappings, vec![])
}

pub(super) fn build_runtime_with_aspects<S>(
    source: InMemoryRelationalBridgeSource,
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
        Err(SignalBridgeSinkError::new("forced sink rejection"))
    }
}

#[derive(Clone)]
pub(super) struct CountingSnapshotReaderPool {
    source: InMemoryRelationalBridgeSource,
    acquire_count: Arc<AtomicUsize>,
    release_count: Arc<AtomicUsize>,
}

impl CountingSnapshotReaderPool {
    pub(super) fn new(source: InMemoryRelationalBridgeSource) -> Self {
        Self {
            source,
            acquire_count: Arc::new(AtomicUsize::new(0)),
            release_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(super) fn acquire_count(&self) -> usize {
        self.acquire_count.load(Ordering::SeqCst)
    }

    pub(super) fn release_count(&self) -> usize {
        self.release_count.load(Ordering::SeqCst)
    }
}

impl SnapshotReaderPool for CountingSnapshotReaderPool {
    fn acquire(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<
        Box<dyn crate::facade::TruthSnapshotReader>,
        crate::facade::RelationalBridgeSourceError,
    > {
        self.acquire_count.fetch_add(1, Ordering::SeqCst);
        crate::facade::SnapshotReadSource::open_snapshot(&self.source, identity)
    }

    fn release(&self, _reader: Box<dyn crate::facade::TruthSnapshotReader>) {
        self.release_count.fetch_add(1, Ordering::SeqCst);
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

pub(super) fn field_aspect_registration_with_kind(
    registration_id: &str,
    surface_kind: TruthDeltaSurfaceKind,
    slice_kind: SubscriptionSliceKind,
) -> BridgeAspectRegistration {
    BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::admit_bridge_owned(registration_id),
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
        surface_kind,
        slice_kind,
        SliceWideningPolicy::Disallow,
    )
}

pub(super) fn merge_declaration(
    declaration_identity: crate::facade::MergeHistoryDeclarationIdentity,
    class: crate::facade::BridgeMergeConsumptionClass,
    parents: impl IntoIterator<Item = crate::facade::TruthCommitIdentity>,
) -> crate::facade::MergeHistoryDeclaration {
    let authority_artifact_identity = format!("merge-artifact:{}", declaration_identity.as_str());
    crate::facade::MergeHistoryDeclaration::new(
        declaration_identity,
        class,
        crate::facade::BridgeMergeOntologyMappingSurface::direct_phase_m9_0("rel-merge-v1"),
        crate::facade::BridgeMergeAuthorityBasis::new(
            crate::facade::BridgeMergeAuthorityBasisKind::OrderedMergeCommit,
            authority_artifact_identity,
            "rel-merge-v1",
            "schema-policy-v1",
            crate::facade::BridgeMergeParentOrderProof::new(parents.into_iter().collect()),
        ),
    )
}
