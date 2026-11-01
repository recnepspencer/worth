//! Explicit redo choices come from admitted completed child links, not UI history caches.
use super::*;
use super::recording::{begin, close, finish, ready};
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryHistoryRedoChoice as Choice, WorthQueryHistoryPageLimits as Limits,
    WorthQueryHistoryEditAdvance as EditAdvance, WorthQueryHistoryEditPosture,
    WorthQueryApplicationHistoryDenial as Failure, WorthQueryDocumentHistoryDenial as Native,
    WorthQueryHistoryPageDenial as PageFailure, WorthQueryHistoryNodeId,
    WorthQueryHistoryNavigationAdvanceDenial, WorthQueryHistoryNavigationDenial,
};
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::navigation::Navigation;
mod authority;
mod handoff;

fn limits(width: usize) -> Limits {
    Limits {
        maximum_entries: NonZeroUsize::new(width).unwrap(),
        maximum_examined_entries: NonZeroUsize::new(width).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    }
}

fn checkout(host: &Runtime, document: &Document<'_>, node: WorthQueryHistoryNodeId, key: u128) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let owner = document
        .navigate::<Navigation>()
        .checkout(node)
        .expected_selection(document.selection(&request).unwrap())
        .idempotency(key)
        .prepare(&request)
        .unwrap();
    super::navigation::complete(host, owner);
}

fn forked(
    host: &App,
) -> (
    Document<'_>,
    [worth_query_host::facade::application_entry::WorthQueryHistoryStep; 2],
) {
    let document = attach(host, head(host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let root = document
        .current_step(&host.request(&principal, &scope))
        .unwrap();
    let (document, first) = finish(
        host,
        ready(
            host,
            begin(host, document, DOCUMENT_IDENTITY, 3, 0x9182_f200),
        ),
        3,
    );
    checkout(host, &document, root.id(), 0x9182_f201);
    let (document, second) = finish(
        host,
        ready(
            host,
            begin(host, document, DOCUMENT_IDENTITY, 5, 0x9182_f202),
        ),
        5,
    );
    checkout(host, &document, root.id(), 0x9182_f203);
    (document, [first, second])
}

fn page_failure(cause: Failure) -> PageFailure {
    let Failure::Document(Native::Paging(cause)) = cause else {
        panic!("page failure: {cause:?}")
    };
    cause
}

#[test]
fn completed_alternatives_require_explicit_choice_and_cancelled_reservations_stay_hidden() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.selection(&request).unwrap();
    let empty = document.redo_choices(&request, limits(1), None).unwrap();
    assert_eq!(empty.choice(), Choice::NoChild);
    assert!(empty.children().rows().is_empty());
    assert!(empty.children().continuation().is_none());
    let (document, first) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_f210),
        ),
        3,
    );
    checkout(&host, &document, root.step().id(), 0x9182_f211);
    let EditAdvance::Progress(edit) =
        begin(&host, document, DOCUMENT_IDENTITY, 9, 0x9182_f212).advance(&request)
    else {
        panic!("real history reservation")
    };
    assert_eq!(
        edit.posture(),
        WorthQueryHistoryEditPosture::CreatingCandidate
    );
    let document = edit.discard(&request).unwrap();
    let single = document.redo_choices(&request, limits(1), None).unwrap();
    assert_eq!(single.choice(), Choice::Single(first.id()));
    assert_eq!(single.children().rows()[0].step(), &first);
    assert!(single.children().continuation().is_none());
    let remembered = document.selection(&request).unwrap();
    let (document, second) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 5, 0x9182_f213),
        ),
        5,
    );
    checkout(&host, &document, root.step().id(), 0x9182_f214);
    let before = document.selection(&request).unwrap();
    for width in [1, 2] {
        let mut cursor = None;
        let mut rows = Vec::new();
        loop {
            let choices = document
                .redo_choices(&request, limits(width), cursor.as_ref())
                .unwrap();
            assert_eq!(choices.choice(), Choice::ChoiceRequired);
            assert_eq!(choices.children().current_step_id(), root.step().id());
            assert!(choices.children().examined_entries() <= width);
            assert!(
                choices.children().admitted_backing_bytes()
                    <= limits(width).maximum_owned_bytes.get()
            );
            rows.extend(
                choices
                    .children()
                    .rows()
                    .iter()
                    .map(|row| row.step().clone()),
            );
            cursor = choices.children().continuation();
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(rows, [first.clone(), second.clone()]);
    }
    assert_eq!(document.selection(&request).unwrap(), before);
    assert_eq!(value(&host, &document), SEED_RETENTION);
    let stale = document
        .navigate::<Navigation>()
        .redo_to(first.id())
        .expected_selection(remembered)
        .idempotency(0x9182_f215)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(
            stale,
            WorthQueryHistoryNavigationAdvanceDenial::Navigation(
                WorthQueryHistoryNavigationDenial::HeadChanged
            )
        ),
        "{stale:?}"
    );
    let owner = document
        .navigate::<Navigation>()
        .redo_to(first.id())
        .expected_selection(before)
        .idempotency(0x9182_f215)
        .prepare(&request)
        .unwrap();
    let selected = super::navigation::complete(&host, owner);
    assert_eq!(selected.selection().step(), &first);
    assert_eq!(value(&host, &document), 3);
    close(&host, document, 6);
}

#[test]
fn child_cursors_cannot_change_listing_scope_document_or_selected_parent() {
    let host = publish_on_first_program();
    let (document, children) = forked(&host);
    let other = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let all_cursor = document
        .history_page(&request, limits(1), None)
        .unwrap()
        .continuation()
        .unwrap();
    let child_cursor = document
        .redo_choices(&request, limits(1), None)
        .unwrap()
        .children()
        .continuation()
        .unwrap();
    assert!(matches!(
        page_failure(
            document
                .history_page(&request, limits(1), Some(&child_cursor))
                .unwrap_err()
        ),
        PageFailure::CursorScopeMismatch
    ));
    assert!(matches!(
        page_failure(
            document
                .redo_choices(&request, limits(1), Some(&all_cursor))
                .unwrap_err()
        ),
        PageFailure::CursorScopeMismatch
    ));
    assert!(matches!(
        page_failure(
            other
                .redo_choices(&request, limits(1), Some(&child_cursor))
                .unwrap_err()
        ),
        PageFailure::ForeignCursor
    ));
    let second = document
        .redo_choices(&request, limits(1), Some(&child_cursor))
        .unwrap();
    assert_eq!(second.children().rows()[0].step(), &children[1]);
    assert!(second.children().continuation().is_none());
    checkout(&host, &document, children[0].id(), 0x9182_f220);
    assert!(matches!(
        page_failure(
            document
                .redo_choices(&request, limits(1), Some(&child_cursor))
                .unwrap_err()
        ),
        PageFailure::StaleCursor
    ));
    assert_eq!(
        document
            .redo_choices(&request, limits(1), None)
            .unwrap()
            .choice(),
        Choice::NoChild
    );
    close(&host, document, 6);
    close(&host, other, 1);
}
