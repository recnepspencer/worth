//! Post-receipt delivery/recovery observes the existing publication in every leg.
use super::*;
use worth_relational::facade::inspection::observe_maintenance_work;
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
    // D1724: advance actual runnable work, but retain the original source.
    // Settled closes its custody; the receipt alone cannot reopen that source.
    let (progress, delivery) = observe_maintenance_work(8, || {
        original
            .required_output_mut()
            .advance(&host, &request)
            .unwrap()
    });
    assert!(matches!(
        progress,
        WorthQueryDiscoveredProgramOutputProgress::Progressed
    ));
    assert_eq!(
        delivery.published_rows.len() + delivery.omitted_publications,
        0
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 4
    );
    assert_eq!(courts::native_head(&host), head);
    let ((recovered, settled), recovery) = observe_maintenance_work(8, || {
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
            match recovered.advance(&host, &request).unwrap() {
                WorthQueryDiscoveredProgramOutputProgress::Progressed => {}
                WorthQueryDiscoveredProgramOutputProgress::Settled(result) => {
                    assert_eq!(result.root_outputs().count(), 2);
                    assert!(result
                        .root_outputs()
                        .all(|(_, root)| root.application_commit_receipt().is_none()));
                    settled = true;
                    break;
                }
                WorthQueryDiscoveredProgramOutputProgress::Pending => {
                    panic!("Stable reuse must not wait for effectful delivery")
                }
            }
        }
        (recovered, settled)
    });
    assert_eq!(
        recovery.published_rows.len() + recovery.omitted_publications,
        0
    );
    assert!(settled);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 4
    );
    assert_eq!(courts::native_head(&host), head);
    // Even with the original handle retained, completion closes the source.
    // This lawful negative distinguishes a receipt from active source custody.
    let closed = request
        .advanced()
        .recover_discovered_required_outputs::<program::Program, program::Tree>(
            &host,
            &receipt,
            WorthQueryOutputDemandControls::default(),
        );
    let Err(WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial)) = closed else {
        panic!("a completed source cannot be reopened by its receipt alone")
    };
    assert!(matches!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable
    ));
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
fn runnable_retained_source_recovers_without_producers_or_commits() {
    exercise(false, false)
}
#[test]
fn worker_only_runnable_retained_source_recovers_without_producers_or_commits() {
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
