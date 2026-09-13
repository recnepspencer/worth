use super::*;

#[test]
fn late_older_physical_completion_cannot_regress_a_superseded_baseline() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let b_basis = sequence.successor;
    let c_basis = basis_for_lineage(
        b_basis.semantic_surface(),
        b_basis.host_surface(),
        b_basis.binding(),
        b_basis.host_lineage(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        Some(b_basis.mounted_frame()),
        false,
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
    );
    let a = owner.admit_pending(sequence.baseline).unwrap();
    let b = owner.admit_pending(b_basis).unwrap();
    let c = owner.admit_pending(c_basis.clone()).unwrap();
    owner.admit_superseded_physical(&b).unwrap();
    owner.admit_superseded_physical(&a).unwrap();
    assert!(owner.current.is_empty());
    owner.reject_before_effects(&c).unwrap();
    let retry = owner.admit_pending(c_basis).unwrap();
    owner
        .admit_presented(&retry, &native_paint_completion(3))
        .unwrap();
    owner.close_terminal_resources().unwrap();
    assert!(owner.retained.is_empty());
}

#[test]
fn superseded_physical_baseline_survives_successor_atlas_rejection() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let a = owner.admit_pending(sequence.baseline).unwrap();
    let b = owner.admit_pending(sequence.successor.clone()).unwrap();
    owner.admit_superseded_physical(&a).unwrap();
    assert!(
        owner.current.is_empty(),
        "stale physical completion cannot become Query current"
    );
    owner.reject_before_effects(&b).unwrap();
    let retry = owner.admit_pending(sequence.successor).unwrap();
    let completion = owner
        .admit_presented(&retry, &native_paint_completion(2))
        .unwrap();
    assert_eq!(
        completion.observation().posture(),
        WorthUiPresentationAsyncPosture::Current
    );
    let closed = owner.close_terminal_resources().unwrap();
    assert!(closed.transition_trace_complete());
    assert!(owner.retained.is_empty());
}

#[test]
fn late_superseded_physical_completion_does_not_replace_current_baseline() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let a = owner.admit_pending(sequence.baseline).unwrap();
    let b_basis = sequence.successor;
    let b = owner.admit_pending(b_basis.clone()).unwrap();
    owner
        .admit_presented(&b, &native_paint_completion(2))
        .unwrap();
    let accepted_nonce = owner.retained.values().next().unwrap().0;
    owner.admit_superseded_physical(&a).unwrap();
    assert_eq!(owner.retained.values().next().unwrap().0, accepted_nonce);
    let current = owner.current.values().next().unwrap();
    assert_eq!(current.0.attempt, b_basis.attempt());
    assert!(matches!(
        owner.admit_pending(b_basis),
        Err(WorthUiPresentationPendingAdmissionDenial::DuplicateAttemptBinding)
    ));
    owner.close_terminal_resources().unwrap();
    assert!(owner.retained.is_empty());
}
