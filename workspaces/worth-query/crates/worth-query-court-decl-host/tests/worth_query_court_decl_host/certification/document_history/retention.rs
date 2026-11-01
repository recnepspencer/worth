//! Real public owners prove document-wide eligibility and independent holder rights.
use super::recording::{begin, close, finish, ready};
use super::*;
mod custody;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistoryDenial as Failure, WorthQueryDocumentHistoryDenial as Native,
    WorthQueryHistoryNodeId, WorthQueryHistoryPageLimits,
    WorthQueryHistoryParentBoundary as Parent, WorthQueryHistoryReleaseOutcome as Release,
};
use worth_query_host::facade::declaration::application_history::ApplicationHistoryRecentSteps;
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::{
    navigation::Navigation,
    retention_policy::{Bookmark, Window},
};

fn limits() -> WorthQueryHistoryPageLimits {
    WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::new(32).unwrap(),
        maximum_examined_entries: NonZeroUsize::new(32).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    }
}
fn window(host: &Runtime, document: &mut Document<'_>, count: usize) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut change = document.set_history_window(ApplicationHistoryRecentSteps::new(
        NonZeroUsize::new(count).unwrap(),
    ));
    for _ in 0..64 {
        let status = change.advance::<Window>(&request).unwrap();
        if status.pending_window.is_none() {
            assert_eq!(status.recent_steps.get(), count);
            return;
        }
    }
    panic!("finite frontier failed to complete");
}
fn checkout(host: &Runtime, document: &Document<'_>, target: WorthQueryHistoryNodeId, key: u128) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let owner = document
        .navigate::<Navigation>()
        .checkout(target)
        .expected_selection(document.selection(&request).unwrap())
        .idempotency(key)
        .prepare(&request)
        .unwrap();
    navigation::complete(host, owner);
}

#[test]
fn a_creation_order_window_spans_futures_and_preserves_only_current_bookmarks_and_existing_holders()
{
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    assert_eq!(
        document
            .history_retention_status(&request)
            .unwrap()
            .recent_steps
            .get(),
        200
    );
    window(&host, &mut document, 3);
    let root = document.current_step(&request).unwrap();
    document
        .set_history_bookmark::<Bookmark>(&request, root.id(), true)
        .unwrap();
    let mut issued = vec![root.id()];
    let mut old_preview = None;
    for (offset, days) in [3, 7, 5, 4, 6, 2].into_iter().enumerate() {
        if offset == 4 {
            checkout(&host, &document, issued[2], 0x9182_6100);
        }
        let (next, step) = finish(
            &host,
            ready(
                &host,
                begin(
                    &host,
                    document,
                    DOCUMENT_IDENTITY,
                    days,
                    0x9182_6000 + offset as u64,
                ),
            ),
            days,
        );
        document = next;
        issued.push(step.id());
        if offset == 0 {
            old_preview = Some(document.preview(&request, step.id()).unwrap());
        }
        // Independent creation-order reference: one protected bookmark, the
        // current endpoint, and the newest three remaining authored occurrences.
        let mut expected = issued
            .iter()
            .copied()
            .filter(|id| *id != root.id() && *id != step.id())
            .rev()
            .take(3)
            .collect::<Vec<_>>();
        expected.extend([root.id(), step.id()]);
        expected.sort();
        let page = document.history_page(&request, limits(), None).unwrap();
        let actual = page
            .rows()
            .iter()
            .map(|row| row.step().id())
            .collect::<Vec<_>>();
        assert_eq!(
            actual, expected,
            "one document-wide budget across both futures"
        );
    }
    window(&host, &mut document, 1);
    let page = document.history_page(&request, limits(), None).unwrap();
    let expected = [root.id(), issued[5], issued[6]];
    assert_eq!(
        page.rows()
            .iter()
            .map(|row| row.step().id())
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(page.rows()[1].step().parent(), Parent::Truncated);
    assert_eq!(page.rows()[2].step().parent(), Parent::Retained(issued[5]));
    let status = document.history_retention_status(&request).unwrap();
    assert_eq!(
        (status.unprotected_steps, status.bookmarks, status.current),
        (1, 1, issued[6])
    );
    assert!(matches!(
        document.preview(&request, issued[1]),
        Err(Failure::Document(Native::HistoryNodeExcluded))
    ));
    assert!(matches!(
        document.set_history_bookmark::<Bookmark>(&request, issued[1], true),
        Err(Failure::Document(Native::HistoryNodeExcluded))
    ));
    let preview = old_preview.take().unwrap();
    let original_branch = preview.step().endpoint().product_branch();
    for _ in 0..32 {
        if document
            .history_retention_status(&request)
            .unwrap()
            .pending_releases
            == 0
        {
            break;
        }
        document.maintain_history(&request).unwrap();
    }
    host.with_host_advancement(|phase| {
        use worth_query_host::facade::product::{
            WorthQueryProductBranchAdmissionDenial as Admission,
            WorthQueryProductBranchAdmissionRefusalKind as Refusal,
        };
        assert!(matches!(
            host.on_branch(original_branch).select(&phase),
            Err(Admission::Refused(Refusal::RetiredBranch))
        ));
    })
    .unwrap();
    assert_eq!(value(&host, &document), 2);
    assert_eq!(
        preview.read(&document, &request).unwrap().rows()[0].retention_days,
        3
    );
    let sibling = document.preview(&request, issued[5]).unwrap();
    document
        .set_history_bookmark::<Bookmark>(&request, issued[5], true)
        .unwrap();
    assert!(matches!(
        document.set_history_bookmark::<Bookmark>(&request, issued[6], true),
        Err(Failure::Document(Native::BookmarkCapacityExhausted))
    ));
    assert_eq!(
        document
            .history_retention_status(&request)
            .unwrap()
            .bookmarks,
        2
    );
    document
        .set_history_bookmark::<Bookmark>(&request, issued[5], false)
        .unwrap();
    preview.close();
    document
        .set_history_bookmark::<Bookmark>(&request, root.id(), false)
        .unwrap();
    assert!(matches!(
        document.preview(&request, root.id()),
        Err(Failure::Document(Native::HistoryNodeExcluded))
    ));
    assert_eq!(
        sibling.read(&document, &request).unwrap().rows()[0].retention_days,
        6
    );
    sibling.close();
    assert_eq!(value(&host, &untouched), SEED_RETENTION);
    close(&host, document, 8); // Seven original heads and one real checkout head.
    close(&host, untouched, 1);
}

#[test]
fn an_original_unfinished_checkout_survives_exclusion_and_restores_a_live_current_source() {
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut target = None;
    for (offset, days) in [3, 7, 5].into_iter().enumerate() {
        let (next, step) = finish(
            &host,
            ready(
                &host,
                begin(
                    &host,
                    document,
                    DOCUMENT_IDENTITY,
                    days,
                    0x9182_6200 + offset as u64,
                ),
            ),
            days,
        );
        document = next;
        target.get_or_insert(step.id());
    }
    let target = target.unwrap();
    let expected = document.selection(&request).unwrap();
    let original = document
        .navigate::<Navigation>()
        .checkout(target)
        .expected_selection(expected.clone())
        .idempotency(0x9182_6210)
        .prepare(&request)
        .unwrap();
    drop(original); // The actual registered Native attempt retains original H.
    window(&host, &mut document, 1);
    assert!(matches!(
        document.preview(&request, target),
        Err(Failure::Document(Native::HistoryNodeExcluded))
    ));
    let mut deferred = false;
    for _ in 0..8 {
        deferred |= document.maintain_history(&request).unwrap() == Release::WaitingForNavigation;
    }
    assert!(
        deferred,
        "actual original H must remain live through unfinished selection"
    );
    let original = document
        .navigate::<Navigation>()
        .checkout(target)
        .expected_selection(expected)
        .idempotency(0x9182_6210)
        .prepare(&request)
        .unwrap();
    navigation::complete(&host, original);
    assert_eq!(value(&host, &document), 3);
    assert_eq!(document.current_step(&request).unwrap().id(), target);
    // A second, new Native navigation must fork the restored canonical H,
    // not merely read through the previous attempt's immutable observation.
    checkout(&host, &document, target, 0x9182_6211);
    assert_eq!(value(&host, &document), 3);
    close(&host, document, 6);
}

#[test]
fn each_resumed_exclusion_batch_invalidates_older_cursors_without_changing_the_current_endpoint() {
    use worth_query_host::facade::application_entry::WorthQueryHistoryPageDenial;
    let host = publish_on_first_program();
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    for (offset, days) in [3, 7, 5, 4, 6].into_iter().enumerate() {
        let (next, _) = finish(
            &host,
            ready(
                &host,
                begin(
                    &host,
                    document,
                    DOCUMENT_IDENTITY,
                    days,
                    0x9182_6300 + offset as u64,
                ),
            ),
            days,
        );
        document = next;
    }
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = document.selection(&request).unwrap();
    let target = ApplicationHistoryRecentSteps::new(NonZeroUsize::MIN);
    let first = document
        .set_history_window(target)
        .advance::<Window>(&request)
        .unwrap();
    assert_eq!(first.pending_window, Some(target));
    let narrow = WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::MIN,
        ..limits()
    };
    let cursor = document
        .history_page(&request, narrow, None)
        .unwrap()
        .continuation()
        .unwrap();
    let second = document
        .set_history_window(target)
        .advance::<Window>(&request)
        .unwrap();
    assert_eq!(second.unprotected_steps + 1, first.unprotected_steps);
    assert_eq!(document.selection(&request).unwrap(), before);
    assert!(matches!(
        document.history_page(&request, narrow, Some(&cursor)),
        Err(Failure::Document(Native::Paging(
            WorthQueryHistoryPageDenial::StaleCursor
        )))
    ));
    window(&host, &mut document, 1);
    close(&host, document, 6);
}
