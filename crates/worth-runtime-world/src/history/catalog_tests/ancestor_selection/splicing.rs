//! A retained splice proves ancestry, never the missing intermediate contents.
use super::*;

#[test]
fn retained_ancestor_selection_follows_retirement_proof_but_exact_pages_refuse_the_gap() {
    for length in [5, 105] {
        check_retained_splice(length);
    }
}

fn check_retained_splice(length: usize) {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let (mut fixture, commits) = linear_history(length);
    let catalog = CompositeHistoryCatalog::new(
        commits[0].identity().owner_identity(),
        history_contract(u64::try_from(length + 1).unwrap(), u64::MAX),
    );
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .unwrap();
    }
    let held = catalog.protect_explicit_commit(&commits[0]).unwrap();
    let target = commits[length - 1].identity();
    let retired = catalog.retire_unprotected_history(target).unwrap();
    assert_eq!(retired.len(), length - 2);
    drop(retired);
    assert!(catalog.lookup(commits[length - 2].identity()).is_none());
    let selected = catalog
        .select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            target,
            commits[0].identity(),
            NonZeroUsize::new(2),
        )
        .expect("the retained authoritative splice proves the source is an ancestor");
    assert_eq!(selected.source().identity(), commits[0].identity());
    assert_eq!(selected.target(), target);
    assert_eq!(
        selected.parent_links_followed(),
        1,
        "one retained parent edge is followed"
    );
    assert!(matches!(
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            target,
            commits[0].identity(),
            Some(NonZeroUsize::MIN)
        ),
        Err(Denial::SelectionLimitReached)
    ));
    let exact = catalog
        .retain_exact_history_position(execution, target)
        .unwrap();
    assert!(
        matches!(
            catalog.next_exact_history_position(execution, &exact),
            Err(Denial::RetiredHistoryGap)
        ),
        "ancestry proof does not disclose a retired intermediate occurrence"
    );
    assert!(matches!(
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            target,
            commits[length - 2].identity(),
            None
        ),
        Err(Denial::SourceUnavailable)
    ));
    let sibling = fixture.child_of(&commits[0]);
    catalog
        .append(sibling.clone(), fixture.history_pins(&sibling))
        .unwrap();
    assert!(matches!(
        catalog.select_ancestor(
            crate::history::AncestorSelectionPath::Retained,
            &execution,
            target,
            sibling.identity(),
            None
        ),
        Err(Denial::NotAncestor)
    ));
    drop((selected, exact, held));
}
