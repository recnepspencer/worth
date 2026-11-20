//! A Host clock observation's granular batch merges only with a transport copy
//! from the same installation, in either order, without duplicating performed
//! Signal authority; a same-width batch from another installation is refused.

pub use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalClockObservationOutcome, WorthQueryGranularInvalidationDeliveryBatch,
    WorthQueryGranularTransportMergeDenial,
};
pub use worth_query_testkit_host::facade::host::world::CourtroomWorld;

pub fn assert_same_installation_transport_merge_in_both_orders(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    for performed_first in [false, true] {
        let mut host = CourtroomWorld::publish("blocked");
        let installation = host.application.granular_invalidation_installation();
        let performed = performed_batch(request_execution, &mut host);
        let direct = performed
            .retain_direct_truth_transport(&installation)
            .expect("the current batch must retain a direct-truth transport copy");
        let merged = if performed_first {
            performed.merge_transport_batch(direct)
        } else {
            direct.merge_transport_batch(performed)
        }
        .expect("same-installation transport batches must merge");
        let lower = merged.observation();
        assert_eq!(
            lower.direct_truth_delivery_count(),
            2,
            "order {performed_first}"
        );
        assert_eq!(
            lower.signal_performed_delivery_count(),
            1,
            "order {performed_first}"
        );
    }
}

pub fn assert_foreign_installation_merge_is_refused(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let mut current_host = CourtroomWorld::publish("blocked");
    let current_batch = performed_batch(request_execution, &mut current_host);
    let mut foreign_host = CourtroomWorld::publish("blocked");
    let foreign_batch = performed_batch(request_execution, &mut foreign_host);
    assert_eq!(current_batch.len(), foreign_batch.len());
    let denial = match current_batch.merge_transport_batch(foreign_batch) {
        Err(denial) => denial,
        Ok(_) => panic!("same-width delivery substitution from another installation must deny"),
    };
    assert_eq!(
        denial,
        WorthQueryGranularTransportMergeDenial::ForeignInstallation
    );
}

pub fn performed_batch(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &mut CourtroomWorld,
) -> WorthQueryGranularInvalidationDeliveryBatch {
    let mut baseline = observe(request_execution, host);
    assert!(baseline.take_granular_invalidation_batch().is_empty());
    host.amend_intent(1, "active", "ready");
    let mut changed = observe(request_execution, host);
    let batch = changed.take_granular_invalidation_batch();
    assert_eq!(batch.len(), 1);
    assert!(batch.bridge_deliveries()[0].performed_signal().is_some());
    batch
}

pub fn observe(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut CourtroomWorld,
) -> worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationReceipt<
    worth_query_testkit_host::facade::host::adapters::CourtroomClock,
> {
    match world.conditional_clock().observe(request_execution) {
        WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => receipt,
        _ => panic!("the convergence observation must be accepted"),
    }
}
