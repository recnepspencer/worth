//! A closed branch's release against heads that are not live: a publication's
//! unpublished successor and a head protection still in recovery custody.

use super::branch_release::{append, installed, live_head};
use super::fixtures::linear_history;

/// A publication installed its successor but never moved the head, and the
/// branch closed. The installed child owns the closed head, so the close
/// completes; recovery then releases the child by walking on from it, which
/// frees the closed branch's commits down to the live root head.
#[test]
fn recovery_walks_from_an_unpublished_successor_through_its_closed_branch() {
    let (mut fixture, commits) = linear_history(3);
    let catalog = installed(&fixture, &commits, 8);
    let _root_head = live_head(&catalog, &commits[0]);
    let branch_head = live_head(&catalog, &commits[2]);
    let unpublished = append(&catalog, &mut fixture, &commits[2]);
    let unpublished_custody = catalog.protect_product_head(&unpublished).unwrap();

    drop(branch_head);
    let (released, held) = catalog
        .release_retired_branch_history(commits[2].identity())
        .unwrap();
    assert_eq!((released.len(), held), (0, None), "the child owns the head");

    let (released, held) = catalog
        .release_retired_branch_history(unpublished.identity())
        .unwrap();
    assert_eq!(released.len(), 0);
    assert_eq!(
        held.as_ref(),
        Some(unpublished.identity()),
        "custody defers recovery's own walk until it lets go"
    );

    drop(unpublished_custody);
    let (released, held) = catalog
        .release_retired_branch_history(unpublished.identity())
        .unwrap();
    assert_eq!(held, None);
    assert_eq!(released.len(), 3, "the child and the closed branch's two");
    drop(released);
    assert_eq!(catalog.len(), 1, "only the live root head remains");
}

/// An abandoned creation keeps a full head protection on the source head in
/// recovery custody. It is not a live head: the source's close defers on it
/// and completes once custody lets go.
#[test]
fn a_head_protection_in_recovery_custody_defers_the_close() {
    let (fixture, commits) = linear_history(3);
    let catalog = installed(&fixture, &commits, 8);
    let _root_head = live_head(&catalog, &commits[0]);
    let branch_head = live_head(&catalog, &commits[2]);
    let abandoned_creation = catalog.protect_product_head(&commits[2]).unwrap();

    drop(branch_head);
    let (released, held) = catalog
        .release_retired_branch_history(commits[2].identity())
        .unwrap();
    assert!(released.is_empty());
    assert_eq!(held.as_ref(), Some(commits[2].identity()), "custody defers");

    drop(abandoned_creation);
    let (released, held) = catalog
        .release_retired_branch_history(commits[2].identity())
        .unwrap();
    assert_eq!((released.len(), held), (2, None));
    drop(released);
    assert_eq!(catalog.len(), 1);
}

/// Seating moves ownership, not protection: an unseated head still keeps its
/// commit installed, and a reseated one owns the ancestry again.
#[test]
fn unseating_a_head_turns_ownership_into_a_hold() {
    let (fixture, commits) = linear_history(2);
    let catalog = installed(&fixture, &commits, 8);
    let _root_head = live_head(&catalog, &commits[0]);
    let mut head = live_head(&catalog, &commits[1]);

    head.unseat();
    let (released, held) = catalog
        .release_retired_branch_history(commits[1].identity())
        .unwrap();
    assert_eq!(
        (released.len(), held.as_ref()),
        (0, Some(commits[1].identity()))
    );
    head.seat();
    let (released, held) = catalog
        .release_retired_branch_history(commits[1].identity())
        .unwrap();
    assert_eq!((released.len(), held), (0, None));
    drop(head);
    let (released, held) = catalog
        .release_retired_branch_history(commits[1].identity())
        .unwrap();
    assert_eq!((released.len(), held), (1, None), "a dropped seat unseats");
}

/// A live fork whose head is the old head owns it, even while another child
/// forms on it: the close completes rather than waiting on the child.
#[test]
fn a_live_head_owns_its_commit_beside_a_forming_child() {
    let (mut fixture, commits) = linear_history(2);
    let catalog = installed(&fixture, &commits, 8);
    let _root_head = live_head(&catalog, &commits[0]);
    let old_head = live_head(&catalog, &commits[1]);
    let _fork_head = live_head(&catalog, &commits[1]);
    let child = fixture.child_of(&commits[1]);
    let _forming = catalog.reserve(child.as_ref()).unwrap();

    drop(old_head);
    let (released, held) = catalog
        .release_retired_branch_history(commits[1].identity())
        .unwrap();
    assert_eq!((released.len(), held), (0, None));
}
