use super::*;
use std::time::Duration;

#[test]
fn original_document_source_changes_after_capture_refuse_adoption_and_preserve_completion() {
    let world = install();
    let application = &world.application;
    warm(application, "anchor-a");
    let (document, branch) = attach_with_branch(application, "anchor-a");
    let mut edit = to_selection(
        begin(document, application, "anchor-a", 2, 0x9182_d300),
        application,
    );
    let output = edit
        .completed_source()
        .unwrap()
        .settlement()
        .performed_source()
        .unwrap()
        .root_fulfillment()
        .output_commit()
        .clone();
    let pause = edit.pause_after_history_capture_for_test();
    let (reached, changed, result) = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            let changed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                assert!(reached, "actual qualified-source capture rendezvous");
                let mut operation = with_branch_request(application, branch, |request| {
                    request
                        .mutate(PlanarInitialAdjustment {
                            scope_key: "anchor-a".into(),
                            replacement_y: length(5),
                        })
                        .without_source()
                        .idempotency_owned(0x9182_d301)
                        .unwrap_or_else(|(_, _, cause)| panic!("outside capture: {cause:?}"))
                        .retain_operation::<HistoryProgram, HistoryAction>(application)
                        .unwrap_or_else(|(_, cause)| panic!("outside construction: {cause:?}"))
                });
                for _ in 0..64 {
                    match with_branch_request(application, branch, |request| {
                        operation.advance(request)
                    }) {
                        WorthQueryOperationAdvance::Progress {
                            operation: next, ..
                        } => operation = next,
                        WorthQueryOperationAdvance::Settled(settled) => {
                            let closed = with_branch_request(application, branch, |request| {
                                settled
                                    .close_for_delivery(request)
                                    .unwrap_or_else(|(_, cause)| panic!("outside close: {cause:?}"))
                            });
                            drop(closed);
                            return;
                        }
                        _ => panic!("ordinary outside Fixed operation must settle"),
                    }
                }
                panic!("outside leaf did not settle")
            }));
            pause.release();
            (reached, changed)
        });
        let result = with_request(application, |request| edit.advance(request));
        let (reached, changed) = worker.join().unwrap();
        (reached, changed, result)
    });
    assert!(reached);
    changed.unwrap();
    let WorthQueryHistoryEditAdvance::Refused { edit, denial } = result else {
        panic!("captured B cannot adopt over a changed original document")
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::Adoption(_)
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(
        edit.posture(),
        WorthQueryHistoryEditPosture::AwaitingSelection
    );
    assert!(edit.selected_step().is_none());
    assert_eq!(
        edit.completed_source()
            .unwrap()
            .settlement()
            .performed_source()
            .unwrap()
            .root_fulfillment()
            .output_commit(),
        &output
    );
    let document = with_request(application, |request| {
        edit.discard(request)
            .unwrap_or_else(|(_, cause)| panic!("captured candidate disposal: {cause:?}"))
    });
    assert_eq!(
        with_branch_request(application, branch, |request| request
            .query(PlanarRead {
                body_key: "anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y),
        length(5)
    );
    close(document, application);
}
