//! Independent authored cohorts expose touched-only and wrong-state substitutes.
use super::super::*;
use crate::facade::application_invariants::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

pub(super) fn graph() -> world::Graph {
    world::Graph {
        nodes: 7,
        edges: vec![(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3), (1, 0)],
        active: false,
        blocked: vec![],
    }
}
fn cohorts<const O: bool>(
    view: &WorthQueryApplicationInvariantReadView<'_, OrbitSchema<O>>,
    rule: &probe::Rule<O>,
) -> Vec<Vec<u64>> {
    let all = view.components(&rule.shells).unwrap();
    let mut result = all
        .iter()
        .map(|group| {
            let mut members = group
                .members()
                .map(|node| view.field(&rule.key, &node).unwrap().unwrap())
                .collect::<Vec<_>>();
            members.sort_unstable();
            members
        })
        .collect::<Vec<_>>();
    result.sort();
    result
}
#[test]
fn complete_invariant_census_preserves_unaffected_isolated_members_and_exact_proposed_splits() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    let stage = Arc::new(AtomicUsize::new(1));
    let current = stage.clone();
    let contacts = Arc::new(AtomicUsize::new(0));
    let seen = contacts.clone();
    let stable = Arc::new(Mutex::new(None));
    let isolated = stable.clone();
    let observer = probe::Probe::<false>::new(move |context, scope, rule| {
        let proposed = context.proposed();
        let committed = context.committed();
        let separate = vec![vec![0, 1, 2], vec![3, 4, 5], vec![6]];
        let joined = vec![vec![0, 1, 2, 3, 4, 5], vec![6]];
        let (before, after) = if current.load(Ordering::SeqCst) == 1 {
            (separate, joined)
        } else {
            (joined, separate)
        };
        assert_eq!(cohorts(&committed, rule), before);
        assert_eq!(cohorts(&proposed, rule), after);
        assert_eq!(proposed.component_count(&rule.shells).unwrap(), after.len());
        let all = proposed.components(&rule.shells).unwrap();
        for group in all.iter() {
            for member in group.members() {
                let selected = proposed.component_of(&rule.shells, &member).unwrap();
                assert_eq!(selected.identity(), group.identity());
                assert_eq!(
                    selected
                        .members()
                        .map(|e| e.entity_id())
                        .collect::<Vec<_>>(),
                    group.members().map(|e| e.entity_id()).collect::<Vec<_>>()
                );
            }
        }
        let singleton = proposed.component_of(&rule.shells, scope.node(6)).unwrap();
        assert_eq!(singleton.len(), 1);
        let mut original = isolated.lock().unwrap();
        if let Some(identity) = original.as_ref() {
            assert_eq!(identity, singleton.identity());
        } else {
            *original = Some(singleton.identity().clone());
        }
        assert_eq!(
            proposed
                .component_of(&rule.shells, &scope.committed[0].1)
                .err()
                .unwrap()
                .kind(),
            WorthQueryInvariantAccessDenialKind::ForeignView
        );
        seen.fetch_add(1, Ordering::SeqCst);
    })
    .complete_components();
    let world = world::install_with_probe(graph(), observer.clone());
    let request = super::super::super::super::fixture::live_scope();
    observer.arm();
    for next in [1, 2] {
        stage.store(next, Ordering::SeqCst);
        let before = contacts.load(Ordering::SeqCst);
        let operation = program::swap_successors(&world, &request, 2, 5);
        assert!(matches!(
            commit(&world, &request, operation, 180 + next as u8),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        assert!(
            contacts.load(Ordering::SeqCst) > before,
            "every real coupling edit freshly ran the installed rule"
        );
    }
}
#[test]
fn component_count_is_an_ordinary_proposed_commit_verdict() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    let observer = probe::Probe::<false>::new(|_, _, _| {}).require_count(3);
    let world = world::install_with_probe(graph(), observer.clone());
    let request = super::super::super::super::fixture::live_scope();
    let before = world.committed_version();
    let edges = world.edges();
    let hostile = program::swap_successors(&world, &request, 2, 5);
    exact_violation(
        commit(&world, &request, hostile, 184),
        probe::Refusal {
            root: 0,
            blocked: 2,
            ordinal: 3,
        },
    );
    assert_eq!(world.committed_version(), before);
    assert_eq!(world.edges(), edges);
    let lawful = program::swap_successors(&world, &request, 0, 1);
    // The twin replaces the redundant 0↔1 pair with self edges; 0–2–1 still couples the same members.
    assert!(matches!(
        commit(&world, &request, lawful, 185),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
}
#[test]
fn complete_census_cannot_widen_an_ordinary_prepared_member_footprint() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let observer = probe::Probe::<false>::new(move |context, scope, rule| {
        let view = context.proposed();
        assert!(view.component_of(&rule.shells, scope.node(0)).is_ok());
        assert_eq!(
            view.components(&rule.shells).err().unwrap().kind(),
            WorthQueryInvariantAccessDenialKind::OutsidePreparedScope
        );
        seen.fetch_add(1, Ordering::SeqCst);
    });
    let world = world::install_with_probe(graph(), observer.clone());
    observer.arm();
    let request = super::super::super::super::fixture::live_scope();
    let operation = program::swap_successors(&world, &request, 0, 1);
    assert!(matches!(
        commit(&world, &request, operation, 186),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert!(calls.load(Ordering::SeqCst) > 0);
}
