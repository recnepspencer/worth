use worth_foundational::facade::{
    AspectKey, AspectLocator, CanonicalFieldPath, FieldKey, LocatorAuthority, ScalarAspectType,
};

use super::{
    canonical_route_entry_basis, canonical_route_entry_order, canonical_snapshot_request_order,
    digest_string, lowering_summary_digest_basis, planning_provenance_digest_basis,
    route_digest_basis, subscription_slice_digest_basis, SnapshotReadRequestSetView,
};
use crate::input::envelope::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem,
    BridgeCommittedPatchTarget, BridgeProducerMetadata,
};
use crate::mapping::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeMappingId,
    BridgeMappingRegistration, CoarseRoutingMode, FrozenAspectMappingRegistry,
    FrozenMappingRegistry, MappingSelector, SignalInvalidationScope, SliceWideningPolicy,
    SubscriptionSliceKind, TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
};
use crate::routing::context::BridgeMappingContext;
use crate::routing::eligibility::validate_route_request;
use crate::routing::lowering::BridgeSubscriptionSlice;
use crate::routing::matching::FineGrainedMatchStatus;
use crate::routing::planning::BridgeRouteIdentity;
use crate::snapshot::{SnapshotReadContract, SnapshotReadRequest};

#[test]
fn route_digest_inputs_use_digest_shaped_route_entry_identity_not_raw_target_text() {
    let target = BridgeCommittedPatchTarget::entity_field_path(
        aspect_locator("profile"),
        CanonicalFieldPath::single(field_key("name")),
    );
    let target_basis = target.canonical_basis();
    let envelope = envelope_for_target(target);
    let mapping_context = BridgeMappingContext::empty();
    let eligible =
        validate_route_request(envelope.clone(), &mapping_registry(), &aspect_registry())
            .expect("field target should be route eligible");

    let route_basis = route_digest_basis(&envelope, &mapping_context, eligible.entries());
    assert_contains_route_entry_digest_only("route basis", &route_basis, &target_basis);
    assert_route_entry_basis_uses_typed_surface_and_registration_proofs(
        eligible.entries()[0]
            .normalized_surface()
            .surface_identity()
            .as_str(),
        eligible.entries()[0]
            .registration()
            .registration_identity()
            .as_str(),
        &canonical_route_entry_basis(&eligible.entries()[0]),
        &target_basis,
    );

    let route_identity =
        BridgeRouteIdentity::admit_bridge_owned(digest_string("route", &route_basis));
    let provenance_basis = planning_provenance_digest_basis(
        &route_identity,
        &envelope,
        &mapping_context,
        eligible.entries(),
        &SnapshotReadRequestSetView::new(&[]),
    );
    assert_contains_route_entry_digest_only(
        "planning provenance",
        &provenance_basis,
        &target_basis,
    );
}

#[test]
fn subscription_slice_digest_inputs_consume_slice_canonical_basis_not_target_basis() {
    let slice = BridgeSubscriptionSlice::from_continuity_parts(
        "entity-1",
        aspect_locator("profile"),
        Some(worth_foundational::facade::AspectFieldLocator::from_aspect(
            aspect_locator("profile"),
            CanonicalFieldPath::single(field_key("name")),
        )),
        worth_foundational::facade::AspectMask::new([CanonicalFieldPath::single(field_key(
            "name",
        ))]),
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        FineGrainedMatchStatus::Matched,
    );

    let slice_basis = subscription_slice_digest_basis("snapshot-a", std::slice::from_ref(&slice));
    assert!(
        slice_basis.contains(slice.canonical_basis()),
        "subscription slice digest basis must consume the retained slice canonical proof: {slice_basis}"
    );
    assert!(
        !slice_basis.contains(slice.native_target_basis()),
        "subscription slice digest basis must not reopen committed target basis: {slice_basis}"
    );

    let lowering_summary_basis = lowering_summary_digest_basis(
        &BridgeRouteIdentity::admit_bridge_owned(digest_string("route", "route-basis")),
        &[],
        std::slice::from_ref(&slice),
        0,
    );
    assert!(
        lowering_summary_basis.contains(slice.canonical_basis()),
        "lowering summary must consume the retained slice canonical proof: {lowering_summary_basis}"
    );
    assert!(
        !lowering_summary_basis.contains(slice.native_target_basis()),
        "lowering summary must not reopen committed target basis: {lowering_summary_basis}"
    );
}

#[test]
fn route_entry_order_consumes_frozen_registration_identity_after_surface_proof() {
    let target = BridgeCommittedPatchTarget::entity_field_path(
        aspect_locator("profile"),
        CanonicalFieldPath::single(field_key("name")),
    );
    let envelope = envelope_for_target(target);
    let eligible = validate_route_request(
        envelope,
        &dual_mapping_registry_for_same_surface(),
        &aspect_registry(),
    )
    .expect("dual registrations should be route eligible");
    assert_eq!(eligible.entries().len(), 2);

    let registration_identity_order = eligible.entries()[0]
        .registration()
        .registration_identity()
        .cmp(eligible.entries()[1].registration().registration_identity());
    assert_eq!(
        canonical_route_entry_order(&eligible.entries()[0], &eligible.entries()[1]),
        registration_identity_order,
        "route entry ordering must consume typed frozen registration identity after the normalized surface proof"
    );
}

#[test]
fn snapshot_read_order_consumes_target_identity_not_native_target_basis() {
    let left = SnapshotReadRequest::for_native_subscription_slice(
        "entity-1",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        aspect_locator("profile"),
        Some(worth_foundational::facade::AspectFieldLocator::from_aspect(
            aspect_locator("profile"),
            CanonicalFieldPath::single(field_key("name")),
        )),
        worth_foundational::facade::AspectMask::new([CanonicalFieldPath::single(field_key(
            "name",
        ))]),
        SubscriptionSliceKind::SignalField,
    );
    let right = SnapshotReadRequest::for_native_subscription_slice(
        "entity-1",
        SnapshotReadContract::scalar(aspect_key("profile"), ScalarAspectType::String),
        aspect_locator("profile"),
        Some(worth_foundational::facade::AspectFieldLocator::from_aspect(
            aspect_locator("profile"),
            CanonicalFieldPath::single(field_key("email")),
        )),
        worth_foundational::facade::AspectMask::new([CanonicalFieldPath::single(field_key(
            "email",
        ))]),
        SubscriptionSliceKind::SignalField,
    );

    let target_identity_order = left
        .target()
        .target_identity()
        .cmp(right.target().target_identity());
    assert_eq!(
        canonical_snapshot_request_order(&left, &right),
        target_identity_order,
        "snapshot read ordering must consume typed target identity after entity/aspect/slice-kind equality"
    );
    assert_ne!(left.native_target_basis(), right.native_target_basis());
}

fn assert_route_entry_basis_uses_typed_surface_and_registration_proofs(
    surface_identity: &str,
    registration_identity: &str,
    route_entry_basis: &str,
    target_basis: &str,
) {
    assert!(
        route_entry_basis.contains(surface_identity),
        "route entry basis must retain typed truth-delta surface proof: {route_entry_basis}"
    );
    assert!(
        route_entry_basis.contains(registration_identity),
        "route entry basis must retain typed frozen registration proof: {route_entry_basis}"
    );
    assert!(
        !route_entry_basis.contains(target_basis),
        "route entry basis must not embed native target basis: {route_entry_basis}"
    );
    assert!(
        !route_entry_basis.contains("entity-1"),
        "route entry basis must not embed entity text: {route_entry_basis}"
    );
    assert!(
        !route_entry_basis.contains("profile"),
        "route entry basis must not embed aspect text: {route_entry_basis}"
    );
    assert!(
        !route_entry_basis.contains("native-target-route"),
        "route entry basis must not embed mapping id text: {route_entry_basis}"
    );
    assert!(
        !route_entry_basis.contains("signal.native-target"),
        "route entry basis must not embed signal scope text: {route_entry_basis}"
    );
}

fn assert_contains_route_entry_digest_only(label: &str, basis: &str, target_basis: &str) {
    assert!(
        basis.contains("entry=route-entry:sha256:"),
        "{label} must contain a digest-shaped route entry: {basis}"
    );
    assert!(
        !basis.contains(target_basis),
        "{label} must not embed the native target basis: {basis}"
    );
    assert!(
        !basis.contains("entity-1"),
        "{label} must not embed entity text: {basis}"
    );
    assert!(
        !basis.contains("profile"),
        "{label} must not embed aspect text: {basis}"
    );
    assert!(
        !basis.contains("signal.native-target"),
        "{label} must not embed signal scope text: {basis}"
    );
    assert!(
        !basis.contains("native-target-route"),
        "{label} must not embed mapping id text: {basis}"
    );
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

fn mapping_registry() -> FrozenMappingRegistry {
    FrozenMappingRegistry::freeze(vec![BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("native-target-route"),
        TruthPatchScope::for_target(
            MappingSelector::exact("entity-1"),
            aspect_key("profile"),
            TruthPatchTargetSelector::entity_field(field_key("name")),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            aspect_key("profile"),
            ScalarAspectType::String,
        ),
        SignalInvalidationScope::admit_bridge_owned("signal.native-target"),
        CoarseRoutingMode::Direct,
    )])
    .expect("mapping registry should freeze")
}

fn dual_mapping_registry_for_same_surface() -> FrozenMappingRegistry {
    FrozenMappingRegistry::freeze(vec![
        mapping_registration("native-target-route-b", "signal.native-target-b"),
        mapping_registration("native-target-route-a", "signal.native-target-a"),
    ])
    .expect("dual mapping registry should freeze")
}

fn mapping_registration(mapping_id: &str, signal_scope: &str) -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned(mapping_id),
        TruthPatchScope::for_target(
            MappingSelector::exact("entity-1"),
            aspect_key("profile"),
            TruthPatchTargetSelector::entity_field(field_key("name")),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            aspect_key("profile"),
            ScalarAspectType::String,
        ),
        SignalInvalidationScope::admit_bridge_owned(signal_scope),
        CoarseRoutingMode::Direct,
    )
}

fn aspect_registry() -> FrozenAspectMappingRegistry {
    FrozenAspectMappingRegistry::freeze(vec![BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::admit_bridge_owned("native-target-aspect"),
        TruthPatchScope::for_target(
            MappingSelector::exact("entity-1"),
            aspect_key("profile"),
            TruthPatchTargetSelector::entity_field(field_key("name")),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            aspect_key("profile"),
            ScalarAspectType::String,
        ),
        TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    )])
    .expect("aspect registry should freeze")
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid route canonicalization aspect key")
}

fn aspect_locator(value: &str) -> AspectLocator {
    AspectLocator::new(LocatorAuthority::Authoritative, aspect_key(value))
}

fn field_key(value: &str) -> FieldKey {
    FieldKey::new(value.to_owned()).expect("valid route canonicalization field key")
}
