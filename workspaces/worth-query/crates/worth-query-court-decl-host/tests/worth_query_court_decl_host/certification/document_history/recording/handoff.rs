use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{WorthQueryApplicationHistoryDenial, WorthQueryHistoryEditDenial},
};

#[test]
fn delivery_loss_after_adoption_retains_selected_head_and_original_completion_owner() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let mut edit = begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_0420);
    let handoff = edit.pause_after_history_adoption_for_test();
    for _ in 0..3 {
        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let Advance::Progress(next) = edit.advance(&host.runtime().request(&principal, &scope))
        else {
            panic!("private operation progression")
        };
        edit = next;
    }
    let cancellation = WorthQueryCancellationSource::new();
    let stopped_scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let principal = authenticate_operator(host.runtime().installed_schema(), &stopped_scope);
    let (reached, result) = std::thread::scope(|threads| {
        let worker =
            threads.spawn(|| edit.advance(&host.runtime().request(&principal, &stopped_scope)));
        let reached = handoff.wait_until_reached(Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        handoff.release();
        (reached, worker.join())
    });
    assert!(reached, "actual adoption did not reach its handoff");
    let result = result.unwrap();
    let Advance::Refused { edit, denial } = result else {
        panic!("checked handoff reports delivery loss")
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(
                WorthQueryApplicationHistoryDenial::Document(
                  worth_query_host::facade::application_entry::WorthQueryDocumentHistoryDenial::Stopped(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    )
                  )
                )
            )
        ),
        "{denial:?}"
    );
    assert_eq!(edit.posture(), Posture::Selected);
    assert!(edit.completed_source().is_some());
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    assert_eq!(
        edit.read(&host.runtime().request(&principal, &scope))
            .unwrap()
            .rows()[0]
            .retention_days,
        3
    );
    let expected = edit.selected_step().unwrap().id();
    let (document, step) = finish(&host, edit, 3);
    assert_eq!(step.id(), expected);
    close(&host, document, 2);
}
