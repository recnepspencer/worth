//! Preserve candidate identity isolation across a real same-installation fork.
use super::*;
use super::{observation::Entity, probe::Probe};
use crate::facade::application_invariants::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
};

#[test]
fn an_actual_sibling_candidate_rejects_the_original_typed_root_before_acquisition() {
    let _isolation = super::super::super::fixture::isolated_request_owner();
    let captured = Arc::new(Mutex::new(None::<Entity>));
    let check_foreign = Arc::new(AtomicBool::new(false));
    let completed = Arc::new(AtomicUsize::new(0));
    let (seen, checking, contacts) = (captured.clone(), check_foreign.clone(), completed.clone());
    let probe = Probe::new(move |context, scope, rule| {
        let local = scope.node(0);
        if !checking.load(Ordering::SeqCst) {
            *seen.lock().unwrap() = Some(local.clone());
            return;
        }
        let foreign = seen
            .lock()
            .unwrap()
            .clone()
            .expect("actual branch A proposed identity");
        assert_eq!(
            foreign.entity_id(),
            local.entity_id(),
            "the fork preserves the original native node"
        );
        assert!(
            foreign != *local,
            "candidate affinity differs within the same installed schema"
        );
        let view = context.proposed();
        let before = context.reads();
        let error = view.reobserve_entity(&rule.node, &foreign).err().unwrap();
        assert_eq!(
            error.kind(),
            WorthQueryInvariantAccessDenialKind::ForeignView
        );
        assert_eq!(
            context.reads(),
            before,
            "a stale selector is refused before acquisition"
        );
        let error = view
            .walk(&rule.next, RelationDirection::Forward, &foreign)
            .unwrap_err();
        assert_eq!(
            error.kind(),
            WorthQueryInvariantAccessDenialKind::ForeignView
        );
        assert_eq!(context.reads(), before);
        let error = view.reaches(&rule.next, local, &foreign).unwrap_err();
        assert_eq!(
            error.kind(),
            WorthQueryInvariantAccessDenialKind::ForeignView
        );
        assert_eq!(context.reads(), before);
        let error = view
            .closure(
                &rule.next,
                RelationDirection::Reverse,
                &[local.clone(), foreign],
            )
            .unwrap_err();
        assert_eq!(
            error.kind(),
            WorthQueryInvariantAccessDenialKind::ForeignView
        );
        assert_eq!(
            context.reads(),
            before,
            "identity refusal precedes graph acquisitions"
        );
        assert_eq!(
            view.walk(&rule.next, RelationDirection::Forward, local)
                .unwrap()
                .len(),
            4
        );
        contacts.fetch_add(1, Ordering::SeqCst);
    });
    let world = world::install_with_probe(world::Graph::three_hops(false, vec![]), probe.clone());
    let sibling = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    assert_ne!(
        sibling,
        world.application.current_world(),
        "two actual owner-issued branch occurrences"
    );
    let request = super::super::super::fixture::live_scope();
    let first = program::set_active(&world, &request, true);
    probe.arm();
    assert!(matches!(
        commit(&world, &request, first, 216),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert!(captured.lock().unwrap().is_some());
    check_foreign.store(true, Ordering::SeqCst);
    let second = program::set_active_on_branch(&world, &request, true, Some(sibling));
    assert!(matches!(
        commit(&world, &request, second, 217),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert!(
        completed.load(Ordering::SeqCst) > 0,
        "B completed foreign refusals and its funded local control"
    );
}
