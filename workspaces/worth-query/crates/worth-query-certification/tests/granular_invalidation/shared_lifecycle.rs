use crate::host_world::CourtroomWorld;
use crate::query_runtime_world::{
    build_primary_query_world, build_shared_primary_query_world, IntentSourceProjection,
};
use worth_query::facade::domain::{
    bind_primary_runtime_granular_invalidations,
    bind_shared_primary_runtime_granular_invalidations, maintain_primary_runtime_granular_batch,
    maintain_primary_runtime_granular_invalidations,
    maintain_shared_primary_runtime_granular_batch, WorthQueryPrimaryGranularMaintenanceDenial,
    WorthQueryPrimaryGranularMaintenanceOutcome, WorthQuerySharedPrimaryGranularMaintenanceOutcome,
};
use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

pub fn assert_shared_consumer_slope() {
    let mut single_host = CourtroomWorld::publish("blocked");
    let mut single = build_primary_query_world(&single_host);
    let single_binding = bind_primary_runtime_granular_invalidations(
        &single.live,
        single_host.application.granular_invalidation_installation(),
    );
    single_host.amend_gate_only("ready");
    let mut single_observation = observe(&mut single_host);
    let single_batch = single_observation.take_granular_invalidation_batch();
    let single_lower = single_batch.observation();
    let single_performed = match maintain_primary_runtime_granular_batch(
            &single.live,
            &mut single.workspace,
            &single_binding,
            single_batch,
        )
        .expect("the single consumer must perform")
    {
        WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) => performed,
        WorthQueryPrimaryGranularMaintenanceOutcome::NoRelevantChange(no_change) => panic!(
            "the single consumer change must remain relevant: duplicate={}, settled={}, irrelevant={}, suppressed={}",
            no_change.duplicate_delivery_count(),
            no_change.already_settled_delivery_count(),
            no_change.irrelevant_delivery_count(),
            no_change.suppressed_impact_count(),
        ),
    };

    let mut shared_host = CourtroomWorld::publish("blocked");
    let mut shared = build_shared_primary_query_world(&shared_host);
    let shared_binding = bind_shared_primary_runtime_granular_invalidations(
        &shared.subject,
        shared_host.application.granular_invalidation_installation(),
    );
    shared_host.amend_gate_only("ready");
    let mut shared_observation = observe(&mut shared_host);
    let shared_batch = shared_observation.take_granular_invalidation_batch();
    let shared_lower = shared_batch.observation();
    let WorthQuerySharedPrimaryGranularMaintenanceOutcome::Performed(shared_performed) =
        maintain_shared_primary_runtime_granular_batch(
            &[&shared.subject, &shared.candidate],
            &mut shared.workspace,
            &shared_binding,
            shared_batch,
        )
        .expect("the shared consumers must perform")
    else {
        panic!("the shared consumer change must remain relevant")
    };

    assert_eq!(
        shared_lower.bridge_performed(),
        single_lower.bridge_performed()
    );
    assert_eq!(
        shared_lower.signal_performed(),
        single_lower.signal_performed()
    );
    assert_eq!(
        shared_performed.admission_counters(),
        single_performed.admission_counters()
    );
    assert_eq!(single_performed.maintenance_operation_count(), 1);
    assert_eq!(shared_performed.shared_execution_count(), 1);
    assert_eq!(single_performed.consumer_publication_count(), 1);
    assert_eq!(shared_performed.publications().len(), 2);
}

pub fn assert_correspondence_rebind_restore() {
    let (foreign_reconstruction, foreign_record) = foreign_reinstallation_receipt();
    let mut host = CourtroomWorld::publish("blocked");
    let mut query = build_primary_query_world(&host);
    let old_binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        host.application.granular_invalidation_installation(),
    );
    let mut baseline = observe(&mut host);
    assert!(baseline.take_granular_invalidation_batch().is_empty());
    host.amend_intent(1, "active", "waiting");
    let mut delayed = observe(&mut host);

    let reconstruction = host.reinstall_conditional_runtime().unwrap();
    assert_reconstruction(reconstruction.lower_runtime_reconstitution());
    let reads_before = query.observations.exact_record_reads();
    let installation = host.application.granular_invalidation_installation();
    let query_rebind = query
        .workspace
        .rebind_primary_graph_source(
            &reconstruction,
            IntentSourceProjection::new(
                host.intent_record_identity(),
                std::sync::Arc::clone(&query.observations),
            ),
        )
        .expect("Query must rebind its source to the reconstructed primary runtime");
    assert!(query_rebind.displaced_previous_runtime());
    assert!(query_rebind.successor_source_readmitted());
    let stale_reuse = query.workspace.rebind_primary_graph_source(
        &reconstruction,
        IntentSourceProjection::new(
            host.intent_record_identity(),
            std::sync::Arc::clone(&query.observations),
        ),
    );
    assert!(
        stale_reuse.is_err(),
        "a consumed rebind transition must be stale"
    );
    let foreign_rebind = query.workspace.rebind_primary_graph_source(
        &foreign_reconstruction,
        IntentSourceProjection::new(foreign_record, std::sync::Arc::clone(&query.observations)),
    );
    assert!(
        foreign_rebind.is_err(),
        "a valid reinstallation receipt from another runtime lineage must deny"
    );
    let current_binding = bind_primary_runtime_granular_invalidations(&query.live, installation);
    let delayed_denial = maintain_primary_runtime_granular_invalidations(
        &query.live,
        &mut query.workspace,
        &current_binding,
        &mut delayed,
    );
    match delayed_denial {
        Err(WorthQueryPrimaryGranularMaintenanceDenial::Admission(_)) => {}
        Err(other) => panic!("unexpected delayed delivery denial: {other:?}"),
        Ok(_) => panic!("the delayed delivery unexpectedly performed"),
    }
    assert_eq!(query.observations.exact_record_reads(), reads_before);

    host.amend_intent(2, "active", "ready");
    let mut current_observation = observe(&mut host);
    assert!(matches!(
        maintain_primary_runtime_granular_invalidations(
            &query.live,
            &mut query.workspace,
            &old_binding,
            &mut current_observation,
        ),
        Err(WorthQueryPrimaryGranularMaintenanceDenial::ForeignPrimaryRuntime)
    ));
    assert_eq!(query.observations.exact_record_reads(), reads_before);
    let current_batch = current_observation.take_granular_invalidation_batch();
    let lower_current = current_batch.observation();
    assert_eq!(lower_current.direct_truth_delivery_count(), 1);
    assert_eq!(lower_current.signal_performed_delivery_count(), 1);
    assert_eq!(
        lower_current.signal_performed().value(
            worth_signal::facade::adapters::InvalidationPerformedCounter::RecoveryReconstructionWork,
        ),
        0,
        "ordinary performed counters must exclude restore reconstruction work",
    );
    let current = maintain_primary_runtime_granular_batch(
        &query.live,
        &mut query.workspace,
        &current_binding,
        current_batch,
    )
    .expect("the restored owners must admit a newly bound current delivery");
    let performed = match current {
        WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) => performed,
        WorthQueryPrimaryGranularMaintenanceOutcome::NoRelevantChange(no_change) => panic!(
            "the rebound current delivery must perform: duplicate={}, settled={}, irrelevant={}, suppressed={}",
            no_change.duplicate_delivery_count(),
            no_change.already_settled_delivery_count(),
            no_change.irrelevant_delivery_count(),
            no_change.suppressed_impact_count(),
        ),
    };
    assert_eq!(performed.lower_truth_delivery_count(), 1);
    assert_eq!(performed.lower_signal_performed_delivery_count(), 1);
    assert_eq!(performed.maintenance_operation_count(), 1);
    assert_eq!(performed.consumer_publication_count(), 1);
    assert_eq!(query.observations.exact_record_reads(), reads_before + 1);
}

fn foreign_reinstallation_receipt() -> (
    worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeReinstallationReceipt,
    worth_query_host::facade::primary_graph::RelationalBridgeRecordIdentityParts,
) {
    let mut foreign_host = CourtroomWorld::publish("blocked");
    let record = foreign_host.intent_record_identity();
    let reinstallation = foreign_host
        .reinstall_conditional_runtime()
        .expect("the foreign host must produce its own valid reinstallation receipt");
    (reinstallation, record)
}

fn assert_reconstruction(
    report: worth_runtime_bridge::facade::BridgeConditionalRuntimeReconstitutionReport,
) {
    assert_eq!(report.signal().service_readmission_count(), 1);
    assert_eq!(report.readmitted_lowering_count(), 1);
    assert!(report
        .correspondence()
        .exact_semantic_dependency_index_parity());
    assert!(report.correspondence().exact_mapping_index_parity());
    assert!(report.correspondence().exact_index_parity());
}

fn observe(
    world: &mut CourtroomWorld,
) -> worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationReceipt<
    crate::adapters::CourtroomClock,
> {
    match world.conditional_clock().observe() {
        WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => receipt,
        WorthQueryConditionalClockObservationOutcome::Duplicate(_) => {
            panic!("the due courtroom observation was duplicate")
        }
        WorthQueryConditionalClockObservationOutcome::Stale => {
            panic!("the due courtroom observation was stale")
        }
        WorthQueryConditionalClockObservationOutcome::Reordered => {
            panic!("the due courtroom observation was reordered")
        }
        WorthQueryConditionalClockObservationOutcome::Closed => {
            panic!("the due courtroom observation was closed")
        }
        WorthQueryConditionalClockObservationOutcome::Failed(_) => {
            panic!("the due courtroom observation failed")
        }
    }
}
