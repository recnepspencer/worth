use super::support::{
    build_runtime, build_runtime_with_aspects, committed_patch, field_aspect_registration,
    field_slice_snapshot, registration, snapshot, surface_widening_registration,
};
use crate::facade::{
    BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeContinuityAuthorityBasis,
    BridgeHistoricalLineageAuthority, BridgeHistoricalLineageRequest,
    BridgeHistoricalResolvedLineageIdentity, BridgeHistoricalResolvedRecordIdentity,
    BridgeLineageContext, BridgeLineageSourceError, BridgePreparationMode,
    BridgeRouteRecordEntityIdentity as RouteEntityIdentity, BridgeRouteRequest,
    ContinuityLineageSource, FineGrainedMatchStatus, SliceWideningPolicy, SubscriptionSliceKind,
    TruthDeltaSurfaceKind,
};
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};
#[derive(Debug, Clone, Default)]
struct ExplanationContinuityLineageSource;

impl ContinuityLineageSource for ExplanationContinuityLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                "lineage:explanation-successor",
            )],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "entity:0:4:2",
            )],
            vec![7],
        )
    }
}

#[test]
fn bridge_route_explanation_reconstructs_patch_to_invalidation_mapping() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    let avatar_field = worth_foundational::facade::FieldKey::new("avatar".to_owned())
        .expect("valid harness field key");
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        avatar_field.clone(),
    ));
    source.insert_snapshot(snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = build_runtime(
        source,
        RecordingSignalBridgeSink::default(),
        vec![surface_widening_registration()],
    );

    let route = runtime
        .plan_committed_patch(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            execution,
        )
        .expect("bridge should plan route for explanation reconstruction");
    runtime
        .deliver_invalidation(route, execution)
        .expect("bridge should deliver route before explanation reconstruction");

    let explanation = runtime
        .diagnostics()
        .explain_last_route_record()
        .expect("bridge should explain the last canonical route record");

    assert_eq!(explanation.route_entries().len(), 1);
    assert_eq!(explanation.invalidation_targets().len(), 1);
    assert_eq!(
        explanation.snapshot_identity().as_str(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a").as_str()
    );
    let entry = &explanation.route_entries()[0];
    assert!(matches!(
        entry.entity_identity(),
        RouteEntityIdentity::TruthSurface(_)
    ));
    assert_eq!(entry.aspect_key().as_str(), "profile");
    assert_eq!(
        entry.target_canonical_basis(),
        expected_field_target_basis(&avatar_field),
    );
    assert_eq!(
        entry
            .target()
            .field_locator()
            .expect("route diagnostics should retain typed target field locator")
            .field_path()
            .fields()[0]
            .as_str(),
        "avatar"
    );
    assert!(!entry.target().projection_mask().is_whole_aspect());
    assert_eq!(
        entry.source_target().surface_kind(),
        TruthDeltaSurfaceKind::EntityField
    );
    assert_eq!(entry.mapping_id().as_str(), "profile-surface-widening");
    assert_eq!(entry.signal_scope(), "signal.profile.widening");
    assert_eq!(
        explanation.invalidation_targets()[0].signal_scope(),
        "signal.profile.widening"
    );
}
#[test]
fn bridge_route_explanation_exposes_fine_grained_match_status() {
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
        .expect("bridge should plan route with fine-grained aspect registration");
    runtime
        .deliver_invalidation(route, execution)
        .expect("bridge should deliver route before explanation reconstruction");

    let explanation = runtime
        .diagnostics()
        .explain_last_route_record()
        .expect("bridge should explain the last canonical route record");

    let entry = &explanation.route_entries()[0];
    assert_eq!(
        entry.truth_surface_kind(),
        TruthDeltaSurfaceKind::EntityField
    );
    assert_eq!(
        entry.fine_grained_match_status(),
        FineGrainedMatchStatus::Matched
    );
    assert_eq!(
        entry.aspect_registration_id().map(|id| id.as_str()),
        Some("profile-name-field")
    );
    assert_eq!(
        entry.subscription_slice_kind(),
        Some(&SubscriptionSliceKind::SignalField)
    );
    assert_eq!(
        entry.slice_widening_policy(),
        Some(SliceWideningPolicy::Disallow)
    );
    assert_eq!(explanation.subscription_slices().len(), 1);
    assert_eq!(
        explanation.subscription_slices()[0].slice_kind(),
        &SubscriptionSliceKind::SignalField
    );
    assert_eq!(
        explanation.subscription_slices()[0].native_target_basis(),
        expected_field_target_basis(&name_field),
    );
}

fn expected_field_target_basis(field: &worth_foundational::facade::FieldKey) -> String {
    let field = field.as_str();
    format!(
        "committed-patch-target|locator=version=bridge.committed-patch-target.v1;domain=locator;entries=[locus=named:aspect_field.aspect_key,kind=locator,value=exact-text:profile;locus=named:aspect_field.authority,kind=locator,value=exact-text:authoritative;locus=named:aspect_field.field_path,kind=locator,value=exact-text:{field};locus=named:aspect_field.kind,kind=locator,value=exact-text:aspect]|mutation-mask=version=bridge.committed-patch-target.v1;domain=aspect-mask;entries=[locus=named:profile.mutation.field.{field},kind=mask,value=exact-text:{field}]|projection-mask=version=bridge.committed-patch-target.v1;domain=aspect-mask;entries=[locus=named:profile.projection.field.{field},kind=mask,value=exact-text:{field}]|kind=entity-field"
    )
}
#[test]
fn bridge_continuity_explanation_reconstructs_canonical_continuity_truth() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let source = InMemoryRelationalBridgeSource::default();
    source.insert_committed_patch(committed_patch(
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        worth_foundational::facade::FieldKey::new("name".to_owned())
            .expect("valid harness field key"),
    ));
    source.insert_snapshot(field_slice_snapshot(
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        "alice",
    ));
    let runtime = crate::facade::RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .with_continuity_lineage_source(ExplanationContinuityLineageSource)
        .register_mapping(registration())
        .register_aspect_mapping(field_aspect_registration())
        .build()
        .expect("runtime should build");

    let route = runtime
        .plan_committed_patch_with_mapping_context(
            BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-a",
            )),
            crate::facade::BridgeMappingContext::default().with_lineage_context(
                BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
                    crate::truth_identity_fixtures::truth_branch_fixture("main"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                )),
            ),
            execution,
        )
        .expect("route should plan");
    let result = runtime
        .deliver_invalidation(route, execution)
        .expect("delivery should succeed");
    let route_record = runtime
        .diagnostics()
        .route_record_for_route_identity(result.result_summary().route_identity())
        .expect("route record should be retained");
    let requests = runtime
        .plan_continuity_requests(&route_record)
        .expect("continuity requests should plan");
    let packet = runtime
        .plan_historical_lineage_packet(&requests)
        .expect("historical lineage packet should plan");
    let resolved = runtime
        .resolve_lineage_continuity(&packet)
        .expect("continuity should resolve");
    let artifact = runtime.lower_continuity_artifact(&resolved);
    let canonical = runtime.canonicalize_continuity_record(&route_record, &requests, &artifact);

    let explanation = runtime.diagnostics().explain_continuity_record(&canonical);

    assert_eq!(explanation.route_identity(), route_record.route_identity());
    assert_eq!(
        explanation.source_snapshot().as_str(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a").as_str()
    );
    assert_eq!(
        explanation.source_branch().as_str(),
        crate::truth_identity_fixtures::truth_branch("main").as_str()
    );
    assert_eq!(explanation.continuity_outcomes().len(), 1);
    assert_eq!(
        explanation.continuity_outcomes()[0].outcome_class(),
        crate::facade::BridgeContinuityOutcomeClass::ContinuesAsSingleSuccessor
    );
    assert_eq!(explanation.remapped_slices().len(), 1);
    assert_eq!(
        explanation.remapped_slices().slices()[0].entity_identity(),
        "entity:0:4:2"
    );
}

mod bulk;
