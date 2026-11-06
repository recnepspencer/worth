//! Recorded source reapplication restores every value and parent-presence bit.
use super::*;

#[test]
fn recorded_undo_and_redo_restore_exact_disjoint_aspect_transition_states() {
    let host = world::install();
    let original = query::Row {
        class: 1,
        first: Some(11),
        second: None,
        required: None,
        third: None,
    };
    let transitioned = query::Row {
        class: 2,
        first: None,
        second: Some(22),
        required: Some(220),
        third: None,
    };
    assert_eq!(world::read(&host), original);
    let outcome = world::edit(&host, authoring::Change::Second, "redo-source");
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: source, ..
    } = outcome
    else {
        panic!("transition must commit: {outcome:?}");
    };
    assert_eq!(world::read(&host), transitioned);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&source, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("redo-undo")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("recorded undo must prepare");
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: reversal, ..
    } = outcome
    else {
        panic!("recorded undo must commit: {outcome:?}");
    };
    assert_eq!(world::read(&host), original);

    let selected = request.corrections().select(&source, limits).unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, limits, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert!(page.continuation().is_none());
    assert_eq!(page.entries().len(), 1);
    let entry = &page.entries()[0];
    assert_eq!(entry.performed_commit(), reversal.commit_reference());
    let alternative = page.select(entry, limits).unwrap();
    let prepared = request
        .corrections()
        .reapply(&alternative)
        .idempotency("redo-reapply")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("retained source redo must prepare");
    };
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    assert_eq!(world::read(&host), transitioned);
    assert_eq!(
        world::read(&world::restore(world::checkpoint(&host))),
        transitioned
    );
}
