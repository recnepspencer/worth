//! Post-receipt delivery/recovery observes the existing publication in every leg.
use super::*;
fn exercise(worker: bool, cold: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let mut host = world::install_program_with_other_model::<program::Program>(limits, 2, Some(3));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 7,
        })
        .without_source()
        .idempotency(&"read-only-discovered-delivery".to_owned())
        .execute_performed_discovered::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(done) = outcome else {
        panic!("the source and all four outputs co-commit")
    };
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 4
    );
    let head = courts::native_head(&host);
    let receipt = done.receipt().clone();
    let mut original = done
        .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    host.delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        original
            .required_output_mut()
            .advance(&host, &request)
            .unwrap(),
        WorthQueryDiscoveredProgramOutputProgress::Pending
    ));
    let mut recovered = request
        .advanced()
        .recover_discovered_required_outputs::<program::Program, program::Tree>(
            &host,
            &receipt,
            WorthQueryOutputDemandControls::default(),
        )
        .unwrap_or_else(|cause| panic!("{cause:?}"));
    let mut settled = false;
    for _ in 0..128 {
        if let WorthQueryDiscoveredProgramOutputProgress::Settled(result) =
            recovered.advance(&host, &request).unwrap()
        {
            assert_eq!(result.root_outputs().count(), 2);
            settled = true;
            break;
        }
    }
    assert!(settled);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 4
    );
    assert_eq!(courts::native_head(&host), head);
    drop(recovered);
    drop(original);
    if cold {
        let checkpoint = super::super::super::failure::checkpoint(&host);
        drop(request);
        drop(principal);
        drop(host);
        host = world::restore_program_checkpoint::<program::Program>(checkpoint, worker, false);
    }
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut demand = request
        .advanced()
        .start_program_outputs::<program::Program, program::DemandTree>(
            &host,
            first::Demand("a".into()),
            WorthQueryOutputDemandControls::default(),
        )
        .unwrap_or_else(|cause| panic!("{cause:?}"));
    let mut settled = false;
    for _ in 0..128 {
        if let WorthQueryApplicationProgramOutputProgress::Settled(_) =
            demand.advance(&host, &request).unwrap()
        {
            settled = true;
            break;
        }
    }
    assert!(settled);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(courts::native_head(&host), head);
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        70
    );
    assert_eq!(
        request
            .query(query::second_output::Read("second:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        1007
    );
    assert_eq!(courts::native_head(&host), head);
}
#[test]
fn interrupted_discovered_delivery_recovers_without_producers_or_commits() {
    exercise(false, false)
}
#[test]
fn worker_only_interrupted_discovered_delivery_recovers_without_producers_or_commits() {
    exercise(true, false)
}
#[test]
fn cold_discovered_delivery_and_current_demand_are_read_only() {
    exercise(false, true)
}
#[test]
fn worker_only_cold_discovered_delivery_and_current_demand_are_read_only() {
    exercise(true, true)
}
