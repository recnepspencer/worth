//! Typed handoffs over an authored cycle, diamond and disconnected members.
use super::probe::Probe;
use super::*;
use crate::facade::application_invariants::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[test]
fn installed_walks_preserve_minimum_distance_direction_and_complete_physical_edges() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let contacts = Arc::new(AtomicUsize::new(0));
    let observed = contacts.clone();
    let edges = vec![(0, 1), (0, 2), (1, 3), (2, 3), (3, 1), (3, 3)];
    let expected_edges = edges.clone();
    let probe = Probe::new(move |context, scope, rule| {
        let view = context.proposed();
        let walk = view
            .walk(&rule.next, RelationDirection::Forward, scope.node(0))
            .unwrap();
        let before = context.reads();
        let visits = walk
            .iter()
            .map(|s| (s.distance(), s.entity().entity_id()))
            .collect::<Vec<_>>();
        assert_eq!(walk.len(), 4);
        assert!(!walk.is_empty());
        assert_eq!(walk.entities().count(), 4);
        assert_eq!(walk.relations().count(), 6);
        assert_eq!(
            context.reads(),
            before,
            "iteration materializes identities without reads"
        );
        let mut ordered = visits.clone();
        ordered.sort();
        assert_eq!(
            visits, ordered,
            "distance then native identity; keys are not the ordering oracle"
        );
        let mut keyed = walk
            .iter()
            .map(|step| {
                (
                    view.field(&rule.key, step.entity()).unwrap().unwrap(),
                    step.distance(),
                )
            })
            .collect::<Vec<_>>();
        keyed.sort();
        assert_eq!(keyed, [(0, 0), (1, 1), (2, 1), (3, 2)]);
        let mut physical = walk
            .relations()
            .map(|edge| {
                (
                    view.field(&rule.key, edge.from()).unwrap().unwrap(),
                    view.field(&rule.key, edge.to()).unwrap().unwrap(),
                )
            })
            .collect::<Vec<_>>();
        physical.sort();
        assert_eq!(
            physical, expected_edges,
            "back/self and second diamond edges remain admitted"
        );

        let reversed = view
            .walk(&rule.next, RelationDirection::Reverse, scope.node(3))
            .unwrap();
        let mut reverse_keys = reversed
            .iter()
            .map(|step| {
                (
                    view.field(&rule.key, step.entity()).unwrap().unwrap(),
                    step.distance(),
                )
            })
            .collect::<Vec<_>>();
        reverse_keys.sort();
        assert_eq!(reverse_keys, [(0, 2), (1, 1), (2, 1), (3, 0)]);
        let roots = [
            scope.node(3).clone(),
            scope.node(0).clone(),
            scope.node(3).clone(),
        ];
        let closure = view
            .closure(&rule.next, RelationDirection::Forward, &roots)
            .unwrap();
        let mut keyed_closure = closure
            .iter()
            .map(|step| {
                (
                    view.field(&rule.key, step.entity()).unwrap().unwrap(),
                    step.distance(),
                )
            })
            .collect::<Vec<_>>();
        keyed_closure.sort();
        assert_eq!(keyed_closure, [(0, 0), (1, 1), (2, 1), (3, 0)]);
        let facts = closure
            .iter()
            .map(|s| (s.entity().entity_id(), s.distance()))
            .collect::<Vec<_>>();
        let permuted = [scope.node(0).clone(), scope.node(3).clone()];
        let other = view
            .closure(&rule.next, RelationDirection::Forward, &permuted)
            .unwrap();
        assert_eq!(
            facts,
            other
                .iter()
                .map(|s| (s.entity().entity_id(), s.distance()))
                .collect::<Vec<_>>()
        );
        for key in [0, 3] {
            assert!(closure
                .iter()
                .any(|s| s.entity() == scope.node(key) && s.distance() == 0));
        }
        assert!(view
            .closure(&rule.next, RelationDirection::Forward, &[])
            .unwrap()
            .is_empty());
        for (from, to, expected) in [
            (0, 0, true),
            (0, 3, true),
            (0, 4, false),
            (4, 4, true),
            (4, 0, false),
        ] {
            assert_eq!(
                view.reaches(&rule.next, scope.node(from), scope.node(to))
                    .unwrap(),
                expected
            );
        }
        let committed = context.committed();
        let before = context.reads();
        let denied = committed
            .walk(&rule.next, RelationDirection::Forward, scope.node(0))
            .unwrap_err();
        assert_eq!(
            denied.kind(),
            WorthQueryInvariantAccessDenialKind::ForeignView
        );
        assert_eq!(
            context.reads(),
            before,
            "posture rejection precedes graph acquisition"
        );
        let local = &scope.committed.iter().find(|(key, _)| *key == 0).unwrap().1;
        assert_eq!(
            committed
                .walk(&rule.next, RelationDirection::Forward, local)
                .unwrap()
                .len(),
            4
        );
        observed.fetch_add(1, Ordering::SeqCst);
    });
    let world = world::install_with_probe(
        world::Graph {
            nodes: 6,
            edges,
            active: false,
            blocked: vec![],
        },
        probe.clone(),
    );
    let request = super::super::super::fixture::live_scope();
    let program = program::set_active(&world, &request, true);
    probe.arm();
    assert!(matches!(
        commit(&world, &request, program, 215),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert!(
        contacts.load(Ordering::SeqCst) > 0,
        "the real installed callback completed every assertion"
    );
}
