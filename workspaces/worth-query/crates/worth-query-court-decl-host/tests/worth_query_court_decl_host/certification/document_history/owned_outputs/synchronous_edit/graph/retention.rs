//! Retention uses the original request's real budget and worker posture.
use super::*;
fn exercise(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<program::Program>(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
    let before = super::super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let guard = host
        .runtime()
        .probe_next_discovery_support_retention_memory_for_test()
        .unwrap();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .without_source()
        .idempotency(&"retained-discovery-memory".to_owned())
        .execute_performed::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        host.runtime()
            .probe_next_discovery_support_retention_memory_for_test()
            .is_err(),
        "the consumed registration remains guarded"
    );
    if worker {
        let WorthQueryApplicationPerformedMutationOutcome::Performed(_) = outcome else {
            panic!("unaccounted support retention must only poll")
        };
        assert_eq!(
            request
                .query(query::first_output::Read("first:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            30
        );
        assert_eq!(
            request
                .query(query::second_output::Read("second:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            1003
        );
        assert_eq!(courts::native_head(&host).1, [head.0]);
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + 2
        );
    } else {
        let WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
                cause,
            )),
        ) = outcome
        else {
            panic!("the actual memory request must refuse before effect")
        };
        let refusal = cause
            .coupled_producer_denial()
            .expect("actual support retention refusal");
        assert!(
            matches!(
                refusal.kind(),
                WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::ChargedBytesOverflow
                    )
                )
            ),
            "{refusal:?}"
        );
        assert!(
            refusal.subject().contains("owned-outputs.first-binding.v1"),
            "{refusal:?}"
        );
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + 1
        );
        assert_eq!(courts::native_head(&host), head);
        super::super::failure::assert_model_unchanged(&host, &before);
        assert!(request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .is_err());
    }
    drop(guard);
    drop(
        host.runtime()
            .probe_next_discovery_support_retention_memory_for_test()
            .unwrap(),
    );
}
#[test]
fn the_actual_discovery_retention_budget_refuses_without_calling_the_producer_unavailable() {
    exercise(false)
}
#[test]
fn worker_only_discovery_retention_never_requires_accounting() {
    exercise(true)
}
