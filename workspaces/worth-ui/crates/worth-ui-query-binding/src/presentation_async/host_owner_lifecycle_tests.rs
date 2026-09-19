use super::tests::{
    basis_for_lineage, installed_owner, mechanic, native_paint_completion, presentation_sequence,
    raster_key,
};
use super::*;
use crate::presentation_async::WorthUiPresentationPinBasis;

#[path = "host_owner_lifecycle_tests/pending_capacity.rs"]
mod pending_capacity;

#[test]
fn newer_pending_attempt_supersedes_the_exact_prior_query_live_view() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let prior = owner.admit_pending(sequence.baseline).unwrap();
    let successor = owner.admit_pending(sequence.successor).unwrap();

    assert_eq!(
        owner.observation(&prior).unwrap().posture(),
        WorthUiPresentationAsyncPosture::Superseded
    );
    assert_eq!(
        owner.observation(&successor).unwrap().posture(),
        WorthUiPresentationAsyncPosture::Pending
    );
    let prior_terminal = owner.admit_superseded_physical(&prior).unwrap();
    assert_eq!(
        prior_terminal.observation().posture(),
        WorthUiPresentationAsyncPosture::Superseded
    );
    assert_eq!(
        owner.observation(&successor).unwrap().posture(),
        WorthUiPresentationAsyncPosture::Pending
    );
    let successor_terminal = owner
        .admit_presented(&successor, &native_paint_completion(2))
        .unwrap();
    assert_eq!(
        successor_terminal.observation().posture(),
        WorthUiPresentationAsyncPosture::Current
    );
}

#[test]
fn successor_can_complete_before_the_exact_superseded_physical_predecessor() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let prior = owner.admit_pending(sequence.baseline).unwrap();
    let successor = owner.admit_pending(sequence.successor).unwrap();

    let successor_terminal = owner
        .admit_presented(&successor, &native_paint_completion(3))
        .unwrap();
    assert_eq!(
        successor_terminal.observation().posture(),
        WorthUiPresentationAsyncPosture::Current
    );
    let prior_terminal = owner.admit_superseded_physical(&prior).unwrap();
    assert_eq!(
        prior_terminal.observation().posture(),
        WorthUiPresentationAsyncPosture::Superseded
    );
    assert!(owner.superseded_awaiting_completion.is_empty());
}

#[test]
fn complete_successor_retains_shared_pin_without_readding_it() {
    let mut owner = installed_owner();
    let semantic_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let host_surface = worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap();
    let binding = worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let lineage =
        worth_ui_host_contract::UiHostPresentationLineageIdentity::from_certification_host_session(
            91,
        )
        .unwrap();
    let layout =
        worth_ui_host_contract::UiQualifiedTextLayoutIdentity::from_text_mechanics([7; 32]);
    let pin = WorthUiPresentationPinBasis::from_runtime(
        worth_ui_host_contract::UiGlyphRasterPinRequest::from_text_mechanics(layout, raster_key()),
    );
    let baseline_frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let baseline = basis_for_lineage(
        semantic_surface,
        host_surface,
        binding,
        lineage,
        baseline_frame,
        None,
        true,
        vec![mechanic(0, layout)].into_boxed_slice(),
        vec![pin].into_boxed_slice(),
        vec![pin].into_boxed_slice(),
        Box::new([]),
    );
    let baseline = owner.admit_pending(baseline).unwrap();
    owner
        .admit_presented(&baseline, &native_paint_completion(10))
        .unwrap();

    let retained_frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let retained = basis_for_lineage(
        semantic_surface,
        host_surface,
        binding,
        lineage,
        retained_frame,
        Some(baseline_frame),
        true,
        Box::new([]),
        vec![pin].into_boxed_slice(),
        Box::new([]),
        Box::new([]),
    );
    assert!(retained.pin_additions().is_empty());
    assert_eq!(retained.binding_pins(), [pin]);
    let retained = owner.admit_pending(retained).unwrap();
    owner
        .admit_presented(&retained, &native_paint_completion(11))
        .unwrap();

    let released = basis_for_lineage(
        semantic_surface,
        host_surface,
        binding,
        lineage,
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        Some(retained_frame),
        true,
        Box::new([]),
        Box::new([]),
        Box::new([]),
        vec![pin].into_boxed_slice(),
    );
    assert!(owner.admit_pending(released).is_ok());
}

#[test]
fn current_successor_supersedes_and_closes_the_prior_query_resource() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let prior = owner.admit_pending(sequence.baseline).unwrap();
    owner
        .admit_presented(&prior, &native_paint_completion(20))
        .unwrap();
    let successor = owner.admit_pending(sequence.successor).unwrap();
    let successor = owner
        .admit_presented(&successor, &native_paint_completion(21))
        .unwrap();

    assert_eq!(
        successor.predecessor_observation().unwrap().posture(),
        WorthUiPresentationAsyncPosture::Superseded
    );
    assert_eq!(
        successor.observation().posture(),
        WorthUiPresentationAsyncPosture::Current
    );
    assert_eq!(owner.current.len(), 1);
    assert_eq!(owner.active_keys.len(), 1);
}

#[test]
fn before_effects_rejection_denies_and_closes_the_query_resource() {
    let mut owner = installed_owner();
    let receipt = owner.admit_pending(super::tests::basis(31)).unwrap();
    owner.reject_before_effects(&receipt).unwrap();

    assert!(owner.observation(&receipt).is_none());
    assert!(owner.pending.is_empty());
    assert!(owner.active_keys.is_empty());
}

#[test]
fn rejected_superseding_attempt_can_retry_from_the_exact_superseded_baseline() {
    let mut owner = installed_owner();
    let sequence = presentation_sequence();
    let prior = owner.admit_pending(sequence.baseline).unwrap();
    let displaced = owner.admit_pending(sequence.successor.clone()).unwrap();

    owner.reject_before_effects(&displaced).unwrap();

    let retried = owner.admit_pending(sequence.successor).unwrap();
    let completed = owner
        .admit_presented(&retried, &native_paint_completion(12))
        .unwrap();
    assert_eq!(
        completed.observation().posture(),
        WorthUiPresentationAsyncPosture::Current
    );
    assert!(owner.observation(&prior).is_none());
    assert!(owner.pending.is_empty());
    assert!(owner.superseded_pending.is_empty());
}
