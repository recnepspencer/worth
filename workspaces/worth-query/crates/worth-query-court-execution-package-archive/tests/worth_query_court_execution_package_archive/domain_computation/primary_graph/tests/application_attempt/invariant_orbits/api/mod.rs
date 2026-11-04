//! Actual installed orbit reads preserve committed naming and prepared authority.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::*;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantOrbit;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantReadView;
use worth_query_execution::facade::application_invariants::WorthQueryInvariantAccessDenialKind;

pub fn keys(
    view: &WorthQueryApplicationInvariantReadView<'_, OrbitSchema<true>>,
    rule: &worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Rule<true>,
    orbit: &WorthQueryApplicationInvariantOrbit<OrbitSchema<true>, worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::Next, Node>,
) -> Vec<u64> {
    orbit
        .members()
        .map(|entity| view.field(&rule.key, &entity).unwrap().unwrap())
        .collect()
}

#[test]
fn installed_orbits_keep_anchor_order_identity_and_separate_before_and_proposed_splits() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let calls = Arc::new(AtomicUsize::new(0));
    let received = calls.clone();
    let phase = Arc::new(AtomicUsize::new(0));
    phase.store(1, Ordering::SeqCst);
    let current_phase = phase.clone();
    let original = Arc::new(Mutex::new(None));
    let old_identity = original.clone();
    let callback = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe::new(move |context, scope, rule| {
        received.fetch_add(1, Ordering::SeqCst);
        let stage = current_phase.load(Ordering::SeqCst);
        let least = scope
            .proposed
            .iter()
            .min_by_key(|(_, entity)| entity.entity_id())
            .unwrap()
            .0;
        let (keeper, other_cycle): (&[u64], &[u64]) = if [0, 2].contains(&least) {
            (&[0, 2], &[4, 1, 3, 5])
        } else {
            (&[4, 1, 3, 5], &[0, 2])
        };
        let selected = *keeper.iter().find(|&&key| key != least).unwrap();
        let other_key = other_cycle[0];
        let rotate = |cycle: &[u64]| {
            let first = cycle
                .iter()
                .enumerate()
                .min_by_key(|(_, key)| scope.node(**key).entity_id())
                .unwrap()
                .0;
            cycle[first..]
                .iter()
                .chain(&cycle[..first])
                .copied()
                .collect::<Vec<_>>()
        };
        let proposed = context.proposed();
        let orbit = proposed.orbit(&rule.next, scope.node(selected)).unwrap();
        let before = context.reads();
        let member_ids = orbit
            .members()
            .map(|entity| entity.entity_id())
            .collect::<Vec<_>>();
        assert_eq!(orbit.relations().len(), orbit.len());
        assert!(!orbit.is_empty());
        assert!(orbit.member(orbit.len()).is_none());
        assert_eq!(
            context.reads(),
            before,
            "iteration performs no Native reads"
        );
        let anchor = member_ids.iter().copied().min().unwrap();
        assert_eq!(orbit.member(0).unwrap().entity_id(), anchor);
        let order = keys(&proposed, rule, &orbit);
        assert_eq!(
            order,
            rotate(if stage == 1 {
                keeper
            } else {
                &[0, 2, 4, 1, 3, 5]
            })
        );
        assert_eq!(
            proposed.orbit_of(&rule.next, scope.node(selected)).unwrap(),
            *orbit.identity()
        );
        let committed = context.committed();
        assert_eq!(
            committed
                .orbit(&rule.next, scope.node(selected))
                .unwrap_err()
                .kind(),
            WorthQueryInvariantAccessDenialKind::ForeignView
        );
        let local = |key| &scope.committed.iter().find(|(k, _)| *k == key).unwrap().1;
        let old = committed.orbit(&rule.next, local(selected)).unwrap();
        let mut saved = old_identity.lock().unwrap();
        if saved.is_none() {
            *saved = Some(old.identity().clone());
        }
        assert_eq!(orbit.identity(), saved.as_ref().unwrap());
        assert_eq!(old.identity(), saved.as_ref().unwrap());
        assert_eq!(old.len(), if stage == 2 { keeper.len() } else { 6 });
        let other = proposed.orbit(&rule.next, scope.node(other_key)).unwrap();
        if stage == 1 {
            assert_ne!(other.identity(), orbit.identity());
            assert_eq!(keys(&proposed, rule, &other), rotate(other_cycle));
        } else {
            assert_eq!(other.identity(), orbit.identity());
        }
        if stage == 2 {
            assert_ne!(
                committed
                    .orbit(&rule.next, local(other_key))
                    .unwrap()
                    .identity(),
                orbit.identity()
            );
        }
    });
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::install_orbit(
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::Graph {
            nodes: 6,
            edges: vec![(0, 2), (2, 4), (4, 1), (1, 3), (3, 5), (5, 0)],
            active: false,
            blocked: vec![],
        },
        callback.clone(),
    );
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    callback.arm();
    for stage in [1, 2] {
        phase.store(stage, Ordering::SeqCst);
        let operation = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::swap_successors(&world, &request, 2, 5);
        let previous = calls.load(Ordering::SeqCst);
        assert!(matches!(
            commit(&world, &request, operation, 230 + stage as u8),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
        assert!(
            calls.load(Ordering::SeqCst) > previous,
            "each real commit reached the installed callback"
        );
    }
    assert!(calls.load(Ordering::SeqCst) >= 2);
}

#[test]
fn cyclic_endorelation_without_orbit_declaration_is_a_typed_refusal() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    let callback = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe::<false>::new(move |context, scope, rule| {
        let refusal = context
            .proposed()
            .orbit(&rule.next, scope.node(0))
            .unwrap_err();
        assert_eq!(
            refusal.kind(),
            WorthQueryInvariantAccessDenialKind::NotOrbitRelation
        );
        observed.fetch_add(1, Ordering::SeqCst);
    });
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::install_with_probe(
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::Graph {
            nodes: 3,
            edges: vec![(0, 1), (1, 2), (2, 0)],
            active: false,
            blocked: vec![],
        },
        callback.clone(),
    );
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    callback.arm();
    let operation = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::swap_successors(&world, &request, 0, 2);
    assert!(matches!(
        commit(&world, &request, operation, 233),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert!(calls.load(Ordering::SeqCst) > 0);
}

#[test]
fn deep_orbit_splice_prepares_every_edge_and_member_for_exact_custom_denial_and_lawful_twin() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let expected_ordinal = Arc::new(AtomicUsize::new(usize::MAX));
    let expected = expected_ordinal.clone();
    let observer = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe::new(move |_, scope, _| {
        if !scope.proposed.iter().any(|(key, _)| *key == 5) {
            return;
        }
        let cycle = [0u64, 1, 2, 3, 4, 5];
        let least = cycle
            .iter()
            .enumerate()
            .min_by_key(|(_, key)| scope.node(**key).entity_id())
            .unwrap()
            .0;
        expected.store((5 + cycle.len() - least) % cycle.len(), Ordering::SeqCst);
    });
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::install_orbit(
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::Graph {
            nodes: 9,
            edges: vec![
                (0, 1),
                (1, 2),
                (2, 0),
                (3, 4),
                (4, 5),
                (5, 3),
                (6, 7),
                (7, 8),
                (8, 6),
            ],
            active: true,
            blocked: vec![5],
        },
        observer.clone(),
    );
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let before = world.committed_version();
    let edges = world.edges();
    world.observation.arm();
    observer.arm();
    let hostile = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::swap_successors(&world, &request, 2, 5);
    exact_violation(
        commit(&world, &request, hostile, 234),
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Refusal {
            root: 0,
            blocked: 5,
            ordinal: expected_ordinal.load(Ordering::SeqCst) as u64,
        },
    );
    assert_deep_callbacks(&world);
    assert_eq!(world.committed_version(), before);
    assert_eq!(world.edges(), edges);
    world.observation.arm();
    let lawful = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::swap_successors(&world, &request, 2, 8);
    assert!(matches!(
        commit(&world, &request, lawful, 235),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_deep_callbacks(&world);
    assert_eq!(
        world.edges(),
        [
            (0, 1),
            (1, 2),
            (2, 6),
            (3, 4),
            (4, 5),
            (5, 3),
            (6, 7),
            (7, 8),
            (8, 0)
        ]
    );
}
