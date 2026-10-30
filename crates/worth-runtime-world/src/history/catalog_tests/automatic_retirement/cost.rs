use super::*;

#[test]
fn a_new_child_does_not_revisit_the_unchanged_protected_history() {
    let mut visits = Vec::new();
    for retained in [1, 100] {
        let (mut fixture, commits) = linear_history(retained);
        let catalog = CompositeHistoryCatalog::new(
            commits[0].identity().owner_identity(),
            history_contract(retained as u64 + 1, u64::MAX),
        );
        let mut held = Vec::new();
        for commit in &commits {
            catalog
                .append(commit.clone(), fixture.history_pins(commit))
                .unwrap();
            held.push(catalog.protect_explicit_commit(commit).unwrap());
            assert!(catalog.retire_changed_history().is_empty());
        }
        let before = catalog.counters().reachability_lookups();
        let next = fixture.child_of(commits.last().unwrap());
        catalog
            .append(next.clone(), fixture.history_pins(&next))
            .unwrap();
        held.push(catalog.protect_explicit_commit(&next).unwrap());
        assert!(catalog.retire_changed_history().is_empty());
        visits.push(catalog.counters().reachability_lookups() - before);
        assert_eq!(catalog.len(), retained + 1);
        let unchanged = catalog.counters().reachability_lookups();
        assert!(catalog.retire_changed_history().is_empty());
        assert_eq!(
            catalog.counters().reachability_lookups(),
            unchanged,
            "an unchanged frontier does no reachability work"
        );
    }
    assert_eq!(
        visits[0], visits[1],
        "one child has the same owner work at both retained populations"
    );
}
