//! A sparse owner revision with live-owner member families reverses and reapplies exactly.
use super::super::super::roster::sparse_owner::{MarkEdit, Revise};
use super::*;
use local_member::Returned;

fn revise(
    host: &world::Host,
    request: &world::Request<'_, '_, '_>,
    input: Revise,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let result = request
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(&key.to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = result else {
        panic!("{key}: {result:?}");
    };
    receipt
}
fn reapply(request: &world::Request<'_, '_, '_>, source: &WorthQueryApplicationCommitReceipt) {
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(source, limits).unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, limits, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.entries().len(), 1);
    let alternative = page.select(&page.entries()[0], limits).unwrap();
    let preparation = request
        .corrections()
        .reapply(&alternative)
        .idempotency("revise-reapply")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = preparation else {
        panic!("reapplication must prepare");
    };
    assert!(matches!(
        prepared.publish().unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
/// Members by key and value: generations are fresh after a return.
fn members(snapshot: &query::Snapshot) -> Vec<(String, u64)> {
    snapshot
        .members
        .iter()
        .map(|(_, row)| (row.key.clone(), row.value))
        .collect()
}
fn roster(host: &world::Host, request: &world::Request<'_, '_, '_>) -> query::Snapshot {
    let mut graph = image(3);
    for row in &mut graph.members {
        row.next = None;
    }
    created(host, request, graph);
    let before = local_snapshot(request);
    assert_eq!(before.value, 7);
    assert_eq!(before.mark, None);
    before
}
fn arriving(key: &str, value: u64) -> Vec<Returned> {
    vec![Returned {
        key: key.into(),
        value,
    }]
}

#[test]
fn a_sparse_owner_revision_reverses_and_reapplies_with_its_members() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = roster(&host, &request);
    let source = revise(
        &host,
        &request,
        Revise {
            owner: "roster".into(),
            value: Some(70),
            mark: MarkEdit::Set(5),
            retire: vec!["part-2".into()],
            create: arriving("part-9", 99),
        },
        "revise",
    );
    let revised = local_snapshot(&request);
    assert_eq!(
        (revised.id, revised.value, revised.mark),
        (before.id, 70, Some(5))
    );
    let expected = [("part-0", 20), ("part-1", 21), ("part-9", 99)];
    let expected: Vec<_> = expected.iter().map(|(k, v)| ((*k).into(), *v)).collect();
    assert_eq!(members(&revised), expected);
    reverse(&request, &source, "revise-undo");
    let undone = local_snapshot(&request);
    // The changed value returns, and the mark that was absent is absent again.
    assert_eq!((undone.id, undone.value, undone.mark), (before.id, 7, None));
    assert_eq!(members(&undone), members(&before));
    assert_eq!(undone.members[..2], before.members[..2]);
    assert_ne!(undone.members[2].0, before.members[2].0);
    reapply(&request, &source);
    let redone = local_snapshot(&request);
    assert_eq!(
        (redone.id, redone.value, redone.mark),
        (before.id, 70, Some(5))
    );
    assert_eq!(members(&redone), expected);
}

#[test]
fn an_unchanged_owner_field_keeps_a_later_independent_edit() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = roster(&host, &request);
    let source = revise(
        &host,
        &request,
        Revise {
            owner: "roster".into(),
            value: Some(70),
            mark: MarkEdit::Unchanged,
            retire: vec!["part-1".into()],
            create: Vec::new(),
        },
        "revise-value",
    );
    revise(
        &host,
        &request,
        Revise {
            owner: "roster".into(),
            value: None,
            mark: MarkEdit::Set(3),
            retire: Vec::new(),
            create: Vec::new(),
        },
        "independent-mark",
    );
    reverse(&request, &source, "revise-value-undo");
    let undone = local_snapshot(&request);
    // Only what the source changed is restored; the uncaptured mark is left alone.
    assert_eq!((undone.value, undone.mark), (7, Some(3)));
    assert_eq!(members(&undone), members(&before));
}

#[test]
fn a_root_only_revision_moves_no_member_and_reverses() {
    let host = world::install(world::Fault::None, Hashes::default());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = roster(&host, &request);
    let source = revise(
        &host,
        &request,
        Revise {
            owner: "roster".into(),
            value: Some(70),
            mark: MarkEdit::Unchanged,
            retire: Vec::new(),
            create: Vec::new(),
        },
        "revise-root",
    );
    assert_eq!(local_snapshot(&request).value, 70);
    reverse(&request, &source, "revise-root-undo");
    assert_eq!(local_snapshot(&request), before);
    reapply(&request, &source);
    let redone = local_snapshot(&request);
    assert_eq!((redone.value, redone.mark), (70, None));
    assert_eq!(redone.members, before.members);
}
