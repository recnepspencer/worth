//! Parent and explicit-child commands share checkout's actual selected-head owner.
use super::*;

pub(super) fn undo(
    host: &Runtime,
    document: &Document<'_>,
    key: u128,
) -> WorthQueryHistoryNavigationCompletion {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    complete(
        host,
        document
            .navigate::<Navigation>()
            .undo_step()
            .expected_selection(document.selection(&request).unwrap())
            .idempotency(key)
            .prepare(&request)
            .unwrap(),
    )
}

fn redo(
    host: &Runtime,
    document: &Document<'_>,
    child: WorthQueryHistoryNodeId,
    key: u128,
) -> WorthQueryHistoryNavigationCompletion {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    complete(
        host,
        document
            .navigate::<Navigation>()
            .redo_to(child)
            .expected_selection(document.selection(&request).unwrap())
            .idempotency(key)
            .prepare(&request)
            .unwrap(),
    )
}

#[test]
fn parent_undo_and_explicit_child_redo_preserve_both_authored_futures() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let (document, s1) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_f001),
        ),
        3,
    );
    let (document, s2) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 7, 0x9182_f002),
        ),
        7,
    );
    let (document, s3) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 5, 0x9182_f003),
        ),
        5,
    );
    let first = undo(&host, &document, 0x9182_f010);
    assert_eq!(first.selection().step(), &s2);
    assert_eq!(value(&host, &document), 7);
    let second = undo(&host, &document, 0x9182_f011);
    assert_eq!(second.selection().step(), &s1);
    assert_eq!(value(&host, &document), 3);
    let (document, t2) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 4, 0x9182_f004),
        ),
        4,
    );
    assert_eq!(t2.parent(), Some(s1.id()));
    undo(&host, &document, 0x9182_f012);
    let original = redo(&host, &document, s2.id(), 0x9182_f013);
    assert_eq!(original.selection().step(), &s2);
    assert_eq!(value(&host, &document), 7);
    let original = redo(&host, &document, s3.id(), 0x9182_f014);
    assert_eq!(original.selection().step(), &s3);
    assert_eq!(value(&host, &document), 5);
    undo(&host, &document, 0x9182_f015);
    undo(&host, &document, 0x9182_f016);
    let alternative = redo(&host, &document, t2.id(), 0x9182_f017);
    assert_eq!(alternative.selection().step(), &t2);
    assert_eq!(value(&host, &document), 4);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for (step, days) in [(&s3, 5), (&t2, 4)] {
        let preview = document.preview(&request, step.id()).unwrap();
        let read = preview.read(&document, &request).unwrap();
        assert_eq!(read.rows()[0].retention_days, days);
        assert_eq!(read.source(), step.endpoint());
        read.close();
        preview.close();
    }
    assert_eq!(value(&host, &untouched), SEED_RETENTION);
    close(&host, document, 13);
    close(&host, untouched, 1);
}

#[test]
fn edge_commands_refuse_invalid_targets_and_bind_original_command_meaning_on_retry() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let other = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.selection(&request).unwrap();
    let refusal = document
        .navigate::<Navigation>()
        .undo_step()
        .expected_selection(root.clone())
        .idempotency(0x9182_f100)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(refusal, Failure::Navigation(Denial::NoParent)),
        "{refusal:?}"
    );
    let (document, s1) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_f101),
        ),
        3,
    );
    let stale = selection(&host, &document);
    let (document, s2) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 5, 0x9182_f102),
        ),
        5,
    );
    let expected = selection(&host, &document);
    let foreign = other.current_step(&request).unwrap().id();
    for (node, foreign) in [(root.step().id(), false), (foreign, true)] {
        let refusal = document
            .navigate::<Navigation>()
            .redo_to(node)
            .expected_selection(expected.clone())
            .idempotency(0x9182_f103)
            .prepare(&request)
            .unwrap_err();
        assert!(
            if foreign {
                matches!(refusal, Failure::Navigation(Denial::Document(worth_query_host::facade::application_entry::WorthQueryDocumentHistoryDenial::ForeignHistoryNode)))
            } else {
                matches!(refusal, Failure::Navigation(Denial::NotChild))
            },
            "{refusal:?}"
        );
    }
    let refusal = document
        .navigate::<Navigation>()
        .undo_step()
        .expected_selection(stale)
        .idempotency(0x9182_f104)
        .prepare(&request)
        .unwrap_err();
    assert!(
        matches!(refusal, Failure::Navigation(Denial::HeadChanged)),
        "{refusal:?}"
    );
    assert_eq!(selection(&host, &document), expected);
    let original_undo = undo(&host, &document, 0x9182_f105);
    assert_eq!(original_undo.selection().step(), &s1);
    let redo_expected = selection(&host, &document);
    let original_redo = redo(&host, &document, s2.id(), 0x9182_f106);
    assert_eq!(original_redo.selection().step(), &s2);
    let later = selection(&host, &document);
    let retried = complete(
        &host,
        document
            .navigate::<Navigation>()
            .undo_step()
            .expected_selection(expected.clone())
            .idempotency(0x9182_f105)
            .prepare(&request)
            .unwrap(),
    );
    assert_eq!(retried.selection(), original_undo.selection());
    assert_eq!(selection(&host, &document), later);
    let retried = complete(
        &host,
        document
            .navigate::<Navigation>()
            .redo_to(s2.id())
            .expected_selection(redo_expected.clone())
            .idempotency(0x9182_f106)
            .prepare(&request)
            .unwrap(),
    );
    assert_eq!(retried.selection(), original_redo.selection());
    assert_eq!(selection(&host, &document), later);
    // Checkout reaches the SAME node but is a different original command.
    for (node, expected, key) in [
        (s1.id(), expected, 0x9182_f105),
        (s2.id(), redo_expected, 0x9182_f106),
    ] {
        let refusal = document
            .navigate::<Navigation>()
            .checkout(node)
            .expected_selection(expected)
            .idempotency(key)
            .prepare(&request)
            .unwrap_err();
        assert!(
            matches!(refusal, Failure::Navigation(Denial::IntentDrift)),
            "{refusal:?}"
        );
    }
    assert_eq!(selection(&host, &document), later);
    assert_eq!(value(&host, &document), 5);
    close(&host, document, 5);
    close(&host, other, 1);
}
