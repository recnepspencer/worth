use super::tests::{basis, basis_for_lineage, installed_owner, presentation_sequence};
use super::*;

#[test]
fn owner_issued_indeterminate_effects_advance_the_exact_request_to_unresolved() {
    let mut owner = installed_owner();
    let pending = owner.admit_pending(basis(77)).unwrap();

    let unresolved = owner.admit_effects_indeterminate(&pending).unwrap();

    assert_eq!(
        unresolved.observation().posture(),
        WorthUiPresentationAsyncPosture::Unresolved
    );
    assert_eq!(unresolved.attempt(), pending.attempt());
    assert_eq!(unresolved.binding(), pending.binding());
    assert_eq!(owner.unresolved.len(), 1);
    assert!(owner.pending.is_empty());
}

#[test]
fn unresolved_lineage_blocks_a_successor_until_recovery_resolves_it() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let pending = owner.admit_pending(sequence.baseline.clone()).unwrap();
    owner.admit_effects_indeterminate(&pending).unwrap();

    let denial = owner.admit_pending(sequence.successor).unwrap_err();
    assert!(matches!(
        denial,
        WorthUiPresentationPendingAdmissionDenial::UnresolvedLineageAdmission
    ));
    assert_eq!(owner.unresolved.len(), 1);
}

#[test]
fn reconstruction_requirement_allows_a_fresh_successor_to_replace_unresolved() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let pending = owner.admit_pending(sequence.baseline.clone()).unwrap();
    let unresolved = owner.admit_effects_indeterminate(&pending).unwrap();
    let recovery = owner.require_reconstruction(&unresolved).unwrap();
    assert_eq!(recovery.attempt(), unresolved.attempt());
    assert_eq!(recovery.binding(), unresolved.binding());
    assert_eq!(
        recovery.observation().posture(),
        WorthUiPresentationAsyncPosture::Unresolved
    );

    let successor = owner
        .admit_pending(reconstruction_basis(&sequence.baseline, None))
        .unwrap();
    assert_eq!(
        owner
            .transition_trace
            .iter()
            .map(|observation| observation.kind())
            .collect::<Vec<_>>(),
        vec![
            WorthUiPresentationTransitionKind::Pending,
            WorthUiPresentationTransitionKind::Unresolved,
            WorthUiPresentationTransitionKind::RecoveryRequired,
        ]
    );
    assert_eq!(owner.unresolved.len(), 1);
    assert!(owner.superseded_pending.is_empty());
    let completed = owner.admit_presented(&successor).unwrap();
    assert_eq!(
        completed.observation().posture(),
        WorthUiPresentationAsyncPosture::Current
    );
    assert!(owner.superseded_pending.is_empty());
    let close = owner.close_terminal_resources().unwrap();
    assert_eq!(close.closed_query_resources(), 1);
    assert!(owner.current.is_empty());
    assert!(owner.active_keys.is_empty());
}

#[test]
fn reconstruction_can_retry_after_a_pre_effect_atlas_deferral() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let pending = owner.admit_pending(sequence.baseline.clone()).unwrap();
    let unresolved = owner.admit_effects_indeterminate(&pending).unwrap();
    owner.require_reconstruction(&unresolved).unwrap();

    let deferred = owner
        .admit_pending(reconstruction_basis(&sequence.baseline, None))
        .unwrap();
    owner.reject_before_effects(&deferred).unwrap();
    assert_eq!(owner.unresolved.len(), 1);
    assert!(owner.superseded_pending.is_empty());

    let retry = owner
        .admit_pending(reconstruction_basis(&sequence.baseline, None))
        .unwrap();
    let completed = owner.admit_presented(&retry).unwrap();

    assert_eq!(
        completed.observation().posture(),
        WorthUiPresentationAsyncPosture::Current
    );
    assert!(owner.pending.is_empty());
    assert!(owner.superseded_pending.is_empty());
    assert_eq!(
        owner
            .close_terminal_resources()
            .unwrap()
            .closed_query_resources(),
        1
    );
}

#[test]
fn indeterminate_settlement_can_issue_reconstruction_authority_atomically() {
    let mut owner = installed_owner();
    let pending = owner.admit_pending(basis(91)).unwrap();

    let recovery = owner
        .admit_effects_indeterminate_requiring_reconstruction(&pending)
        .unwrap();

    assert_eq!(recovery.attempt(), pending.attempt());
    assert_eq!(recovery.binding(), pending.binding());
    assert_eq!(
        owner
            .transition_trace
            .iter()
            .map(|observation| observation.kind())
            .collect::<Vec<_>>(),
        vec![
            WorthUiPresentationTransitionKind::Pending,
            WorthUiPresentationTransitionKind::Unresolved,
            WorthUiPresentationTransitionKind::RecoveryRequired,
        ]
    );
}

#[test]
fn reconstruction_rejects_a_predecessor_other_than_the_retained_mounted_authority() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let pending = owner.admit_pending(sequence.baseline.clone()).unwrap();
    let unresolved = owner.admit_effects_indeterminate(&pending).unwrap();
    owner.require_reconstruction(&unresolved).unwrap();

    let denial = owner
        .admit_pending(reconstruction_basis(
            &sequence.baseline,
            Some(worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap()),
        ))
        .unwrap_err();

    assert!(matches!(
        denial,
        WorthUiPresentationPendingAdmissionDenial::StalePredecessor
    ));
}

fn reconstruction_basis(
    unresolved: &WorthUiPresentationRequestBasis,
    predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
) -> WorthUiPresentationRequestBasis {
    basis_for_lineage(
        unresolved.semantic_surface(),
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        unresolved.host_lineage(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        predecessor,
        true,
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
    )
}
