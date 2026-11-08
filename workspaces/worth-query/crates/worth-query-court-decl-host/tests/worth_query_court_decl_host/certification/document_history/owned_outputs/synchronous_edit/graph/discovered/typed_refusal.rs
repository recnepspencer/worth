//! Connection failures retain their actual typed owner cause through whole-edit refusal.
use super::*;
fn exercise(worker: bool, value: u64, connection: &str, detail: &str, prepared: u64) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_other_model::<program::Program>(limits, 2, Some(3));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value,
        })
        .without_source()
        .idempotency(&"typed-discovery-refusal".to_owned())
        .execute_performed_discovered::<program::Program, program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            cause,
        )),
    ) = outcome
    else {
        panic!("the typed connection failure must refuse the complete edit")
    };
    let refusal = cause
        .coupled_producer_denial()
        .expect("the original connection failure");
    assert_eq!(refusal.subject(), connection);
    assert_eq!(
        refusal.discovery_connection_denial(),
        Some(&WorthQueryRequiredOutputConnectionDenial::new(detail))
    );
    assert_eq!(
        refusal.kind(),
        WorthQueryOutputDemandDenialKind::DiscoveryConnection(Box::new(
            WorthQueryRequiredOutputConnectionDenial::new(detail)
        ))
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + prepared
    );
    super::super::super::failure::assert_model_unchanged(&host, &before);
}
#[test]
fn root_discovery_keeps_its_typed_connection_failure() {
    exercise(
        false,
        13,
        "owned-outputs.discovered-edit.start",
        "root choice thirteen is unsupported",
        0,
    )
}
#[test]
fn worker_only_root_discovery_keeps_its_typed_connection_failure() {
    exercise(
        true,
        13,
        "owned-outputs.discovered-edit.start",
        "root choice thirteen is unsupported",
        0,
    )
}
#[test]
fn child_discovery_keeps_its_typed_connection_failure() {
    exercise(
        false,
        11,
        "owned-outputs.discovered-edit.child",
        "child choice one hundred ten is unsupported",
        2,
    )
}
#[test]
fn worker_only_child_discovery_keeps_its_typed_connection_failure() {
    exercise(
        true,
        11,
        "owned-outputs.discovered-edit.child",
        "child choice one hundred ten is unsupported",
        2,
    )
}
