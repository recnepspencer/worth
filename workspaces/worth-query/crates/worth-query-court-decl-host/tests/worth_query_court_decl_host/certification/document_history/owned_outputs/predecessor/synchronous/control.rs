//! Current admission expiring inside a coupled old-input loan vetoes the edit.
use super::super::{courts::reset, courts::settle, PROBE};
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionTimeController, WorthQueryCancellationSource,
    },
    runtime::ExecutionAllocationPolicy,
};
#[test]
fn coupled_authentication_expiring_during_predecessor_read_has_no_effect() {
    let controller = std::sync::Arc::new(WorthQueryAdmissionTimeController::new().unwrap());
    controller.hold();
    let clock = controller.clock();
    let host =
        world::authorization_clock::install::<predecessor::program::Program>(clock.clone(), false, 1);
    reset(false);
    let scope =
        WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token(), clock);
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&request, &host).unwrap();
    let checkpoint = synchronous_edit::failure::memory_basis_state(&host);
    let before = host.current_world();
    PROBE.with(|probe| {
        probe.borrow_mut().after_read = Some(Box::new(move || {
            controller
                .advance(std::time::Duration::from_secs(7200))
                .unwrap();
        }))
    });
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"expired-coupled-read".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!(
            "expired C authentication cannot publish: receipt {:?}",
            outcome.receipt().map(|receipt| receipt.commit_reference())
        );
    };
    let producer = denial
        .coupled_producer_denial()
        .expect("the true coupled read refusal");
    assert_eq!(
        producer.kind(),
        WorthQueryOutputDemandDenialKind::RequestAuthorization(
            WorthQueryOperationAuthorizationRefusalKind::ExpiredAuthentication,
        )
    );
    PROBE.with(|probe| {
        assert!(
            matches!(probe.borrow().values.last(), Some(Err(_))),
            "the value cannot escape an expired current authorization"
        )
    });
    assert_eq!(host.current_world(), before);
    synchronous_edit::failure::assert_memory_basis_unchanged(&host, &checkpoint);
}
