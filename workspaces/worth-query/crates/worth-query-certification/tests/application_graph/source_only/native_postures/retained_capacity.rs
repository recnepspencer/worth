//! Retained delivery loses real finite World observation capacity before effect.
use super::*;
use std::time::Duration;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryProductBranchAdmissionDenial,
    WorthQueryProductBranchAdmissionRefusalKind,
};
use worth_query_host::facade::product::WorthQueryApplicationNoEffectCause;

#[test]
fn retained_native_no_effect_capacity_keeps_terminal_custody_until_fresh_close() {
    let host = host::install_program::<RetainedProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let original = operation(&host, &request, 19, 0x9176_6219);
    let pause = host
        .runtime()
        .pause_after_application_candidate_preparation_for_test(NonZeroUsize::MIN);
    let mut reads = Vec::new();
    let terminal = std::thread::scope(|threads| {
        let advancing = threads.spawn(|| {
            let fresh_scope = request_scope();
            let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
            original.advance(
                &host.runtime().request(&fresh_principal, &fresh_scope),
                bound(),
            )
        });
        assert!(pause.wait_until_reached(Duration::from_secs(10)));
        let mut exhausted = false;
        for _ in 0..=crate::document_retention_model::host::ACTIVE_WORLD_OBSERVATIONS {
            match request.retain_current_read() {
                Ok(read) => reads.push(read),
                Err(WorthQueryApplicationReadObservationDenial::ProductSelection(
                    WorthQueryProductBranchAdmissionDenial::Refused(
                        WorthQueryProductBranchAdmissionRefusalKind::ObservationCapacityExhausted,
                    ),
                )) => {
                    exhausted = true;
                    break;
                }
                Err(other) => panic!("ordinary pressure reached a different owner: {other:?}"),
            }
        }
        assert!(
            exhausted,
            "configured finite World observation owner refused"
        );
        pause.release();
        match advancing.join().unwrap() {
            WorthQuerySourceOnlyAdvance::Terminal(terminal) => terminal,
            _ => panic!("actual World no-effect remains terminal"),
        }
    });
    assert!(
        matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::NoEffect(no_effect))) if no_effect.cause() == WorthQueryApplicationNoEffectCause::CapacityExhausted)
    );
    drop(reads);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_eq!(read_retention(host.runtime(), host.current_world()), 7);
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(host.runtime().active_application_attempts_for_test(), 0);
    assert_eq!(
        host.runtime().world_active_publication_attempts_for_test(),
        0
    );
    assert!(host
        .runtime()
        .product_publication_recovery_page(None, bound())
        .unwrap()
        .rows()
        .is_empty());
    let stop = authentication_stop();
    let (terminal, cause) = terminal
        .close(&host.runtime().request(&principal, &stop))
        .err()
        .expect("stopped close preserves native evidence");
    assert!(matches!(
        cause,
        WorthQueryOperationCloseDenial::Execution(_)
    ));
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::NoEffect(_)
        ))
    ));
    drop(request);
    drop(principal);
    drop(scope);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    terminal
        .close(&host.runtime().request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, cause)| panic!("fresh no-effect close: {cause:?}"));
    assert_eq!(host.contacts.counts(), (1, 1));
}
fn authentication_stop(
) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope {
    let stop = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    stop.cancel();
    worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
        None,
        stop.token(),
    )
}
#[test]
fn retained_native_capacity_unpressured_twin_delivers_actual_commit() {
    let host = host::install_program::<RetainedProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let done = match operation(&host, &request, 19, 0x9176_6319).advance(&request, bound()) {
        WorthQuerySourceOnlyAdvance::Completed(done) => done,
        _ => panic!("real retained observation reservation succeeds"),
    };
    assert_eq!(read_retention(host.runtime(), host.current_world()), 19);
    assert_eq!(
        request
            .at(observation(done.evidence()))
            .query(ReadRetention {
                identity: DOCUMENT_IDENTITY.to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .retention_days,
        19
    );
    done.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("valid retained close: {cause:?}"));
    assert_eq!(host.contacts.counts(), (1, 1));
}
