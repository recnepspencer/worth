//! Real retained readers fill the installed release queue before edit effects.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryHistoryEditAdvance as Advance, WorthQueryHistoryEditDenial,
    WorthQueryHistoryEditPosture as Posture,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::ReleaseBoundedRetentionHistory as History,
    presented_request::set_retention,
    readback::observe_head,
    retention_entry::SetRetentionHandler,
    settled_verdict::{settle, RetentionVerdict},
};

fn attach_bounded(host: &Runtime) -> Document<'_, History> {
    let head = head(host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.with_host_advancement(|phase| {
        host.request(&principal, &scope)
            .on_branch(head.branch())
            .attach_history::<History>(
                &phase,
                head,
                DocumentRetentionRead {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                },
            )
            .unwrap()
    })
    .unwrap()
}

#[test]
fn a_full_release_queue_refuses_edit_enrollment_until_actual_cleanup_frees_capacity() {
    let (host, contacts) =
        SetRetentionHandler::observe_installation_for_test(publish_on_first_program);
    let mut document = attach_bounded(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let mut first = None;
    for (offset, days) in [3, 5, 6].into_iter().enumerate() {
        let (next, step) = finish(
            &host,
            ready(
                &host,
                begin(
                    &host,
                    document,
                    DOCUMENT_IDENTITY,
                    days,
                    0x9182_6800 + offset as u64,
                ),
            ),
            days,
        );
        document = next;
        first.get_or_insert(step);
    }
    let first = first.unwrap();
    let root_branch = root.endpoint().product_branch();
    let first_branch = first.endpoint().product_branch();
    assert_eq!(
        settle(set_retention(&host, root_branch, 9, 0x9182_6810)),
        RetentionVerdict::Performed(9)
    );
    let root_reader = observe_head(host.runtime(), root_branch);
    assert_eq!(
        settle(set_retention(&host, first_branch, 8, 0x9182_6811)),
        RetentionVerdict::Performed(8)
    );
    let first_reader = observe_head(host.runtime(), first_branch);

    let target = ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN);
    let one = document
        .set_history_window(target)
        .advance::<Window>(&request)
        .unwrap();
    assert_eq!(
        (one.pending_window, one.pending_releases),
        (Some(target), 1)
    );
    let two = document
        .set_history_window(target)
        .advance::<Window>(&request)
        .unwrap();
    assert_eq!(
        (
            two.pending_window,
            two.pending_releases,
            two.unprotected_steps
        ),
        (None, 2, 1)
    );
    let before = document.selection(&request).unwrap();
    let calls = contacts.snapshot();
    let Advance::Refused { edit, denial } =
        begin(&host, document, DOCUMENT_IDENTITY, 4, 0x9182_6820).advance(&request)
    else {
        panic!("both actual cleanup owners occupy the installed release bound");
    };
    assert!(
        matches!(
            denial,
            WorthQueryHistoryEditDenial::History(Failure::Document(
                Native::RetentionReleaseCapacityExhausted
            ))
        ),
        "{denial:?}"
    );
    assert_eq!(edit.posture(), Posture::AwaitingAdmission);
    assert_eq!(edit.read(&request).unwrap().rows()[0].retention_days, 6);
    assert_eq!(
        contacts.snapshot(),
        calls,
        "neither decision nor candidate handler ran"
    );
    // The edit owns the original document; discard that unselected attempt before
    // borrowing its same document for maintenance. Reader Drop alone is no receipt.
    let mut document = edit.discard(&request).unwrap();
    assert_eq!(document.selection(&request).unwrap(), before);
    drop(root_reader);
    assert_eq!(
        document
            .history_retention_status(&request)
            .unwrap()
            .pending_releases,
        2
    );
    let mut closed = false;
    for _ in 0..4 {
        if document.maintain_history(&request).unwrap() == Release::Closed {
            closed = true;
            break;
        }
    }
    assert!(closed, "the original queued head must finish cleanup");
    assert_eq!(
        document
            .history_retention_status(&request)
            .unwrap()
            .pending_releases,
        1
    );
    let (document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 4, 0x9182_6820),
        ),
        4,
    );
    let after = contacts.snapshot();
    assert_eq!(after, (calls.0 + 1, calls.1 + 1));
    drop(first_reader);
    close(&host, document, 5);
}
