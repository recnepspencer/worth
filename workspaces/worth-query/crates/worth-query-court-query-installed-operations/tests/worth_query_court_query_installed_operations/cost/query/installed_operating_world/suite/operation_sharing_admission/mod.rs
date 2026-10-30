pub(crate) use worth_query_testkit_query::facade::query::suite::operation_sharing_admission::*;

#[test]
fn exact_owner_lookup_remains_bounded_with_unrelated_active_owners() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = sharing_workspace(request_execution, "projection-sharing-unrelated-owners");
    let unrelated_a = singleton(request_execution, &mut workspace);
    let unrelated_b = singleton(request_execution, &mut workspace);
    let live = promote(request_execution, settle(&mut workspace), &mut workspace);
    let candidate = settle(&mut workspace).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("bounded sharing stopped: {}", stop.detail())
        }
    };
    assert_eq!(shared.counters().unrelated_registry_scans, 0);
    let (subject, candidate) = shared.into_leases();
    let subject_delivery = subject.drain(&mut workspace).unwrap();
    let candidate_delivery = candidate.drain(&mut workspace).unwrap();
    assert_eq!(subject_delivery.counters().unrelated_owner_scans, 0);
    assert_eq!(subject_delivery.counters().unrelated_lease_scans, 0);
    assert_eq!(subject_delivery.counters().lease_index_visits, 2);
    assert_eq!(subject_delivery.drain_counters().owner_index_lookups, 1);
    assert_eq!(subject_delivery.drain_counters().lease_index_lookups, 1);
    assert_eq!(
        subject_delivery.drain_counters().sharing_readmission_checks,
        2
    );
    assert_eq!(subject_delivery.drain_counters().unrelated_owner_scans, 0);
    assert_eq!(subject_delivery.drain_counters().unrelated_lease_scans, 0);
    assert_eq!(
        subject_delivery.maintenance_ordinal(),
        candidate_delivery.maintenance_ordinal()
    );
    drop((unrelated_a, unrelated_b, subject, candidate));
}
