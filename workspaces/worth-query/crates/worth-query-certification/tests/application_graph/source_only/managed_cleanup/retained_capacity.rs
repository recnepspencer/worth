//! Retained delivery loses real finite World observation capacity before effect.
use super::*;
use crate::source_only::{
    host,
    retained::{operation, RetainedProgram},
};
use std::time::Duration;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryProductBranchAdmissionDenial,
    WorthQueryProductBranchAdmissionRefusalKind,
};
use worth_query_host::facade::product::WorthQueryApplicationNoEffectCause;

#[test]
fn managed_no_effect_capacity_retains_actual_cleanup_failure() {
    let host = host::install_program::<RetainedProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let (terminal, _pause, _cleanup) = {
        let request = host.runtime().request(&principal, &scope);
        let before = request
            .retain_current_read()
            .unwrap()
            .selected_commit()
            .clone();
        let original = operation(&host, &request, 19, 0x9176_6219);
        let _cleanup = host
            .runtime()
            .schedule_next_managed_cleanup_on_worker_for_test()
            .unwrap();
        let pause = host
            .runtime()
            .pause_after_application_candidate_preparation_for_test(NonZeroUsize::MIN);
        let (release, released) = std::sync::mpsc::channel();
        let terminal = std::thread::scope(|threads| {
            let host = &host;
            let pause = &pause;
            let pressure = threads.spawn(move || {
                let fresh_scope = request_scope();
                let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
                let request = host.runtime().request(&fresh_principal, &fresh_scope);
                let mut reads = Vec::new();
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
                released.recv().unwrap();
                drop(reads);
            });
            let terminal = match original.advance(&request, None) {
                WorthQuerySourceOnlyAdvance::Terminal(terminal) => terminal,
                _ => panic!("actual World no-effect retains real managed failure"),
            };
            release.send(()).unwrap();
            pressure.join().unwrap();
            terminal
        });
        assert_native_failure(terminal.managed_cleanup_failure().unwrap());
        assert!(
            matches!(terminal.native_outcome(), Some(WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::NoEffect(no_effect))) if no_effect.cause() == WorthQueryApplicationNoEffectCause::CapacityExhausted)
        );
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
            .product_publication_recovery_page(None, NonZeroUsize::MIN)
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
        (terminal, pause, _cleanup)
    };
    drop(principal);
    drop(scope);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let closed = terminal
        .close(&host.runtime().request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, cause)| panic!("fresh no-effect close: {cause:?}"));
    assert!(closed
        .managed_cleanup_receipt()
        .unwrap()
        .inspection()
        .resources_released());

    assert_eq!(host.contacts.counts(), (1, 1));
}
pub(super) fn authentication_stop(
) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope {
    let stop = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    stop.cancel();
    worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
        None,
        stop.token(),
    )
}
