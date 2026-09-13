use super::*;

#[test]
fn terminal_exit_retention_survives_track_terminalization_until_exact_release() {
    let mut state = runtime_state();
    let frame = frame();
    let target = target(24);
    let commit = commit(
        &mut state,
        proposal(24),
        request_for_target_with_declaration(
            target,
            1,
            2,
            frame,
            UiMotionDeclaration::portal_exit(),
        ),
        frame,
    );
    let retention = commit
        .exit_retention()
        .expect("portal exit reserves exact Motion retention");
    assert_eq!(state.census().exit_retentions(), 1);

    let terminal = state
        .terminalize(commit.track().identity(), UiMotionTerminalCause::Completed)
        .expect("retained exit remains terminalizable");
    assert_eq!(terminal.exit_retention(), Some(retention));
    assert_eq!(state.census().active_tracks(), 0);
    assert_eq!(state.census().exit_retentions(), 1);

    assert!(state.release_exit_retention(retention));
    assert!(!state.release_exit_retention(retention));
    assert!(state.census().is_zero());
}

#[test]
fn interruption_displaces_a_terminal_exit_retention_without_leaking_census() {
    let mut state = runtime_state();
    let frame = frame();
    let target = target(25);
    let exit = commit(
        &mut state,
        proposal(25),
        request_for_target_with_declaration(
            target,
            1,
            2,
            frame,
            UiMotionDeclaration::portal_exit(),
        ),
        frame,
    );
    let displaced = exit.exit_retention().unwrap();
    state
        .terminalize(exit.track().identity(), UiMotionTerminalCause::Completed)
        .unwrap();

    let entrance = commit(
        &mut state,
        proposal(26),
        request_for_target_with_declaration(
            target,
            2,
            3,
            frame,
            UiMotionDeclaration::portal_entrance(),
        ),
        frame,
    );
    assert_eq!(entrance.displaced_exit_retention(), Some(displaced));
    assert_eq!(entrance.exit_retention(), None);
    assert_eq!(state.census().exit_retentions(), 0);
    assert!(!state.release_exit_retention(displaced));
}

#[test]
fn exit_retarget_replaces_retention_one_for_one_at_capacity_boundary() {
    let mut state = runtime_state();
    let frame = frame();
    let target = target(27);
    let first = commit(
        &mut state,
        proposal(27),
        request_for_target_with_declaration(
            target,
            1,
            2,
            frame,
            UiMotionDeclaration::portal_exit(),
        ),
        frame,
    );
    let second = commit(
        &mut state,
        proposal(28),
        request_for_target_with_declaration(
            target,
            2,
            3,
            frame,
            UiMotionDeclaration::portal_exit(),
        ),
        frame,
    );

    assert_eq!(second.displaced_exit_retention(), first.exit_retention());
    assert_ne!(second.exit_retention(), first.exit_retention());
    assert_eq!(state.census().active_tracks(), 1);
    assert_eq!(state.census().exit_retentions(), 1);
}
