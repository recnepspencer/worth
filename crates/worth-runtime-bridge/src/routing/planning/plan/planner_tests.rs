use worth_foundational::facade::{AspectKey, AspectLocator, LocatorAuthority, ScalarAspectType};

use crate::adapter::{
    CommittedPatchSource, InvalidationSink, RelationalBridgeSourceError,
    RelationalCommittedPatchRequest, SignalBridgeSinkError, SnapshotReadSource,
};
use crate::builder::RuntimeBridgeBuilder;
use crate::delivery::BridgeDeliveryReceipt;
use crate::facade::RuntimeBridge;
use crate::input::envelope::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem,
    BridgeCommittedPatchTarget, BridgeProducerMetadata,
};
use crate::mapping::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeMappingId,
    BridgeMappingRegistration, CoarseRoutingMode, MappingSelector, SignalInvalidationScope,
    SliceWideningPolicy, SubscriptionSliceKind, TruthDeltaSurfaceKind, TruthPatchScope,
    TruthPatchTargetSelector,
};
use crate::routing::BridgeSignalInvalidationDelivery;
use crate::snapshot::{SnapshotReadContract, TruthSnapshotIdentity, TruthSnapshotReader};

#[test]
fn final_route_artifacts_preserve_native_whole_target_basis_only() {
    let cases = [
        (
            BridgeCommittedPatchTarget::entity_relation_endpoint(aspect_locator("profile")),
            TruthPatchTargetSelector::relation_endpoint(),
            TruthDeltaSurfaceKind::EntityRelationEndpoint,
            SubscriptionSliceKind::SignalLens,
            "entity-relation-endpoint",
        ),
        (
            BridgeCommittedPatchTarget::entity_region(aspect_locator("profile")),
            TruthPatchTargetSelector::region(),
            TruthDeltaSurfaceKind::EntityRegion,
            SubscriptionSliceKind::SignalRegion,
            "entity-region",
        ),
        (
            BridgeCommittedPatchTarget::entity_partition(aspect_locator("profile")),
            TruthPatchTargetSelector::partition(),
            TruthDeltaSurfaceKind::EntityPartition,
            SubscriptionSliceKind::SignalPartition,
            "entity-partition",
        ),
        (
            BridgeCommittedPatchTarget::entity_facet(aspect_locator("profile")),
            TruthPatchTargetSelector::facet(),
            TruthDeltaSurfaceKind::EntityFacet,
            SubscriptionSliceKind::SignalFacet,
            "entity-facet",
        ),
    ];

    for (target, target_selector, surface_kind, slice_kind, kind_label) in cases {
        let planned_route = runtime_for_target(target_selector, surface_kind, slice_kind.clone())
            .plan_envelope(envelope_for_target(target))
            .expect("native whole-aspect target should produce a planned route");

        let [route_record] = planned_route.route_record_entries() else {
            panic!("expected exactly one final route record entry");
        };

        assert_eq!(route_record.truth_surface_kind(), surface_kind);
        assert_eq!(
            route_record.fine_grained_match_status(),
            crate::routing::matching::FineGrainedMatchStatus::Matched
        );
        assert_native_target_basis(&route_record.target_canonical_basis(), kind_label);
        assert_native_target_basis(&route_record.source_target_canonical_basis(), kind_label);
        assert_native_target_carrier(route_record.target(), surface_kind);
        assert_native_target_carrier(route_record.source_target(), surface_kind);
        assert_native_route_identity(route_record.truth_surface_identity());

        let [read_request] = planned_route.read_packet().reads() else {
            panic!("native route should produce one subscription-slice read request");
        };
        assert_native_snapshot_target_basis(read_request.native_target_basis());
        assert_eq!(read_request.slice_kind(), Some(&slice_kind));
    }
}

fn assert_native_target_basis(value: &str, kind_label: &str) {
    assert!(value.contains("committed-patch-target|locator="));
    assert!(value.contains("mutation-mask="));
    assert!(value.contains("projection-mask="));
    assert!(value.contains(&format!("|kind={kind_label}")));
    assert_no_old_target_residue(value);
}

fn assert_native_target_carrier(
    target: &BridgeCommittedPatchTarget,
    expected_kind: TruthDeltaSurfaceKind,
) {
    assert_eq!(target.surface_kind(), expected_kind);
    match expected_kind {
        TruthDeltaSurfaceKind::EntityField => {
            assert!(target.field_locator().is_some());
            assert!(!target.mutation_mask().is_whole_aspect());
            assert!(!target.projection_mask().is_whole_aspect());
        }
        TruthDeltaSurfaceKind::AuthoritativeAspect
        | TruthDeltaSurfaceKind::EntityRelationEndpoint
        | TruthDeltaSurfaceKind::EntityRegion
        | TruthDeltaSurfaceKind::EntityPartition
        | TruthDeltaSurfaceKind::EntityFacet
        | TruthDeltaSurfaceKind::LifecycleTransition => {
            assert_eq!(target.field_locator(), None);
            assert!(target.mutation_mask().is_whole_aspect());
            assert!(target.projection_mask().is_whole_aspect());
        }
    }
}

fn assert_native_route_identity(value: &str) {
    assert!(
        value.starts_with("truth-delta-surface:sha256:"),
        "truth surface identity must be digest-shaped: {value}"
    );
    assert!(
        !value.contains("committed-patch-target|locator="),
        "truth surface identity must not embed native target basis: {value}"
    );
    assert_no_old_target_residue(value);
}

fn assert_native_snapshot_target_basis(value: &str) {
    assert!(value.contains("snapshot-read-target|locator="));
    assert!(value.contains("projection-mask="));
    assert_no_old_target_residue(value);
}

fn assert_no_old_target_residue(value: &str) {
    for forbidden_marker in old_target_authority_markers() {
        assert!(!value.contains(&forbidden_marker));
    }
}

fn old_target_authority_markers() -> [String; 3] {
    [
        format!("{}:", "field"),
        format!("{}_{}", "surface", "label"),
        format!("{}_{}", "aspect", "label"),
    ]
}

fn runtime_for_target(
    target_selector: TruthPatchTargetSelector,
    surface_kind: TruthDeltaSurfaceKind,
    slice_kind: SubscriptionSliceKind,
) -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_relational_source(PlanningTestSource)
        .with_signal_sink(PlanningTestSink)
        .register_mapping(mapping_registration(target_selector.clone()))
        .register_aspect_mapping(aspect_registration(
            target_selector,
            surface_kind,
            slice_kind,
        ))
        .build()
        .expect("runtime should build for route artifact proof")
}

fn mapping_registration(target_selector: TruthPatchTargetSelector) -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("native-target-route"),
        TruthPatchScope::for_target(
            MappingSelector::exact("entity-1"),
            aspect_key("profile"),
            target_selector,
        ),
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        SignalInvalidationScope::admit_bridge_owned("signal.native-target"),
        CoarseRoutingMode::Direct,
    )
}

fn aspect_registration(
    target_selector: TruthPatchTargetSelector,
    surface_kind: TruthDeltaSurfaceKind,
    slice_kind: SubscriptionSliceKind,
) -> BridgeAspectRegistration {
    BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::admit_bridge_owned("native-target-aspect"),
        TruthPatchScope::for_target(
            MappingSelector::exact("entity-1"),
            aspect_key("profile"),
            target_selector,
        ),
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        surface_kind,
        slice_kind,
        SliceWideningPolicy::Disallow,
    )
}

fn envelope_for_target(target: BridgeCommittedPatchTarget) -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new_with_metadata(
            BridgeProducerMetadata::bridge_harness_fixture(),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
        ),
        vec![BridgeCommittedPatchItem::with_target("entity-1", target)],
    )
    .expect("native target envelope should construct")
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid patch target aspect key")
}

fn aspect_locator(value: &str) -> AspectLocator {
    AspectLocator::new(LocatorAuthority::Authoritative, aspect_key(value))
}

struct PlanningTestSource;

impl CommittedPatchSource for PlanningTestSource {
    fn load_committed_patch(
        &self,
        _request: RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        unreachable!("planning artifact test plans explicit envelopes")
    }
}

impl SnapshotReadSource for PlanningTestSource {
    fn open_snapshot(
        &self,
        _identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        unreachable!("planning artifact test does not materialize snapshots")
    }
}

struct PlanningTestSink;

impl InvalidationSink for PlanningTestSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}
