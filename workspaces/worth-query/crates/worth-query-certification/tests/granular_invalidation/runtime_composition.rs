use crate::host_world::CourtroomWorld;
use crate::query_runtime_world::{
    build_primary_query_world, build_primary_query_world_with_dimensions,
    build_with_foreign_snapshot_adapter, PrimaryQueryScale,
};
use worth_query::facade::domain::{
    bind_primary_runtime_granular_invalidations, maintain_primary_runtime_granular_batch,
    maintain_primary_runtime_granular_invalidations, WorthQueryPrimaryGranularMaintenanceDenial,
    WorthQueryPrimaryGranularMaintenanceOutcome,
};
use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

pub fn assert_primary_runtime_composition() {
    let mut world = CourtroomWorld::publish("blocked");
    let mut query = build_primary_query_world(&world);
    assert_eq!(query.observations.full_target_reads(), 1);
    assert_eq!(query.observations.exact_record_reads(), 0);
    let mut suppressed = observe(&mut world);
    assert_eq!(suppressed.committed_operation_count(), 0);
    assert!(suppressed.take_granular_invalidation_batch().is_empty());

    world.amend_intent(1, "active", "ready");
    let mut reconsidered = observe(&mut world);
    assert_eq!(reconsidered.committed_operation_count(), 1);
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        world.application.granular_invalidation_installation(),
    );
    let foreign_host = CourtroomWorld::publish("blocked");
    let mut foreign_query = build_primary_query_world(&foreign_host);
    let foreign_exact_reads = foreign_query.observations.exact_record_reads();
    assert!(matches!(
        maintain_primary_runtime_granular_invalidations(
            &foreign_query.live,
            &mut foreign_query.workspace,
            &binding,
            &mut reconsidered,
        ),
        Err(WorthQueryPrimaryGranularMaintenanceDenial::ForeignPrimaryRuntime)
    ));
    assert_eq!(
        foreign_query.observations.exact_record_reads(),
        foreign_exact_reads
    );
    let batch = reconsidered.take_granular_invalidation_batch();
    let lower = batch.observation();
    assert_eq!(lower.direct_truth_delivery_count(), 1);
    assert_eq!(lower.signal_performed_delivery_count(), 1);
    assert_eq!(lower.bridge_performed().source_load_attempts, 1);
    assert_eq!(lower.bridge_performed().truth_targets_admitted, 1);
    assert_eq!(lower.bridge_performed().signal_seeds_emitted, 1);
    assert_eq!(
        lower
            .signal_performed()
            .value(worth_signal::facade::adapters::InvalidationPerformedCounter::NodesEvaluated),
        1
    );
    let performed =
        maintain_primary_runtime_granular_batch(&query.live, &mut query.workspace, &binding, batch)
            .expect("the exact primary delivery must maintain the Query projection");
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = performed else {
        panic!("the changed primary record must perform Query maintenance")
    };
    assert_eq!(performed.maintenance_operation_count(), 1);
    assert_eq!(performed.consumer_publication_count(), 1);
    assert_eq!(performed.lower_truth_delivery_count(), 1);
    assert_eq!(performed.lower_signal_performed_delivery_count(), 1);
    let admission = performed.admission_counters();
    assert_eq!(admission.delivery_changes_examined(), 1);
    assert_eq!(admission.locality_entries_examined(), 1);
    assert_eq!(admission.candidate_deliveries_returned(), 1);
    assert_eq!(admission.admitted_impacts(), 1);
    assert!(admission.impact_index_probes() > 0);
    let maintenance = performed.maintenance_counters();
    assert_eq!(maintenance.maintenance_operations(), 1);
    assert_eq!(maintenance.projected_fields(), 1);
    assert_eq!(maintenance.consumer_publications(), 1);
    let effect = performed.deliveries()[0]
        .effect()
        .projection_patch()
        .expect("the scalar maintenance publication must carry the performed field patch");
    assert_eq!(effect.affected_entities().len(), 1);
    assert_eq!(effect.fields().len(), 1);
    assert_eq!(
        effect.fields()[0]
            .field_path()
            .canonical_field_path()
            .expect("the performed gate patch must retain its canonical source path")
            .fields()
            .iter()
            .map(worth_foundational::facade::FieldKey::as_str)
            .collect::<Vec<_>>(),
        ["IntentFacts", "IntentGateField"]
    );
    assert!(!effect.fact_set_digest().is_empty());
    assert!(!effect.identity().is_empty());
    assert_eq!(query.observations.exact_record_reads(), 1);
    assert_eq!(query.observations.full_target_reads(), 1);

    assert_reinstallation_revokes_captured_delivery();
}

pub fn assert_head_advance_preserves_admitted_granular_read() {
    let mut world = CourtroomWorld::publish("blocked");
    let mut query = build_primary_query_world(&world);
    drop(world.conditional_clock());
    world.amend_intent(1, "active", "ready");
    let mut receipt = observe(&mut world);
    let batch = receipt.take_granular_invalidation_batch();
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        world.application.granular_invalidation_installation(),
    );
    let reads_before = query.observations.exact_record_reads();

    world.amend_gate_only("blocked");

    let outcome =
        maintain_primary_runtime_granular_batch(&query.live, &mut query.workspace, &binding, batch)
            .expect("the retained exact basis must survive a later primary head advance");
    assert!(matches!(
        outcome,
        WorthQueryPrimaryGranularMaintenanceOutcome::Performed(_)
    ));
    assert_eq!(query.observations.exact_record_reads(), reads_before + 1);
}

pub fn assert_granular_receipt_uses_execution_snapshot_basis() {
    let mut world = CourtroomWorld::publish("blocked");
    let mut ambient_snapshot_world = CourtroomWorld::publish("blocked");
    let mut query = build_with_foreign_snapshot_adapter(&world, &ambient_snapshot_world);
    drop(world.conditional_clock());
    ambient_snapshot_world.amend_intent(8, "active", "ready");

    world.amend_intent(1, "active", "ready");
    let mut receipt = observe(&mut world);
    let batch = receipt.take_granular_invalidation_batch();
    let expected_bridge_snapshot = batch
        .source_read_basis()
        .expect("the primary execution batch must carry its settled source basis")
        .snapshot()
        .clone();
    let expected_snapshot =
        worth_query::facade::foundation::WorthQuerySnapshotIdentity::from_bridge_snapshot_projection(
            expected_bridge_snapshot,
        )
        .expect("the primary execution snapshot must retain relational identity");
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        world.application.granular_invalidation_installation(),
    );
    let outcome =
        maintain_primary_runtime_granular_batch(&query.live, &mut query.workspace, &binding, batch)
            .expect("the current primary batch must perform against its own source basis");
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the exact primary change must perform maintenance")
    };
    let (refresh, _) = performed.into_parts();
    let actual_snapshot = refresh
        .authority()
        .basis_authority()
        .snapshot_identity()
        .expect("the refreshed projection must retain its source snapshot");
    assert_eq!(
        actual_snapshot.evidence_identity(),
        expected_snapshot.evidence_identity()
    );
}

pub fn assert_unrelated_bridge_mapping_slope() {
    let baseline = observe_scaled_primary_world(0, PrimaryQueryScale::default(), false);
    let wide = observe_scaled_primary_world(
        0,
        PrimaryQueryScale {
            unrelated_bridge_mappings: 64,
            ..PrimaryQueryScale::default()
        },
        false,
    );
    assert_eq!(wide, baseline);
}

pub fn assert_unrelated_result_row_slope() {
    let baseline = observe_scaled_primary_world(0, PrimaryQueryScale::default(), false);
    let wide = observe_scaled_primary_world(64, PrimaryQueryScale::default(), false);
    assert_eq!(wide, baseline);
}

pub fn assert_unrelated_signal_subscriber_slope() {
    let baseline = observe_scaled_primary_world(0, PrimaryQueryScale::default(), false);
    let wide = observe_scaled_primary_world(
        0,
        PrimaryQueryScale {
            unrelated_signal_subscribers: 64,
            ..PrimaryQueryScale::default()
        },
        false,
    );
    assert_eq!(wide, baseline);
}

pub fn assert_unrelated_installed_query_slope() {
    let baseline = observe_scaled_primary_world(0, PrimaryQueryScale::default(), false);
    let wide = observe_scaled_primary_world(
        0,
        PrimaryQueryScale {
            install_unrelated_query: true,
            ..PrimaryQueryScale::default()
        },
        false,
    );
    assert_eq!(wide, baseline);
}

pub fn assert_returned_bridge_candidate_rejection_slope() {
    let baseline = observe_scaled_primary_world(0, PrimaryQueryScale::default(), false);
    let noisy = observe_scaled_primary_world(0, PrimaryQueryScale::default(), true);
    assert_eq!(noisy.signal, baseline.signal);
    assert_eq!(noisy.query_admission, baseline.query_admission);
    assert_eq!(noisy.query_maintenance, baseline.query_maintenance);
    assert_eq!(noisy.exact_record_reads, baseline.exact_record_reads);
    assert_eq!(
        noisy.bridge.truth_targets_admitted,
        baseline.bridge.truth_targets_admitted
    );
    assert_eq!(
        noisy.bridge.correspondence_lookups - baseline.bridge.correspondence_lookups,
        4
    );
    assert_eq!(
        noisy.bridge.semantic_match_checks - baseline.bridge.semantic_match_checks,
        4
    );
    assert_eq!(
        noisy.bridge.projection_rejections - baseline.bridge.projection_rejections,
        4
    );
}

#[derive(Debug, Eq, PartialEq)]
struct ObservedCompositionWork {
    bridge: worth_query_execution::facade::primary_graph::WorthQueryBridgeGranularDeliveryCounters,
    signal: worth_signal::facade::adapters::SignalInvalidationRealizedCounters,
    query_admission: worth_query::facade::domain::WorthQueryGranularAdmissionCounters,
    query_maintenance: worth_query::facade::domain::WorthQueryGranularMaintenanceCounters,
    exact_record_reads: usize,
}

fn observe_scaled_primary_world(
    unrelated_result_rows: usize,
    scale: PrimaryQueryScale,
    rejected_bridge_candidates: bool,
) -> ObservedCompositionWork {
    use crate::query_runtime_world::ConsumerProfile;

    let mut host = CourtroomWorld::publish_with_unrelated_rows("blocked", unrelated_result_rows);
    let mut query =
        build_primary_query_world_with_dimensions(&host, ConsumerProfile::ValuePatch, scale);
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        host.application.granular_invalidation_installation(),
    );
    drop(host.conditional_clock());
    if rejected_bridge_candidates {
        host.supersede_intent(2, 6, "active", "changed", "ready");
    } else {
        host.amend_gate_only("ready");
    }
    let mut observation = observe(&mut host);
    let batch = observation.take_granular_invalidation_batch();
    let lower = batch.observation();
    let outcome =
        maintain_primary_runtime_granular_batch(&query.live, &mut query.workspace, &binding, batch)
            .unwrap();
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the exact scaled world must perform maintenance")
    };
    ObservedCompositionWork {
        bridge: lower.bridge_performed(),
        signal: lower.signal_performed(),
        query_admission: performed.admission_counters(),
        query_maintenance: performed.maintenance_counters(),
        exact_record_reads: query.observations.exact_record_reads(),
    }
}

fn assert_reinstallation_revokes_captured_delivery() {
    let mut host = CourtroomWorld::publish("blocked");
    let mut query = build_primary_query_world(&host);
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        host.application.granular_invalidation_installation(),
    );
    host.amend_intent(1, "active", "ready");
    let mut receipt = observe(&mut host);
    host.reinstall_conditional_runtime().unwrap();
    assert!(matches!(
        maintain_primary_runtime_granular_invalidations(
            &query.live,
            &mut query.workspace,
            &binding,
            &mut receipt,
        ),
        Err(WorthQueryPrimaryGranularMaintenanceDenial::ForeignPrimaryRuntime)
    ));
    assert_eq!(query.observations.exact_record_reads(), 0);
}

fn observe(
    world: &mut CourtroomWorld,
) -> worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationReceipt<
    crate::adapters::CourtroomClock,
> {
    match world.conditional_clock().observe() {
        WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => receipt,
        _ => panic!("the due courtroom observation must be accepted"),
    }
}
