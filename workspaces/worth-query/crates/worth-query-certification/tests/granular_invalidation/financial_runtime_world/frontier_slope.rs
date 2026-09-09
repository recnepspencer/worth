use worth_query::facade::domain::{
    bind_primary_runtime_granular_invalidations,
    maintain_primary_runtime_granular_collection_batch, WorthQueryGranularAdmissionCounters,
    WorthQueryGranularMaintenanceCounters, WorthQueryPrimaryGranularMaintenanceOutcome,
};
use worth_query_host::facade::primary_graph::WorthQueryConditionalClockObservationOutcome;

use super::{host::FinancialCourtroomWorld, query};

pub fn assert_frontier_expansion_slope() {
    let projected_value = observe(false);
    let ordered_window = observe(true);
    assert_eq!(
        ordered_window.direct_truth_deliveries,
        projected_value.direct_truth_deliveries
    );
    assert_eq!(
        ordered_window.signal_performed_deliveries,
        projected_value.signal_performed_deliveries
    );
    assert_eq!(
        ordered_window.admission.admitted_impacts(),
        projected_value.admission.admitted_impacts()
    );
    assert_eq!(
        ordered_window.admission.candidate_roles_returned()
            - projected_value.admission.candidate_roles_returned(),
        2
    );
    assert_eq!(ordered_window.maintenance.maintenance_operations(), 1);
    assert_eq!(projected_value.maintenance.maintenance_operations(), 1);
    assert!(ordered_window.maintenance.ordering_keys() > 0);
    assert!(ordered_window.maintenance.window_rows() > 0);
    assert_eq!(projected_value.maintenance.ordering_keys(), 0);
    assert_eq!(projected_value.maintenance.window_rows(), 0);
}

struct ObservedFrontier {
    direct_truth_deliveries: usize,
    signal_performed_deliveries: usize,
    admission: WorthQueryGranularAdmissionCounters,
    maintenance: WorthQueryGranularMaintenanceCounters,
}

fn observe(expand: bool) -> ObservedFrontier {
    let mut host = FinancialCourtroomWorld::publish_portfolio();
    let mut query = query::build_portfolio_with_unrelated_rows(&host, 64);
    assert!(matches!(
        host.conditional_clock(&host.portfolio_clock)
            .unwrap()
            .observe(),
        WorthQueryConditionalClockObservationOutcome::Accepted(_)
    ));
    host.portfolio_gate.release();
    if expand {
        host.amend_portfolio_rank(2, 3);
    } else {
        host.amend_portfolio_value(2, 5_120);
    }
    host.portfolio_clock_control.push(2, 11);
    let WorthQueryConditionalClockObservationOutcome::Accepted(mut receipt) = host
        .conditional_clock(&host.portfolio_clock)
        .unwrap()
        .observe()
    else {
        panic!("the frontier slope mutation must be observed")
    };
    let batch = receipt.take_granular_invalidation_batch();
    let lower = batch.observation();
    let binding = bind_primary_runtime_granular_invalidations(
        &query.live,
        host.application.granular_invalidation_installation(),
    );
    let outcome = maintain_primary_runtime_granular_collection_batch(
        &query.live,
        query
            .collection
            .as_mut()
            .expect("portfolio collection state"),
        &mut query.workspace,
        &binding,
        batch,
    )
    .expect("the frontier slope must perform");
    let WorthQueryPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the frontier slope mutation must publish")
    };
    ObservedFrontier {
        direct_truth_deliveries: lower.direct_truth_delivery_count(),
        signal_performed_deliveries: lower.signal_performed_delivery_count(),
        admission: performed.admission_counters(),
        maintenance: performed.maintenance_counters(),
    }
}
