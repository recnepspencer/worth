use super::*;

#[test]
fn scoped_reconciliation_preserves_accepted_focus_until_its_surface_is_retired() {
    let owner = World::new(1);
    let neighbor = World::new(1);
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    state
        .reconcile_mounted_participation(&owner.snapshot)
        .unwrap();
    let plan = state
        .plan(UiFocusRequest::Direct {
            scope: owner.scope,
            participant: owner.identities[0].0,
            incarnation: owner.identities[0].1,
            cause: UiFocusCause::Direct,
        })
        .unwrap();
    let accepted = state.commit(plan).unwrap().current().unwrap();
    let scoped = crate::mounting::UiMountedFocusParticipationSnapshot::new(
        neighbor.snapshot.frame(),
        neighbor.snapshot.participants().to_vec(),
        neighbor.snapshot.nodes_visited(),
        vec![owner.scope.semantic_surface()],
    );
    let rejected = state.prepare_mounted_reconciliation(&scoped).unwrap();
    assert_eq!(state.inspect().current(), Some(accepted));
    drop(rejected);
    assert_eq!(state.inspect().current(), Some(accepted));
    let retry = state.prepare_mounted_reconciliation(&scoped).unwrap();
    let committed = state.commit_mounted_reconciliation(retry).unwrap();
    assert!(committed.transition().is_none());
    assert_eq!(committed.mounted_nodes_visited(), 2);
    assert_eq!(state.inspect().current(), Some(accepted));

    let retired = state
        .reconcile_mounted_participation(&neighbor.snapshot)
        .unwrap();
    assert_eq!(retired.participants_installed(), 1);
    assert!(retired.transition().unwrap().current().is_none());
    assert!(state.inspect().current().is_none());
}

#[test]
fn prepared_reconciliation_is_effect_free_until_exact_commit() {
    let world = World::new(2);
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    state
        .reconcile_mounted_participation(&world.snapshot)
        .unwrap();
    let current = state
        .commit(
            state
                .plan(UiFocusRequest::Direct {
                    scope: world.scope,
                    participant: world.identities[1].0,
                    incarnation: world.identities[1].1,
                    cause: UiFocusCause::Direct,
                })
                .unwrap(),
        )
        .unwrap()
        .current()
        .unwrap();
    let successor = world.with_reincarnated_participant(1);

    let prepared = state
        .prepare_mounted_reconciliation(&successor.snapshot)
        .unwrap();
    assert_eq!(state.inspect().current(), Some(current));

    let committed = state.commit_mounted_reconciliation(prepared).unwrap();
    let fallback = committed.transition().unwrap();
    assert_eq!(fallback.cause(), UiFocusCause::RebindFallback);
    assert_eq!(
        fallback.current().unwrap().participant(),
        successor.identities[0].0
    );
    assert_eq!(committed.participants_installed(), 2);
    assert_eq!(committed.mounted_nodes_visited(), 2);
}

#[test]
fn prepared_reconciliation_rejects_catalog_change_without_visible_focus_change() {
    let world = World::new(2);
    let mut state = UiFocusRuntimeState::new_session_restore_candidate();
    state
        .reconcile_mounted_participation(&world.snapshot)
        .unwrap();
    let prepared = state
        .prepare_mounted_reconciliation(&world.snapshot)
        .unwrap();
    let posture = state.appearance_posture();
    let successor = world.with_reincarnated_participant(1);
    state
        .install_mounted_participation(&successor.snapshot)
        .unwrap();
    assert_eq!(state.appearance_posture(), posture);
    assert!(!state.admits_mounted_reconciliation(&prepared));
    assert!(matches!(
        state.commit_mounted_reconciliation(prepared),
        Err(crate::runtime::focus::UiFocusRoutingDenial::StalePlan)
    ));
}
