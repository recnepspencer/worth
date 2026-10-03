use std::num::NonZeroUsize;

use super::super::CompositeHistoryCatalog;
use super::fixtures::{history_contract, linear_history};

#[test]
fn a_held_commit_keeps_only_itself_and_released_history_retires_to_the_head() {
    let (fixture, commits) = linear_history(5);
    let owner = commits[0].identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(5, u64::MAX));
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .expect("linear install");
    }
    let held = catalog
        .protect_explicit_commit(&commits[2])
        .expect("installed commit protection");
    let head = commits[4].identity();

    let retired = catalog.retire_unprotected_history(head).expect("retire");
    assert_eq!(
        retired.len(),
        3,
        "the prefix and the interior commit after the held one retire"
    );
    drop(retired);
    assert_eq!(catalog.len(), 2);
    assert!(catalog.lookup(commits[2].identity()).is_some());
    assert!(catalog.lookup(commits[3].identity()).is_none());

    let traversal = catalog
        .trace_ancestry(head.clone(), NonZeroUsize::new(5).unwrap())
        .expect("trace");
    let visited: Vec<_> = traversal.commits().map(|c| c.identity().clone()).collect();
    assert_eq!(
        visited,
        [head.clone(), commits[2].identity().clone()],
        "the walk skips the spliced commit"
    );
    assert_eq!(traversal.generations_to(1), Some(2));
    assert!(traversal.crossed_retired_history());
    assert_eq!(traversal.next_parent(), Some(commits[1].identity()));
    drop(traversal);

    drop(held);
    let retired = catalog.retire_unprotected_history(head).expect("retire");
    assert_eq!(retired.len(), 1, "released history retires up to the head");
    drop(retired);
    assert_eq!(catalog.len(), 1);
    assert!(catalog.lookup(head).is_some());
    assert_eq!(
        catalog
            .retire_unprotected_history(head)
            .expect("retire")
            .len(),
        0,
        "a lone head has no history behind it"
    );
}

#[test]
fn a_long_lived_sibling_keeps_its_fork_point_while_main_history_stays_steady() {
    let (mut fixture, commits) = linear_history(2);
    let owner = commits[0].identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(8, u64::MAX));
    for commit in &commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .expect("linear install");
    }
    let fork = &commits[1];
    let sibling = fixture.child_of(fork);
    catalog
        .append(sibling.clone(), fixture.history_pins(&sibling))
        .expect("sibling install");
    let _sibling_head = catalog
        .protect_explicit_commit(&sibling)
        .expect("sibling head protection");

    let mut head = std::sync::Arc::clone(fork);
    let mut head_protection = None;
    let mut steady = None;
    for cycle in 0..100 {
        let next = fixture.child_of(&head);
        catalog
            .append(next.clone(), fixture.history_pins(&next))
            .expect("main install");
        head_protection = Some(
            catalog
                .protect_explicit_commit(&next)
                .expect("main head protection"),
        );
        head = next;
        drop(
            catalog
                .retire_unprotected_history(head.identity())
                .expect("retire"),
        );
        let retained = (catalog.len(), catalog.metadata_ledger());
        if cycle >= 2 {
            assert_eq!(Some(&retained), steady.as_ref(), "cycle {cycle}");
        }
        steady = Some(retained);
    }
    assert_eq!(catalog.len(), 3, "fork point, sibling and main head");
    assert!(catalog.lookup(fork.identity()).is_some());
    assert!(catalog.lookup(sibling.identity()).is_some());

    let traversal = catalog
        .trace_ancestry(head.identity().clone(), NonZeroUsize::new(4).unwrap())
        .expect("trace");
    assert_eq!(
        traversal.visited_count(),
        2,
        "main head, then the fork point"
    );
    assert_eq!(traversal.generations_to(1), Some(100));
    drop(traversal);
    drop(head_protection);
}
