//! Allocator axes: the same history operation at 1x and 10x of one dimension
//! allocates at most 1.1x, by count and by requested bytes. Each size gets its
//! own host sized by `growth::host::install` (the default world holds 32 live
//! branches), so only the varied dimension differs; a smaller host runs every
//! measured kind once first, so process-wide first use is outside both samples.
use super::growth::host;
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryHistoryNavigation, WorthQueryHistoryNavigationAdvance as Advance,
    WorthQueryHistoryNodeId, WorthQueryHistoryRedoChoice,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    document_history::{navigation::Navigation, RetentionHistory},
    host::DocumentRetentionSchema,
};
use worth_testkit_foundational::allocation_probe::{assert_flat, AllocationCount, AllocationProbe};

type Owner<'d, 'r> =
    WorthQueryHistoryNavigation<'d, 'r, DocumentRetentionSchema, RetentionHistory, Navigation>;

fn select(host: &App, mut owner: Owner<'_, '_>) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for _ in 0..16 {
        owner = match owner.advance(&request) {
            Advance::Pending(owner) => owner,
            Advance::Selected(_) => return,
            Advance::Refused { cause, .. } => panic!("lawful navigation refused: {cause:?}"),
        };
    }
    panic!("bounded navigation did not select")
}

fn undo(host: &App, document: &Document<'_>, key: u128) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let owner = document
        .navigate::<Navigation>()
        .undo_step()
        .expected_selection(document.selection(&request).unwrap())
        .idempotency(key)
        .prepare(&request)
        .unwrap();
    select(host, owner);
}

fn redo_to(host: &App, document: &Document<'_>, child: WorthQueryHistoryNodeId, key: u128) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let owner = document
        .navigate::<Navigation>()
        .redo_to(child)
        .expected_selection(document.selection(&request).unwrap())
        .idempotency(key)
        .prepare(&request)
        .unwrap();
    select(host, owner);
}

fn checkout(host: &App, document: &Document<'_>, node: WorthQueryHistoryNodeId, key: u128) {
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
    select(host, owner);
}

fn edit<'a>(host: &'a App, document: Document<'a>, days: u64, key: u64) -> Document<'a> {
    finish(
        host,
        ready(host, begin(host, document, DOCUMENT_IDENTITY, days, key)),
        days,
    )
    .0
}

/// A host and document with room and a retention window for `nodes` steps,
/// so no measured operation releases retained history (as `growth` does).
fn roomy<'a>(host: &'a App, nodes: usize) -> Document<'a> {
    let mut document = attach(host, head(host), DOCUMENT_IDENTITY);
    host::window(host, &mut document, nodes);
    document
}

/// One recorded edit, then `undo_step` back, on a history `depth` steps deep.
fn at_depth(depth: usize) -> (AllocationCount, AllocationCount) {
    let host = host::install(depth + 8);
    let mut document = roomy(&host, depth + 8);
    let key = 0x9183_0000 + (depth as u64) * 0x1000;
    for offset in 0..depth {
        let days = if offset % 2 == 0 { 3 } else { 7 };
        document = edit(&host, document, days, key + offset as u64);
    }
    let before = if depth % 2 == 0 { 7 } else { 3 };
    assert_eq!(value(&host, &document), before);
    let (document, recorded) = AllocationProbe::measure(|| edit(&host, document, 5, key + 0xffe));
    let ((), undone) = AllocationProbe::measure(|| undo(&host, &document, u128::from(key + 0xfff)));
    assert_eq!(value(&host, &document), before);
    close(&host, document, depth + 3);
    (recorded, undone)
}

#[test]
fn undo_allocates_flat_in_history_depth() {
    at_depth(2);
    let (_, small) = at_depth(20);
    let (_, large) = at_depth(200);
    assert_flat(&[("depth/undo_step", small, large)]);
}

/// Each commit's naming-index ingestion walks every fork's whole ancestry, and
/// every edit forks, so recording grows with the edits ever made (#461).
#[test]
#[ignore = "held on public #461"]
fn recording_allocates_flat_in_history_depth() {
    at_depth(2);
    let (small, _) = at_depth(20);
    let (large, _) = at_depth(200);
    assert_flat(&[("depth/record", small, large)]);
}

/// The root's first redo page, then `redo_to` its newest child, with
/// `children` completed children under the root.
fn with_children(children: usize) -> (AllocationCount, AllocationCount) {
    let nodes = 2 * children + 8;
    let host = host::install(nodes);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let key = 0x9184_0000 + (children as u64) * 0x100;
    let mut document = roomy(&host, nodes);
    let context = document
        .history_page(&request, host::prefix(), None)
        .unwrap()
        .context()
        .clone();
    let root = document.current_step(&request).unwrap().id();
    let mut newest = None;
    for child in 0..children {
        if child > 0 {
            checkout(
                &host,
                &document,
                root,
                u128::from(key + 0x80 + child as u64),
            );
            host::maintain(&host, &mut document, &context);
        }
        let days = if child % 2 == 0 { 3 } else { 5 };
        document = edit(&host, document, days, key + child as u64);
        newest = Some(document.current_step(&request).unwrap().id());
    }
    checkout(&host, &document, root, u128::from(key + 0xff));
    host::maintain(&host, &mut document, &context);
    let (page, paged) = AllocationProbe::measure(|| {
        document
            .redo_choices(&request, host::prefix(), None)
            .unwrap()
    });
    assert_eq!(page.choice(), WorthQueryHistoryRedoChoice::ChoiceRequired);
    assert_eq!(page.children().rows().len(), 1);
    drop(page);
    let newest = newest.unwrap();
    let ((), redone) =
        AllocationProbe::measure(|| redo_to(&host, &document, newest, u128::from(key + 0xfe)));
    assert_eq!(document.current_step(&request).unwrap().id(), newest);
    assert_eq!(value(&host, &document), 5);
    close(&host, document, 2 * children + 2);
    (paged, redone)
}

#[test]
fn redo_choices_and_redo_to_allocate_flat_in_sibling_count() {
    with_children(2);
    let (small_page, small_redo) = with_children(4);
    let (large_page, large_redo) = with_children(40);
    assert_flat(&[
        ("branch/redo_choices", small_page, large_page),
        ("branch/redo_to", small_redo, large_redo),
    ]);
}
