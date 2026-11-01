//! Snapshot checkout preserves authored futures and creates a writable context.
use super::recording::{begin, close, finish, ready};
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryHistoryNavigation, WorthQueryHistoryNavigationAdvance as Advance,
    WorthQueryHistoryNavigationAdvanceDenial as Failure, WorthQueryHistoryNavigationCompletion,
    WorthQueryHistoryNavigationDenial as Denial, WorthQueryHistoryNavigationPosture as Posture,
    WorthQueryHistoryNodeId, WorthQueryHistoryPageLimits, WorthQueryHistorySelection,
};
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::navigation::Navigation;

mod authority;
mod cancellation;
mod closing;
mod commands;
mod eligibility;
mod race;
mod recovery;
mod retirement;
mod retries;
mod user_tree;

type Owner<'d, 'r> =
    WorthQueryHistoryNavigation<'d, 'r, DocumentRetentionSchema, RetentionHistory, Navigation>;

fn prepare<'d, 'r>(
    host: &Runtime,
    document: &'d Document<'r>,
    node: WorthQueryHistoryNodeId,
    expected: WorthQueryHistorySelection,
    key: u128,
) -> Owner<'d, 'r> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    document
        .navigate::<Navigation>()
        .checkout(node)
        .expected_selection(expected)
        .idempotency(key)
        .prepare(&host.request(&principal, &scope))
        .unwrap()
}

fn pending<'d, 'r>(host: &Runtime, owner: Owner<'d, 'r>) -> Owner<'d, 'r> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    match owner.advance(&host.request(&principal, &scope)) {
        Advance::Pending(owner) => owner,
        Advance::Selected(_) => panic!("private candidate remains unselected"),
        Advance::Refused { cause, .. } => panic!("lawful private progression refused: {cause:?}"),
    }
}

fn awaiting<'d, 'r>(host: &Runtime, mut owner: Owner<'d, 'r>) -> Owner<'d, 'r> {
    for _ in 0..16 {
        if owner.posture() == Posture::AwaitingSelection {
            return owner;
        }
        owner = pending(host, owner);
    }
    panic!("candidate publication did not finish")
}

pub(super) fn complete(
    host: &Runtime,
    mut owner: Owner<'_, '_>,
) -> WorthQueryHistoryNavigationCompletion {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for _ in 0..16 {
        owner = match owner.advance(&request) {
            Advance::Pending(owner) => owner,
            Advance::Selected(completion) => return completion,
            Advance::Refused { cause, .. } => panic!("lawful checkout refused: {cause:?}"),
        };
    }
    panic!("bounded checkout did not select")
}

fn checkout(
    host: &Runtime,
    document: &Document<'_>,
    node: WorthQueryHistoryNodeId,
    expected: WorthQueryHistorySelection,
    key: u128,
) -> WorthQueryHistoryNavigationCompletion {
    complete(host, prepare(host, document, node, expected, key))
}

fn selection(host: &Runtime, document: &Document<'_>) -> WorthQueryHistorySelection {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    document
        .selection(&host.request(&principal, &scope))
        .unwrap()
}

#[test]
fn checkout_forks_from_an_old_occurrence_and_keeps_both_authored_futures() {
    let host = publish_on_first_program();
    let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let untouched = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let root = document.current_step(&request).unwrap();
    let (document, s1) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 3, 0x9182_c001),
        ),
        3,
    );
    let (document, s2) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 7, 0x9182_c002),
        ),
        7,
    );
    let (document, s3) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 5, 0x9182_c003),
        ),
        5,
    );
    let original = [root, s1.clone(), s2, s3.clone()];
    use worth_query_testkit_document_retention::facade::document_retention_model::{
        presented_request::set_retention,
        readback::read_retention,
        settled_verdict::{settle, RetentionVerdict},
    };
    assert_eq!(
        settle(set_retention(
            &host,
            s1.endpoint().product_branch(),
            6,
            0x9182_c005
        )),
        RetentionVerdict::Performed(6)
    );
    assert_eq!(read_retention(&host, s1.endpoint().product_branch()), 6);
    let before = selection(&host, &document);
    let first = checkout(&host, &document, s1.id(), before.clone(), 0x9182_c010);
    assert_eq!(first.requested_node(), s1.id());
    assert_eq!(first.selection().step(), &s1);
    assert_ne!(first.selection().physical_endpoint(), s1.endpoint());
    assert_ne!(
        first.selection().physical_endpoint(),
        before.physical_endpoint()
    );
    assert_eq!(selection(&host, &document), *first.selection());
    assert_eq!(value(&host, &document), 3);
    let page = document
        .history_page(
            &request,
            WorthQueryHistoryPageLimits {
                maximum_entries: NonZeroUsize::new(16).unwrap(),
                maximum_examined_entries: NonZeroUsize::new(16).unwrap(),
                maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
            },
            None,
        )
        .unwrap();
    let rows: Vec<_> = page.rows().iter().map(|row| row.step().clone()).collect();
    assert_eq!(rows, original);
    assert!(page.continuation().is_none());
    drop(page);
    let (document, t2) = finish(
        &host,
        ready(
            &host,
            begin(&host, document, DOCUMENT_IDENTITY, 4, 0x9182_c004),
        ),
        4,
    );
    assert_eq!(t2.parent(), Some(s1.id()));
    assert_ne!(t2.id(), s3.id());
    let preview = document.preview(&request, s3.id()).unwrap();
    let old = preview.read(&document, &request).unwrap();
    assert_eq!(old.rows()[0].retention_days, 5);
    assert_eq!(old.source(), s3.endpoint());
    old.close();
    preview.close();
    let second = checkout(
        &host,
        &document,
        s3.id(),
        selection(&host, &document),
        0x9182_c011,
    );
    assert_eq!(second.selection().step(), &s3);
    assert_ne!(second.selection().physical_endpoint(), s3.endpoint());
    assert_eq!(value(&host, &document), 5);
    assert_eq!(value(&host, &untouched), SEED_RETENTION);
    close(&host, document, 7);
    close(&host, untouched, 1);
}
