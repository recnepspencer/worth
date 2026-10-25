use std::sync::Arc;

use super::super::CompositeHistoryCatalog;
use super::fixtures::{history_contract, linear_history, HistoryFixture};
use crate::history::CompositeRuntimeWorldCommit;

fn installed(
    fixture: &HistoryFixture,
    commits: &[Arc<CompositeRuntimeWorldCommit>],
    maximum_commits: u64,
) -> CompositeHistoryCatalog {
    let owner = commits[0].identity().owner_identity();
    let catalog = CompositeHistoryCatalog::new(owner, history_contract(maximum_commits, u64::MAX));
    for commit in commits {
        catalog
            .append(commit.clone(), fixture.history_pins(commit))
            .expect("linear install");
    }
    catalog
}

fn append(
    catalog: &CompositeHistoryCatalog,
    fixture: &mut HistoryFixture,
    parent: &CompositeRuntimeWorldCommit,
) -> Arc<CompositeRuntimeWorldCommit> {
    let child = fixture.child_of(parent);
    catalog
        .append(child.clone(), fixture.history_pins(&child))
        .expect("child install")
}

#[test]
fn a_refused_fork_closing_onto_its_live_parent_releases_only_its_own_commits() {
    let (mut fixture, commits) = linear_history(3);
    let catalog = installed(&fixture, &commits, 8);
    let _parent_head = catalog.protect_product_head(&commits[2]).unwrap();
    let first = append(&catalog, &mut fixture, &commits[2]);
    let head = append(&catalog, &mut fixture, &first);
    drop(catalog.protect_product_head(&head).unwrap());

    let (released, held) = catalog
        .release_retired_branch_history(head.identity())
        .expect("release");
    assert_eq!(released.len(), 2, "the fork's two commits");
    assert_eq!(held, None, "the live parent head owns the rest");
    drop(released);
    assert_eq!(catalog.len(), 3);
    assert!(catalog.lookup(commits[2].identity()).is_some());
}

#[test]
fn a_closed_parent_a_live_fork_extends_releases_nothing_and_completes() {
    let (mut fixture, commits) = linear_history(3);
    let catalog = installed(&fixture, &commits, 8);
    let parent_head = catalog.protect_product_head(&commits[2]).unwrap();
    let fork = append(&catalog, &mut fixture, &commits[2]);
    let _fork_head = catalog.protect_product_head(&fork).unwrap();

    drop(parent_head);
    let (released, held) = catalog
        .release_retired_branch_history(commits[2].identity())
        .expect("release");
    assert!(released.is_empty(), "the fork's ancestry is all shared");
    assert_eq!(held, None, "the live fork owns its ancestry");
    assert_eq!(catalog.len(), 4);
}

#[test]
fn a_held_read_keeps_the_release_pending_until_it_lets_go() {
    let (mut fixture, commits) = linear_history(3);
    let catalog = installed(&fixture, &commits, 8);
    let _parent_head = catalog.protect_product_head(&commits[2]).unwrap();
    let first = append(&catalog, &mut fixture, &commits[2]);
    let head = append(&catalog, &mut fixture, &first);
    let read = catalog.protect_explicit_commit(&first).unwrap();

    let (released, held) = catalog
        .release_retired_branch_history(head.identity())
        .expect("release");
    assert_eq!(released.len(), 1);
    assert_eq!(held.as_ref(), Some(first.identity()), "resume at the read");
    drop(released);
    assert!(catalog.lookup(first.identity()).is_some());

    drop(read);
    let (released, held) = catalog
        .release_retired_branch_history(first.identity())
        .expect("resume");
    assert_eq!((released.len(), held), (1, None));
    drop(released);
    assert_eq!(catalog.len(), 3);
    let (released, held) = catalog
        .release_retired_branch_history(first.identity())
        .expect("a released commit has nothing left");
    assert_eq!((released.len(), held), (0, None));
}

#[test]
fn the_last_closed_branch_releases_inherited_history_nothing_live_reaches() {
    let (mut fixture, commits) = linear_history(3);
    let catalog = installed(&fixture, &commits, 8);
    let _root_head = catalog.protect_product_head(&commits[0]).unwrap();
    let parent_head = catalog.protect_product_head(&commits[2]).unwrap();
    let fork = append(&catalog, &mut fixture, &commits[2]);
    let fork_head = catalog.protect_product_head(&fork).unwrap();

    drop(parent_head);
    let (released, _) = catalog
        .release_retired_branch_history(commits[2].identity())
        .unwrap();
    assert!(released.is_empty());
    drop(fork_head);
    let (released, held) = catalog
        .release_retired_branch_history(fork.identity())
        .unwrap();
    assert_eq!(held, None);
    let released: Vec<_> = released
        .iter()
        .filter_map(|slot| slot.get())
        .map(|entry| entry.commit().identity().clone())
        .collect();
    assert_eq!(
        released,
        [
            fork.identity().clone(),
            commits[2].identity().clone(),
            commits[1].identity().clone(),
        ],
        "past the fork point down to the live root head"
    );
    assert_eq!(catalog.len(), 1);
}

#[test]
fn a_forming_child_holds_the_release_until_it_settles() {
    let (mut fixture, commits) = linear_history(2);
    let catalog = installed(&fixture, &commits, 8);
    let _root_head = catalog.protect_product_head(&commits[0]).unwrap();
    let child = fixture.child_of(&commits[1]);
    let forming = catalog.reserve(child.as_ref()).expect("child reservation");

    let (released, held) = catalog
        .release_retired_branch_history(commits[1].identity())
        .unwrap();
    assert!(released.is_empty());
    assert_eq!(held.as_ref(), Some(commits[1].identity()));

    drop(forming);
    let (released, held) = catalog
        .release_retired_branch_history(commits[1].identity())
        .unwrap();
    assert_eq!((released.len(), held), (1, None), "an abandoned child");
}

/// A transaction forks the current head, steps on the fork, and adopts it
/// by closing the old head. Retained history stays flat over 100 cycles: the
/// catalog's own commit limit would refuse an install if it grew.
#[test]
fn fork_head_step_adopt_cycles_retain_constant_history() {
    let (mut fixture, commits) = linear_history(1);
    let catalog = installed(&fixture, &commits, 6);
    let mut head = Arc::clone(&commits[0]);
    let mut head_protection = catalog.protect_product_head(&head).unwrap();
    let mut steady = None;
    for cycle in 0..100 {
        let fork = append(&catalog, &mut fixture, &head);
        let fork_protection = catalog.protect_product_head(&fork).unwrap();
        let step = append(&catalog, &mut fixture, &fork);
        let step_protection = catalog.protect_product_head(&step).unwrap();
        drop(fork_protection);
        drop(catalog.retire_unprotected_history(step.identity()).unwrap());

        drop(std::mem::replace(&mut head_protection, step_protection));
        let (released, held) = catalog
            .release_retired_branch_history(head.identity())
            .unwrap();
        assert_eq!(held, None, "cycle {cycle}: the adopted fork owns the rest");
        drop(released);
        head = step;
        drop(catalog.retire_unprotected_history(head.identity()).unwrap());

        let retained = (catalog.len(), catalog.metadata_ledger());
        if cycle >= 2 {
            assert_eq!(Some(&retained), steady.as_ref(), "cycle {cycle}");
        }
        steady = Some(retained);
    }
    assert!(catalog.len() <= 2, "the root base and the adopted head");
}
