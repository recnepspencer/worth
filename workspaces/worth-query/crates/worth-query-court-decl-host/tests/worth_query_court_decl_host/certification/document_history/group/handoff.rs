//! Stops after real owner transfer preserve reservation, fork and selected completion custody.
use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::WorthQueryHistoryGroupHandoffPointForTest as Point,
};

fn cancelled<T>(result: Result<T, WorthQueryHistoryGroupDenial>) {
    let Err(cause) = result else {
        panic!("stopped owner handoff cannot report ordinary success")
    };
    assert!(
        matches!(
            cause,
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::Stopped(WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                ))
            )) | WorthQueryHistoryGroupDenial::History(
                WorthQueryApplicationHistoryDenial::ExecutionRequest(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    )
                )
            )
        ),
        "{cause:?}"
    );
}

#[test]
fn stopped_reservation_and_candidate_handoffs_keep_the_original_group_abortable() {
    for point in [Point::Reservation, Point::Candidate] {
        let host = publish_on_first_program();
        let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let original = document.selection(&request).unwrap();
        let mut group = begin(&host, document, 5, 0x9182_5700);
        let posture = if point == Point::Reservation {
            Posture::Admitting
        } else {
            Posture::CreatingCandidate
        };
        until(&host, &mut group, |group| group.posture() == posture);
        let pause = {
            let (_, execution) = group.parts();
            execution.pause_on_group_handoff_for_test(point)
        };
        let cancellation = WorthQueryCancellationSource::new();
        let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
        let interrupted_principal = authenticate_operator(host.installed_schema(), &stopped);
        let (reached, result) = std::thread::scope(|threads| {
            let (document, execution) = group.parts();
            let worker = threads.spawn(|| {
                execution.advance(document, &host.request(&interrupted_principal, &stopped))
            });
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            if reached {
                cancellation.cancel();
            }
            pause.release();
            (reached, worker.join().unwrap())
        });
        assert!(
            reached,
            "actual group owner handoff must occur before cancellation"
        );
        cancelled(result);
        assert_eq!(
            group.posture(),
            if point == Point::Reservation {
                Posture::CreatingCandidate
            } else {
                Posture::Running
            }
        );
        assert_eq!(group.document().selection(&request).unwrap(), original);
        let document = group.abort(&request).unwrap();
        assert_eq!(document.selection(&request).unwrap(), original);
        let mut lawful = begin(&host, document, 5, 0x9182_5700);
        until(&host, &mut lawful, |group| {
            group.posture() == Posture::ReadyToComplete
        });
        {
            let (document, execution) = lawful.parts();
            execution.complete(document, &request).unwrap();
        }
        let finished = lawful.finish(&request).unwrap();
        drop(finished.completions);
        close(&host, finished.document, 2);
    }
}

#[test]
fn delivery_stop_after_group_adoption_retains_the_selected_result_for_its_original_retry() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut group = begin(&host, document, 5, 0x9182_5800);
    until(&host, &mut group, |group| {
        group.posture() == Posture::ReadyToComplete
    });
    let pause = {
        let (_, execution) = group.parts();
        execution.pause_on_group_handoff_for_test(Point::Adoption)
    };
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let interrupted_principal = authenticate_operator(host.installed_schema(), &stopped);
    let (reached, result) = std::thread::scope(|threads| {
        let (document, execution) = group.parts();
        let worker = threads.spawn(|| {
            execution.complete(document, &host.request(&interrupted_principal, &stopped))
        });
        let reached = pause.wait_until_reached(Duration::from_secs(10));
        if reached {
            cancellation.cancel();
        }
        pause.release();
        (reached, worker.join().unwrap())
    });
    assert!(
        reached,
        "selection must actually complete before delivery stops"
    );
    cancelled(result);
    assert_eq!(group.posture(), Posture::Selected);
    assert_eq!(value(&host, group.document()), 5);
    let original = group.document().selection(&request).unwrap();
    {
        let (document, execution) = group.parts();
        assert_eq!(
            execution.complete(document, &request).unwrap(),
            *original.step()
        );
    }
    let finished = group.finish(&request).unwrap();
    assert_eq!(finished.document.selection(&request).unwrap(), original);
    drop(finished.completions);
    close(&host, finished.document, 2);
}
