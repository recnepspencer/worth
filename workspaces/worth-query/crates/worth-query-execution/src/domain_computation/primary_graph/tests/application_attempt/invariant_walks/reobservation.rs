//! Exact proposal views freshly observe reached survivors and real CRUD absences.
use super::probe::Probe;
use super::*;
use crate::facade::application_invariants::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
pub(super) mod declaration;
mod program;

#[test]
fn incidence_reached_untouched_survivor_is_freshly_observed_in_both_views() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let contacts = Arc::new(AtomicUsize::new(0));
    let contacted = contacts.clone();
    let probe = Probe::new(move |context, scope, rule| {
        let before = scope
            .committed
            .iter()
            .find(|(key, _)| *key == 3)
            .unwrap()
            .1
            .clone();
        let proposed = context.proposed();
        assert!(proposed
            .touched_entities_of(&rule.node)
            .unwrap()
            .iter()
            .all(|node| node.entity_id() != before.entity_id()));
        assert_eq!(
            proposed.field(&rule.key, &before).unwrap_err().kind(),
            WorthQueryInvariantAccessDenialKind::ForeignView
        );
        let current = proposed
            .reobserve_entity(&rule.node, &before)
            .unwrap()
            .unwrap();
        assert_eq!(current.entity_id(), before.entity_id());
        assert!(
            current != before,
            "the actual target posture is freshly issued"
        );
        assert_eq!(proposed.field(&rule.key, &current).unwrap(), Some(3));
        let committed = context.committed();
        assert_eq!(
            committed.field(&rule.key, &current).unwrap_err().kind(),
            WorthQueryInvariantAccessDenialKind::ForeignView
        );
        let old = committed
            .reobserve_entity(&rule.node, &current)
            .unwrap()
            .unwrap();
        assert!(old == before);
        assert_eq!(committed.field(&rule.key, &old).unwrap(), Some(3));
        contacted.fetch_add(1, Ordering::SeqCst);
    });
    let world = world::install_with_probe(world::Graph::three_hops(false, vec![]), probe.clone());
    let request = super::super::super::fixture::live_scope();
    let change = super::program::set_active(&world, &request, true);
    probe.arm();
    assert!(matches!(
        commit(&world, &request, change, 253),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert!(contacts.load(Ordering::SeqCst) > 0);
}

#[test]
fn new_and_removed_entities_return_absence_only_in_the_view_where_they_do_not_exist() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let stage = Arc::new(AtomicUsize::new(0));
    let mode = stage.clone();
    let contacts = Arc::new(AtomicUsize::new(0));
    let contacted = contacts.clone();
    let probe = Probe::new(move |context, scope, rule| {
        let find = |nodes: &Vec<(u64, super::observation::Entity)>| {
            nodes
                .iter()
                .find(|(key, _)| *key == 6)
                .map(|(_, node)| node.clone())
        };
        let old = find(&scope.committed);
        let new = find(&scope.proposed);
        match mode.load(Ordering::SeqCst) {
            1 => {
                assert!(old.is_none());
                let created = new.expect("real proposed creation");
                assert!(context
                    .committed()
                    .reobserve_entity(&rule.node, &created)
                    .unwrap()
                    .is_none());
                assert!(context
                    .proposed()
                    .reobserve_entity(&rule.node, &created)
                    .unwrap()
                    .is_some());
            }
            2 => {
                assert!(new.is_none());
                let removed = old.expect("real committed preimage of deletion");
                assert!(context
                    .proposed()
                    .reobserve_entity(&rule.node, &removed)
                    .unwrap()
                    .is_none());
                assert!(context
                    .committed()
                    .reobserve_entity(&rule.node, &removed)
                    .unwrap()
                    .is_some());
            }
            _ => panic!("actual CRUD stage required"),
        }
        contacted.fetch_add(1, Ordering::SeqCst);
    });
    let world = world::install_with_probe(world::Graph::three_hops(false, vec![]), probe.clone());
    let request = super::super::super::fixture::live_scope();
    for (ordinal, change) in [program::Change::Create, program::Change::Delete]
        .into_iter()
        .enumerate()
    {
        stage.store(ordinal + 1, Ordering::SeqCst);
        let change = program::change_node(&world, &request, change);
        probe.arm();
        assert!(matches!(
            commit(&world, &request, change, 254 + ordinal as u8),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }
    assert!(contacts.load(Ordering::SeqCst) >= 2);
}
