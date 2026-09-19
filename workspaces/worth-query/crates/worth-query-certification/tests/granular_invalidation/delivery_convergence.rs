use crate::host_world::CourtroomWorld;
use crate::query_runtime_world::build_primary_query_world;
use worth_query::facade::domain::{
    bind_primary_runtime_granular_invalidations, maintain_primary_runtime_granular_batch,
    WorthQueryPrimaryGranularMaintenanceOutcome,
};
use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

pub fn assert_duplicate_and_reordered_convergence() {
    let direct_then_performed = run_case(false);
    let performed_then_direct = run_case(true);
    assert_eq!(direct_then_performed.duplicate_count, 1);
    assert_eq!(performed_then_direct.duplicate_count, 1);
    assert_eq!(direct_then_performed.promotion_count, 1);
    assert_eq!(performed_then_direct.promotion_count, 0);
    assert_eq!(direct_then_performed.maintenance_count, 1);
    assert_eq!(performed_then_direct.maintenance_count, 1);
    assert_eq!(direct_then_performed.publication_count, 1);
    assert_eq!(performed_then_direct.publication_count, 1);
    assert_eq!(direct_then_performed.affected_entity_count, 1);
    assert_eq!(
        direct_then_performed.affected_entity_count,
        performed_then_direct.affected_entity_count
    );
    assert_eq!(
        direct_then_performed.projected_field_count,
        performed_then_direct.projected_field_count
    );
    assert_same_width_runtime_substitution_is_denied();
}

#[derive(Debug, PartialEq, Eq)]
struct ConvergenceObservation {
    duplicate_count: usize,
    promotion_count: usize,
    maintenance_count: usize,
    publication_count: usize,
    affected_entity_count: usize,
    projected_field_count: usize,
}

fn run_case(performed_first: bool) -> ConvergenceObservation {
    let mut host = CourtroomWorld::publish("blocked");
    let mut query = build_primary_query_world(&host);
    let installation = host.application.granular_invalidation_installation();
    let binding = bind_primary_runtime_granular_invalidations(&query.live, installation.clone());
    let mut baseline = observe(&mut host);
    assert!(baseline.take_granular_invalidation_batch().is_empty());
    host.amend_intent(1, "active", "ready");
    let mut changed = observe(&mut host);
    let performed = changed.take_granular_invalidation_batch();
    assert_eq!(performed.len(), 1);
    assert!(performed.bridge_deliveries()[0]
        .performed_signal()
        .is_some());
    let direct = performed
        .retain_direct_truth_transport(&installation)
        .expect("the current batch must retain a direct-truth transport copy");
    let batch = if performed_first {
        performed.merge_transport_batch(direct)
    } else {
        direct.merge_transport_batch(performed)
    }
    .expect("same-installation transport batches must merge");
    let lower = batch.observation();
    assert_eq!(lower.direct_truth_delivery_count(), 2);
    assert_eq!(lower.signal_performed_delivery_count(), 1);
    let outcome =
        maintain_primary_runtime_granular_batch(&query.live, &mut query.workspace, &binding, batch)
            .expect("duplicate direct/performed delivery must converge");
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the converged performed delivery must maintain Query")
    };
    let patch = performed.deliveries()[0]
        .effect()
        .projection_patch()
        .expect("the converged delivery must publish its field patch");
    ConvergenceObservation {
        duplicate_count: performed.duplicate_delivery_count(),
        promotion_count: performed.performed_promotion_count(),
        maintenance_count: performed.maintenance_operation_count(),
        publication_count: performed.consumer_publication_count(),
        affected_entity_count: patch.affected_entities().len(),
        projected_field_count: patch.fields().len(),
    }
}

fn assert_same_width_runtime_substitution_is_denied() {
    let mut current_host = CourtroomWorld::publish("blocked");
    let query = build_primary_query_world(&current_host);
    let current_batch = performed_batch(&mut current_host);

    let mut substituted_host = CourtroomWorld::publish("blocked");
    let substituted_batch = performed_batch(&mut substituted_host);
    let reads_before = query.observations.exact_record_reads();
    let denial = match current_batch.merge_transport_batch(substituted_batch) {
        Err(denial) => denial,
        Ok(_) => panic!("same-width delivery substitution from another runtime must deny"),
    };
    assert_eq!(
        denial,
        worth_query_execution::facade::primary_graph::WorthQueryGranularTransportMergeDenial::ForeignInstallation
    );
    assert_eq!(query.observations.exact_record_reads(), reads_before);
}

fn performed_batch(
    host: &mut CourtroomWorld,
) -> worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationDeliveryBatch {
    let mut baseline = observe(host);
    assert!(baseline.take_granular_invalidation_batch().is_empty());
    host.amend_intent(1, "active", "ready");
    let mut changed = observe(host);
    let batch = changed.take_granular_invalidation_batch();
    assert_eq!(batch.len(), 1);
    assert!(batch.bridge_deliveries()[0].performed_signal().is_some());
    batch
}

fn observe(
    world: &mut CourtroomWorld,
) -> worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationReceipt<
    crate::adapters::CourtroomClock,
> {
    match world.conditional_clock().observe() {
        WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => receipt,
        _ => panic!("the convergence observation must be accepted"),
    }
}
