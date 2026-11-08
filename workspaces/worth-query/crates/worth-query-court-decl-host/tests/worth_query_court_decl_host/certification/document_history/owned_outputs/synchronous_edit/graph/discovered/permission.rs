//! D can change grants; the discovery query is independently authorized only at C.
use super::*;
#[path = "permission_program.rs"]
mod permission_program;
fn exercise(worker: bool, allowed: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<permission_program::Program>(
        limits,
        if allowed { 1 } else { 2 },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let observation = source_projection::Observation::begin();
    let value = if allowed { 7 } else { 1 };
    let outcome = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value,
        })
        .without_source()
        .idempotency(&"root-permission".to_owned())
        .execute_performed_discovered::<permission_program::Program, permission_program::Tree>(
            &host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    if allowed {
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationDiscoveredMutationOutcome::Performed(_)
            ),
            "permission proved at C survives this edit removing its grant"
        );
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts + 1
        );
        assert_eq!(observation.protected_member_reads(), 1);
        assert_eq!(
            request
                .query(query::first_output::Read("first:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            70
        );
    } else {
        let WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
                cause,
            )),
        ) = outcome
        else {
            panic!("a D grant cannot authorize root discovery at C")
        };
        let refusal = cause
            .coupled_producer_denial()
            .expect("the discovery query's original refusal");
        assert_eq!(
            refusal.kind(),
            WorthQueryOutputDemandDenialKind::RequestAuthorization(
                WorthQueryOperationAuthorizationRefusalKind::PermissionDenied
            )
        );
        assert!(
            refusal
                .subject()
                .contains("owned-outputs.member-binding.v1"),
            "{refusal:?}"
        );
        assert!(!refusal.subject().contains("EntityId"));
        assert_eq!(
            observation.protected_member_reads(),
            0,
            "unauthorized data is never projected"
        );
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
        super::super::super::failure::assert_model_unchanged(&host, &before);
    }
}
#[test]
fn root_discovery_does_not_borrow_a_d_grant() {
    exercise(false, false)
}
#[test]
fn worker_only_root_discovery_does_not_borrow_a_d_grant() {
    exercise(true, false)
}
#[test]
fn root_discovery_keeps_the_current_c_grant_removed_by_its_own_edit() {
    exercise(false, true)
}
#[test]
fn worker_only_root_discovery_keeps_the_current_c_grant_removed_by_its_own_edit() {
    exercise(true, true)
}
