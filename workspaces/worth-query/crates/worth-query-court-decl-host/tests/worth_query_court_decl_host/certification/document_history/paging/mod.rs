//! Real completed annotations produce bounded pages independently of caller layout caches.
use super::recording::{begin, close, finish, ready};
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationHistoryDenial as Failure, WorthQueryDocumentHistoryDenial as Native,
    WorthQueryHistoryEditAdvance as Advance, WorthQueryHistoryPageDenial as PageFailure,
    WorthQueryHistoryPageLimits as Limits, WorthQueryHistoryStep as Step,
};
use worth_query_testkit_document_retention::facade::document_retention_model::record_history::capability;
mod budget;
mod handoff;

fn limits(width: usize) -> Limits {
    Limits {
        maximum_entries: NonZeroUsize::new(width).unwrap(),
        maximum_examined_entries: NonZeroUsize::new(width).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
    }
}
fn chain(host: &App) -> (Document<'_>, Vec<Step>) {
    let mut document = attach(host, head(host), DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let mut expected = vec![document
        .current_step(&host.runtime().request(&principal, &scope))
        .unwrap()];
    for (offset, days) in [3, 7, 3].into_iter().enumerate() {
        let (next, step) = finish(
            host,
            ready(
                host,
                begin(
                    host,
                    document,
                    DOCUMENT_IDENTITY,
                    days,
                    0x9182_9000 + offset as u64,
                ),
            ),
            days,
        );
        document = next;
        expected.push(step);
    }
    (document, expected)
}
fn page_failure(cause: Failure) -> PageFailure {
    let Failure::Document(Native::Paging(cause)) = cause else {
        panic!("expected page boundary: {cause:?}");
    };
    cause
}

#[test]
fn bounded_pages_rebuild_the_same_completed_tree_at_multiple_widths() {
    let host = publish_on_first_program();
    let (document, expected) = chain(&host);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    for (width, examined) in [(1, 3), (3, 1), (2, 3), (3, 2), (3, 3)] {
        let limits = Limits {
            maximum_examined_entries: NonZeroUsize::new(examined).unwrap(),
            ..limits(width)
        };
        let mut after = None;
        let mut layout = Vec::new();
        loop {
            let page = document
                .history_page(&request, limits, after.as_ref())
                .unwrap();
            assert!(page.rows().len() <= width);
            assert!(page.examined_entries() <= examined);
            assert!(page.admitted_backing_bytes() <= limits.maximum_owned_bytes.get());
            assert_eq!(page.current_step_id(), expected.last().unwrap().id());
            for row in page.rows() {
                assert_eq!(row.retained_source_at_read(), row.step().endpoint());
                layout.push(row.step().clone());
            }
            after = page.continuation();
            if after.is_none() {
                break;
            }
        }
        assert_eq!(layout, expected);
        for (index, step) in layout.iter().enumerate() {
            assert_eq!(
                step.parent(),
                index.checked_sub(1).map(|parent| expected[parent].id())
            );
            assert_eq!(
                step.label(),
                if index == 0 {
                    None
                } else {
                    Some("Set retention")
                }
            );
        }
        drop(layout); // Discard only the consumer's derived view before rebuilding.
    }
    assert_eq!(value(&host, &document), 3);
    assert_eq!(value(&host, &untouched), 7);
    close(&host, untouched, 1);
    close(&host, document, 4);
}

#[test]
fn cursors_reject_foreign_and_changed_views_without_silent_restart() {
    let host = publish_on_first_program();
    let (document, _) = chain(&host);
    let other = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let cursor = document
        .history_page(&request, limits(1), None)
        .unwrap()
        .continuation()
        .unwrap();
    assert!(matches!(
        page_failure(
            other
                .history_page(&request, limits(1), Some(&cursor))
                .unwrap_err()
        ),
        PageFailure::ForeignCursor
    ));
    let (document, _) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 7, 0x9182_9010),
        ),
        7,
    );
    assert!(matches!(
        page_failure(
            document
                .history_page(&request, limits(1), Some(&cursor))
                .unwrap_err()
        ),
        PageFailure::StaleCursor
    ));
    assert!(document.history_page(&request, limits(1), None).is_ok());
    close(&host, other, 1);
    close(&host, document, 5);
}

#[test]
fn private_reservations_and_discarded_attempts_do_not_change_the_visible_view() {
    let host = publish_on_first_program();
    let (document, expected) = chain(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let cursor = document
        .history_page(&request, limits(1), None)
        .unwrap()
        .continuation()
        .unwrap();
    let edit = begin(&host, document, DOCUMENT_IDENTITY, 7, 0x9182_9020);
    let Advance::Progress(edit) = edit.advance(&request) else {
        panic!("actual reservation");
    };
    let page = edit.history_page(&request, limits(8), None).unwrap();
    assert_eq!(
        page.rows()
            .iter()
            .map(|row| row.step().clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(page.continuation().is_none());
    assert!(edit
        .history_page(&request, limits(1), Some(&cursor))
        .is_ok());
    let document = edit.discard(&request).unwrap();
    let page = document
        .history_page(&request, limits(8), Some(&cursor))
        .unwrap();
    assert_eq!(
        page.rows()
            .iter()
            .map(|row| row.step().clone())
            .collect::<Vec<_>>(),
        expected[1..]
    );
    assert!(page.continuation().is_none());
    // Publish after the cancelled ordinal: traversal must cross a real private gap.
    let (document, later) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 5, 0x9182_9021),
        ),
        5,
    );
    let mut reference = expected.clone();
    reference.push(later);
    for width in [1, 2, 4] {
        let mut cursor = None;
        let mut visible = Vec::new();
        loop {
            let next = document
                .history_page(&request, limits(width), cursor.as_ref())
                .unwrap();
            assert!(next.rows().len() <= width);
            assert!(next.examined_entries() <= width);
            visible.extend(next.rows().iter().map(|row| row.step().clone()));
            cursor = next.continuation();
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(visible, reference);
    }
    close(&host, document, 5);
    // A retained page describes past retention; it did not secretly pin payloads.
    assert_eq!(
        page.rows()[0].retained_source_at_read(),
        expected[1].endpoint()
    );
}

#[test]
fn metadata_refusal_precedes_foreign_cursor_and_small_page_diagnostics() {
    let host = publish_on_first_program();
    let (public, _) = chain(&host);
    let mut document = super::metadata::governed(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let foreign = public
        .history_page(&request, limits(1), None)
        .unwrap()
        .continuation()
        .unwrap();
    let tiny = Limits {
        maximum_owned_bytes: NonZeroUsize::MIN,
        ..limits(1)
    };
    let observation = document.observe_history_enrollment_for_test();
    let denied = document
        .history_page(&request, tiny, Some(&foreign))
        .unwrap_err();
    let Failure::Document(Native::Metadata(
        worth_query_host::facade::primary_graph::WorthQueryApplicationHistoryDenial::Query(cause),
    )) = denied
    else {
        panic!("metadata before cursor/byte limits: {denied:?}");
    };
    assert_eq!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired);
    assert_eq!(observation.snapshot().node_probes, 0);
    drop(observation);
    let page = document
        .history_page_governed(
            &request,
            capability::MetadataCapability::reference(),
            capability::MetadataRead::reference(),
            capability::Input {
                identity: DOCUMENT_IDENTITY.to_owned(),
            },
            limits(1),
            None,
        )
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    assert_eq!(page.rows()[0].step().parent(), None);
    close(&host, public, 4);
    close(&host, document, 1);
}
