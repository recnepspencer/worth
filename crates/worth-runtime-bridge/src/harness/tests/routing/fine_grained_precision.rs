use crate::facade::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeMappingId,
    BridgeMappingRegistration, BridgeRouteRequest, CoarseRoutingMode, MappingSelector,
    SignalInvalidationScope, SliceWideningPolicy, SubscriptionSliceKind, TruthDeltaSurfaceKind,
    TruthPatchScope,
};

use super::super::support::{
    build_runtime_with_aspects, committed_patch, committed_region_patch, field_aspect_registration,
    field_slice_snapshot, registration,
};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};

#[test]
fn field_surface_invalidates_only_registered_field_slice() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    let name_field = worth_foundational::facade::FieldKey::new("name".to_owned())
        .expect("valid harness field key");
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        name_field.clone(),
    ));
    source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );

    let route = runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            execution,
        )
        .expect("field-scoped route should plan");

    assert_eq!(route.subscription_slices().len(), 1);
    assert_eq!(
        route.subscription_slices().slices()[0].slice_kind(),
        &SubscriptionSliceKind::SignalField
    );
    assert_eq!(
        route.subscription_slices().slices()[0].native_target_basis(),
        expected_field_target_basis(&name_field),
    );
    assert_eq!(route.counters().truth_delta_surface_count(), 1);
    assert_eq!(route.counters().normalized_truth_delta_surface_count(), 1);
    assert_eq!(route.counters().planned_slice_match_count(), 1);
    assert_eq!(route.counters().slice_widening_count(), 0);
    assert_eq!(route.counters().slice_suppression_count(), 0);
    assert_eq!(route.counters().mapping_widening_count(), 0);
    assert_eq!(route.lowering_summary().subscription_slice_count(), 1);
    assert_eq!(route.routing_summary().invalidation_target_count(), 1);
    assert_invalidation_target_retains_native_basis(
        &route,
        expected_field_target_basis(&name_field).as_str(),
    );
}

fn region_mapping_registration() -> BridgeMappingRegistration {
    region_mapping_registration_with_signal_scope(SignalInvalidationScope::admit_bridge_owned(
        "signal.profile.region",
    ))
}

fn region_mapping_registration_with_signal_scope(
    signal_scope: SignalInvalidationScope,
) -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("profile-region"),
        TruthPatchScope::for_target(
            MappingSelector::exact("user"),
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            crate::facade::TruthPatchTargetSelector::region(),
        ),
        crate::snapshot::SnapshotReadContract::scalar(
            worth_foundational::facade::AspectKey::new("profile").expect("valid native aspect key"),
            worth_foundational::facade::ScalarAspectType::String,
        ),
        signal_scope,
        CoarseRoutingMode::Direct,
    )
}

#[test]
fn region_surface_invalidates_only_registered_region_slice() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_region_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let runtime = build_runtime_with_aspects(
        source,
        RecordingSignalBridgeSink::default(),
        vec![region_mapping_registration()],
        vec![BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::admit_bridge_owned("profile-name-region"),
            TruthPatchScope::for_target(
                MappingSelector::exact("user"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                crate::facade::TruthPatchTargetSelector::region(),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            TruthDeltaSurfaceKind::EntityRegion,
            SubscriptionSliceKind::SignalRegion,
            SliceWideningPolicy::Disallow,
        )],
    );

    let route = runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            execution,
        )
        .expect("region-scoped route should plan");

    assert_eq!(route.subscription_slices().len(), 1);
    assert_eq!(
        route.subscription_slices().slices()[0].slice_kind(),
        &SubscriptionSliceKind::SignalRegion
    );
    assert_eq!(
        route.subscription_slices().slices()[0].native_target_basis(),
        expected_region_target_basis(),
    );
    assert_eq!(route.counters().truth_delta_surface_count(), 1);
    assert_eq!(route.counters().normalized_truth_delta_surface_count(), 1);
    assert_eq!(route.counters().planned_slice_match_count(), 1);
    assert_eq!(route.counters().slice_widening_count(), 0);
    assert_eq!(route.counters().slice_suppression_count(), 0);
    assert_eq!(route.counters().mapping_widening_count(), 0);
    assert_eq!(route.lowering_summary().subscription_slice_count(), 1);
    assert_eq!(route.routing_summary().invalidation_target_count(), 1);
    assert_invalidation_target_retains_native_basis(&route, expected_region_target_basis());
}

#[test]
fn invalidation_target_identity_changes_with_surface_proof_even_for_shared_signal_scope() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let field_source = InMemoryRelationalBridgeSource::default();
    let name_field = worth_foundational::facade::FieldKey::new("name".to_owned())
        .expect("valid harness field key");
    field_source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-field"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-field"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-field"),
        name_field,
    ));
    field_source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-field"),
        "alice",
    ));
    let field_runtime = build_runtime_with_aspects(
        field_source,
        RecordingSignalBridgeSink::default(),
        vec![registration()],
        vec![field_aspect_registration()],
    );

    let region_source = InMemoryRelationalBridgeSource::default();
    region_source.insert_committed_patch(committed_region_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-region"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-region"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-region"),
    ));
    let region_runtime = build_runtime_with_aspects(
        region_source,
        RecordingSignalBridgeSink::default(),
        vec![region_mapping_registration_with_signal_scope(
            SignalInvalidationScope::admit_bridge_owned("signal.profile"),
        )],
        vec![BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::admit_bridge_owned("profile-name-region-shared-signal"),
            TruthPatchScope::for_target(
                MappingSelector::exact("user"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                crate::facade::TruthPatchTargetSelector::region(),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            TruthDeltaSurfaceKind::EntityRegion,
            SubscriptionSliceKind::SignalRegion,
            SliceWideningPolicy::Disallow,
        )],
    );

    let field_route = field_runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-field",
            )),
            execution,
        )
        .expect("field route should plan");
    let region_route = region_runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-region",
            )),
            execution,
        )
        .expect("region route should plan");

    let field_target = &field_route.invalidation_targets().targets()[0];
    let region_target = &region_route.invalidation_targets().targets()[0];
    assert_eq!(field_target.signal_scope(), region_target.signal_scope());
    assert_ne!(
        field_target.native_target_basis(),
        region_target.native_target_basis()
    );
    assert_ne!(
        field_target.surface_identity(),
        region_target.surface_identity()
    );
    assert_ne!(
        field_target.target_identity(),
        region_target.target_identity()
    );
    assert_ne!(
        field_route.validated_lowering_summary().digest(),
        region_route.validated_lowering_summary().digest()
    );
}

fn expected_field_target_basis(field: &worth_foundational::facade::FieldKey) -> String {
    let field = field.as_str();
    format!(
        "committed-patch-target|locator=version=bridge.committed-patch-target.v1;domain=locator;entries=[locus=named:aspect_field.aspect_key,kind=locator,value=exact-text:profile;locus=named:aspect_field.authority,kind=locator,value=exact-text:authoritative;locus=named:aspect_field.field_path,kind=locator,value=exact-text:{field};locus=named:aspect_field.kind,kind=locator,value=exact-text:aspect]|mutation-mask=version=bridge.committed-patch-target.v1;domain=aspect-mask;entries=[locus=named:profile.mutation.field.{field},kind=mask,value=exact-text:{field}]|projection-mask=version=bridge.committed-patch-target.v1;domain=aspect-mask;entries=[locus=named:profile.projection.field.{field},kind=mask,value=exact-text:{field}]|kind=entity-field"
    )
}

fn expected_region_target_basis() -> &'static str {
    "committed-patch-target|locator=version=bridge.committed-patch-target.v1;domain=locator;entries=[locus=named:aspect.aspect_key,kind=locator,value=exact-text:profile;locus=named:aspect.authority,kind=locator,value=exact-text:authoritative;locus=named:aspect.kind,kind=locator,value=exact-text:aspect]|mutation-mask=version=bridge.committed-patch-target.v1;domain=aspect-mask;entries=[locus=named:profile.mutation.whole,kind=mask,value=exact-text:whole]|projection-mask=version=bridge.committed-patch-target.v1;domain=aspect-mask;entries=[locus=named:profile.projection.whole,kind=mask,value=exact-text:whole]|kind=entity-region"
}

fn assert_invalidation_target_retains_native_basis(
    route: &crate::facade::BridgePlannedRoute,
    expected_native_target_basis: &str,
) {
    let target = &route.invalidation_targets().targets()[0];
    assert!(target
        .target_identity()
        .as_str()
        .starts_with("invalidation-target:sha256:"));
    assert_eq!(target.native_target_basis(), expected_native_target_basis);
    assert_eq!(
        target.surface_identity().as_str(),
        route.route_record_entries()[0].truth_surface_identity()
    );
    assert!(!target
        .target_identity()
        .as_str()
        .contains(target.signal_scope()));
    assert!(!target
        .target_identity()
        .as_str()
        .contains("committed-patch-target"));
}
