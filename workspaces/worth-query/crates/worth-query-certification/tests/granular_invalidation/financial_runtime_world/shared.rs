use worth_query::facade::domain::{
    bind_shared_primary_runtime_granular_invalidations,
    maintain_shared_primary_runtime_granular_batch,
    perform_prepared_shared_primary_runtime_granular_maintenance,
    prepare_shared_primary_runtime_granular_batch,
    WorthQuerySharedPrimaryGranularMaintenanceDenial,
    WorthQuerySharedPrimaryGranularMaintenanceOutcome,
    WorthQuerySharedPrimaryGranularSelectionOutcome,
};
use worth_query::facade::{domain, runtime};
use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalClockObservationOutcome, WorthQueryConditionalClockObservationReceipt,
};

use super::{host::FinancialCourtroomWorld, query};
use crate::adapters::CourtroomClock;

pub fn assert_shared_financial_execution_and_publication() {
    let mut host = FinancialCourtroomWorld::publish_curve();
    let mut query = query::build_shared_curve(&host);
    baseline_and_amend(&mut host);
    let binding = bind_shared_primary_runtime_granular_invalidations(
        &query.subject,
        host.application.granular_invalidation_installation(),
    );
    let mut observation = observe(&mut host);
    let outcome = maintain_shared_primary_runtime_granular_batch(
        &[&query.subject, &query.candidate],
        &mut query.workspace,
        &binding,
        observation.take_granular_invalidation_batch(),
    )
    .expect("the shared financial consumers must admit the 5y curve change");
    let WorthQuerySharedPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the shared financial consumers must perform one owner refresh")
    };
    assert_eq!(performed.shared_execution_count(), 1);
    assert_eq!(performed.publications().len(), 2);
    assert_eq!(performed.denied_consumer_count(), 0);
    assert_eq!(performed.maintenance_counters().maintenance_operations(), 1);
    assert_eq!(performed.maintenance_counters().consumer_publications(), 2);
    assert_eq!(
        performed
            .maintenance_counters()
            .authorization_revalidations(),
        2
    );
    let first = performed.publications()[0].consumer_delivery_authority();
    let second = performed.publications()[1].consumer_delivery_authority();
    assert_ne!(first.consumer_identity(), second.consumer_identity());
    assert_ne!(first.authority_identity(), second.authority_identity());
    assert_ne!(first.purpose_identity(), second.purpose_identity());
    assert_ne!(first.disclosure_identity(), second.disclosure_identity());
    assert_eq!(first.backpressure_posture(), "retain_within_window");
    assert_eq!(second.backpressure_posture(), "drop_with_gap_notice");
    host.amend_curve(3, 4_250, 5_100);
    host.curve_clock_control.push(3, 12);
    let mut reverse_observation = observe(&mut host);
    let reverse = maintain_shared_primary_runtime_granular_batch(
        &[&query.subject, &query.candidate],
        &mut query.workspace,
        &binding,
        reverse_observation.take_granular_invalidation_batch(),
    )
    .expect("the shared owner must compare the reverse against its performed baseline");
    let WorthQuerySharedPrimaryGranularMaintenanceOutcome::Performed(reverse) = reverse else {
        panic!("the shared B -> A change must not compare against the initial A baseline")
    };
    assert_eq!(reverse.publications().len(), 2);
    assert!(reverse.publications().iter().all(|publication| {
        publication
            .effect()
            .projection_patch()
            .is_some_and(|patch| !patch.fields().is_empty())
    }));
}

pub fn assert_shared_financial_disclosure_revalidation() {
    changed_policy_is_denied_before_refresh();
    revoked_lease_does_not_disturb_the_survivor();
}

fn changed_policy_is_denied_before_refresh() {
    let mut host = FinancialCourtroomWorld::publish_curve();
    let mut query = query::build_shared_curve(&host);
    baseline_and_amend(&mut host);
    let binding = bind_shared_primary_runtime_granular_invalidations(
        &query.subject,
        host.application.granular_invalidation_installation(),
    );
    let mut observation = observe(&mut host);
    let selected = prepare_shared_primary_runtime_granular_batch(
        &[&query.subject, &query.candidate],
        &mut query.workspace,
        &binding,
        observation.take_granular_invalidation_batch(),
    )
    .expect("both financial leases must select the exact 5y change");
    let WorthQuerySharedPrimaryGranularSelectionOutcome::Prepared(prepared) = selected else {
        panic!("the financial change must prepare shared maintenance")
    };
    query
        .candidate
        .admit_consumer_delivery_policy(
            &mut query.workspace,
            domain::WorthQuerySharedConsumerDeliveryPolicy::new(
                "regulatory-capital",
                "restricted-capital-v2",
                "regulatory-cursor",
                runtime::DeliveryBackpressurePolicy::TerminateConsumer,
            )
            .unwrap(),
        )
        .unwrap();
    assert!(matches!(
        perform_prepared_shared_primary_runtime_granular_maintenance(
            prepared,
            &[&query.subject, &query.candidate],
            &mut query.workspace,
            &binding,
        ),
        Err(WorthQuerySharedPrimaryGranularMaintenanceDenial::ConsumerSetMismatch)
    ));
}

fn revoked_lease_does_not_disturb_the_survivor() {
    let mut host = FinancialCourtroomWorld::publish_curve();
    let mut query = query::build_shared_curve(&host);
    baseline_and_amend(&mut host);
    let binding = bind_shared_primary_runtime_granular_invalidations(
        &query.subject,
        host.application.granular_invalidation_installation(),
    );
    let mut observation = observe(&mut host);
    let selected = prepare_shared_primary_runtime_granular_batch(
        &[&query.subject, &query.candidate],
        &mut query.workspace,
        &binding,
        observation.take_granular_invalidation_batch(),
    )
    .unwrap();
    let WorthQuerySharedPrimaryGranularSelectionOutcome::Prepared(prepared) = selected else {
        panic!("the financial change must prepare shared maintenance")
    };
    let disposed = match query.candidate.dispose(&mut query.workspace) {
        domain::WorthQuerySharedProjectionDisposalOutcome::Disposed(disposed) => disposed,
        domain::WorthQuerySharedProjectionDisposalOutcome::Stopped(_) => {
            panic!("the candidate financial lease must revoke")
        }
    };
    assert!(!disposed.release().owner_terminal());
    let outcome = perform_prepared_shared_primary_runtime_granular_maintenance(
        prepared,
        &[&query.subject],
        &mut query.workspace,
        &binding,
    )
    .expect("the surviving financial lease must receive the shared result");
    let WorthQuerySharedPrimaryGranularMaintenanceOutcome::Performed(performed) = outcome else {
        panic!("the surviving financial lease must publish")
    };
    assert_eq!(performed.shared_execution_count(), 1);
    assert_eq!(performed.publications().len(), 1);
    assert_eq!(performed.denied_consumer_count(), 1);
    assert_eq!(performed.maintenance_counters().authorization_denials(), 1);
}

fn baseline_and_amend(host: &mut FinancialCourtroomWorld) {
    assert!(matches!(
        host.conditional_clock(&host.curve_clock).unwrap().observe(),
        WorthQueryConditionalClockObservationOutcome::Accepted(_)
    ));
    host.amend_curve(2, 4_260, 5_100);
    host.curve_gate.release();
    host.curve_clock_control.push(2, 11);
}

fn observe(
    host: &mut FinancialCourtroomWorld,
) -> WorthQueryConditionalClockObservationReceipt<CourtroomClock> {
    match host.conditional_clock(&host.curve_clock).unwrap().observe() {
        WorthQueryConditionalClockObservationOutcome::Accepted(receipt) => receipt,
        _ => panic!("the due financial curve observation was not accepted"),
    }
}
