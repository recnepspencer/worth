use super::*;
use std::time::Duration;

fn refused(edit: FullEdit<'_>, app: &App) {
    assert_eq!(
        edit.posture(),
        WorthQueryHistoryEditPosture::AwaitingSelection
    );
    assert!(edit.selected_step().is_none());
    assert_eq!(
        edit.completed_source().unwrap().settlement().output_count(),
        1
    );
    let document = with_request(app, |request| {
        edit.discard(request)
            .unwrap_or_else(|(_, cause)| panic!("original completed tree disposal: {cause:?}"))
    });
    assert_eq!(
        with_request(app, |request| document.read(request).unwrap().rows()[0]
            .value),
        length(1)
    );
    close(document, app);
}

#[test]
fn a_candidate_write_after_completion_cannot_replace_the_original_common_basis() {
    let world = install_full();
    let app = &world.application;
    let document = attach(app, "anchor-a");
    let edit = completed(begin_full(document, app, 2, 0x9182_e200), app);
    let original = edit.completed_source().unwrap();
    let c = original
        .settlement()
        .observation()
        .selected_commit()
        .clone();
    ordinary(app, original.branch(), 5, 0x9182_e201);
    let WorthQueryHistoryEditAdvance::Refused { edit, denial } =
        with_request(app, |request| edit.advance(request))
    else {
        panic!("later candidate head must not become this completed tree's endpoint")
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
        edit.completed_source()
            .unwrap()
            .settlement()
            .observation()
            .selected_commit(),
        &c
    );
    refused(edit, app);
}

#[test]
fn a_candidate_write_after_common_basis_capture_refuses_exact_adoption() {
    let world = install_full();
    let app = &world.application;
    let document = attach(app, "anchor-a");
    let mut edit = completed(begin_full(document, app, 2, 0x9182_e210), app);
    let branch = edit.completed_source().unwrap().branch();
    let pause = edit.pause_after_history_capture_for_test();
    let result = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let reached = pause.wait_until_reached(Duration::from_secs(10));
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                assert!(reached);
                ordinary(app, branch, 5, 0x9182_e211);
            }));
            pause.release();
            outcome
        });
        let result = with_request(app, |request| edit.advance(request));
        worker.join().unwrap().unwrap();
        result
    });
    let WorthQueryHistoryEditAdvance::Refused { edit, denial } = result else {
        panic!("exact captured C cannot adopt D")
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
    refused(edit, app);
}
