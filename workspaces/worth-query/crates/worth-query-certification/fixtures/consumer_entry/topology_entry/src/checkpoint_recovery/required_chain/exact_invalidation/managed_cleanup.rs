//! A clean caller cannot suppress the real terminal of a dirty independent queued root.
use super::*;
use worth_query_host::facade::primary_graph::{
    BridgeExecutionBasisFinalizationFailureKind as Bridge,
    WorthQueryManagedRunCleanupFailureKind as Native,
    WorthQueryMutationCleanupFailureKind as Failure,
};
#[test]
fn noncaller_queued_producer_keeps_unique_cleanup_after_refusal_suppression() {
    let _observation = primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let application = install(None, ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_7d00);
    let dirty = Ring::seeded(0);
    let later = Ring::seeded(1);
    let independent = Ring::seeded(2);
    let mut a = root!(court, dirty.key("a"), "real initial queued root");
    let mut later_root = root!(court, later.key("a"), "later independent queued root");
    let mut b = root!(court, independent.key("a"), "independent cached caller");
    settled!(court, a, "initial A");
    settled!(court, later_root, "initial later root");
    settled!(court, b, "initial B");
    court.write_y(&dirty.key("a"), 5, "real source mutation queues A");
    court.write_y(&later.key("a"), 9, "real source mutation queues later root");
    assert!(application.queued_required_work_for_test() > 0);
    assert_eq!(court.length(&dirty.key("a")), 2);
    assert_eq!(court.length(&independent.key("a")), 2);
    let guard = application
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let denial = b
        .demand
        .advance(&request)
        .err()
        .expect("a NONCALLER native owner cannot be suppressed after B is decided");
    let WorthQueryApplicationOutputDemandDenial::Demand(mut denial) = denial else {
        panic!("native producer stop");
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryOutputDemandDenialKind::ManagedCleanupRequired(Failure::Native(
            Native::BridgeFinalization(Bridge::SignalRuntimeThreadAffinityViolation)
        ))
    );
    let mut diagnostic = denial.clone();
    assert!(diagnostic.take_managed_cleanup().is_none());
    assert!(
        denial.take_managed_cleanup().is_none(),
        "the actual owner is retained by B's demand"
    );
    drop(diagnostic);
    drop(denial);
    drop(guard);
    let run = b
        .demand
        .managed_cleanup_failure()
        .expect("diagnostic destruction cannot discard actual queued cleanup")
        .run_identity()
        .to_owned();
    // B did not run: A's independently queried output is the publication that failed cleanup.
    assert_eq!(court.length(&dirty.key("a")), 6);
    assert_eq!(
        court.length(&later.key("a")),
        2,
        "first native cleanup refusal stops independent publication, not just diagnostic collection"
    );
    assert_eq!(court.length(&independent.key("a")), 2);
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let refused = std::thread::scope(|threads| {
        let demand = &mut b.demand;
        let application = &application;
        threads
            .spawn(move || {
                let (scope, principal) = authenticate(application);
                demand.release_managed_cleanup(&application.request(&principal, &scope))
            })
            .join()
            .unwrap()
    })
    .err()
    .expect("the actual Signal terminal refuses a different native thread");
    assert!(
        matches!(refused, WorthQueryApplicationOutputDemandDenial::Demand(ref denial)
        if denial.kind() == primary_graph::WorthQueryOutputDemandDenialKind::ManagedCleanupRequired(Failure::Native(
            Native::BridgeFinalization(Bridge::SignalRuntimeThreadAffinityViolation))))
    );
    assert_eq!(
        b.demand.managed_cleanup_failure().unwrap().run_identity(),
        run
    );
    let released = b
        .demand
        .release_managed_cleanup(&request)
        .unwrap()
        .expect("the same native owner releases");
    assert_eq!(released.inspection().run_identity(), run);
    assert!(released.inspection().resources_released());
    assert!(b.demand.managed_cleanup_failure().is_none());
    assert!(
        b.demand.advance(&request).is_err(),
        "cleanup release never authorizes producer rerun or business settlement"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(court.length(&dirty.key("a")), 6);
    assert_eq!(
        court.length(&later.key("a")),
        2,
        "first native cleanup refusal stops independent publication, not just diagnostic collection"
    );
    assert_eq!(court.length(&independent.key("a")), 2);
}
