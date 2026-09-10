use super::super::super::{UiNativePendingDeltaSettlement, UiNativePendingSurfaceSettlement};
use super::*;

use crate::native::physical_work_signal::{
    UiNativePhysicalPresentationBasis, UiNativePhysicalSignalStatus,
};
use crate::native::presentation::retained_draw_list::UiNativeRetainedDeltaUndo;
use crate::native::presentation::{
    reserve_presentation_owners, settle_port_result, UiNativePendingExternalObligation,
    UiNativePresentationEffects, UiNativePresentationFailure, UiNativePresentationPortFailure,
};
use worth_ui_host_contract::{
    UiHostProtocolContract, UiHostProtocolNegotiation, UiMountedFrameConsumptionInput,
    UiMountedFrameConsumptionView, UiMountedPresentationAttemptIdentity,
    UiMountedPresentationInitial, UiMountedPresentationWorkView, UiPresentationDeadline,
};

struct TerminalPresentationProbe {
    status: UiNativePhysicalSignalStatus,
}

impl UiNativePendingExternalObligation for TerminalPresentationProbe {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(self.status)
    }

    fn take_presented_observation(
        &mut self,
    ) -> Option<crate::native::presentation::UiNativePresentationPortObservation> {
        (self.status == UiNativePhysicalSignalStatus::Completed)
            .then(crate::native::presentation::UiNativePresentationPortObservation::test)
    }
}

#[test]
fn refused_superseding_delta_rolls_back_to_the_last_committed_frame() {
    let world = DrawListWorld::new();
    let frame_zero = UiMountedFrameIdentity::mint_unbound().unwrap();
    let zero = world.rect(
        frame_zero,
        world.first,
        0.0,
        UiMountedRgba8::new(10, 20, 30, 255),
    );
    let initial = world.initial(frame_zero, [zero]);
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();

    let frame_one = UiMountedFrameIdentity::mint_unbound().unwrap();
    let one = world.rect(
        frame_one,
        world.first,
        0.0,
        UiMountedRgba8::new(40, 50, 60, 255),
    );
    let first_delta = replacement_delta(&world, frame_zero, zero, frame_one, one);
    let (_, first_undo) = retained.stage_delta(&first_delta, &[]).unwrap();

    let frame_two = UiMountedFrameIdentity::mint_unbound().unwrap();
    let two = world.rect(
        frame_two,
        world.first,
        0.0,
        UiMountedRgba8::new(70, 80, 90, 255),
    );
    let second_delta = replacement_delta(&world, frame_one, one, frame_two, two);
    let (_, second_undo) = retained.stage_delta(&second_delta, &[]).unwrap();

    let mut successor = UiNativePendingSurfaceSettlement::Delta(
        UiNativePendingDeltaSettlement::new(second_undo, UiNativePresentationEffects::default()),
    );
    if successor
        .inherit_predecessor(UiNativePendingSurfaceSettlement::Delta(
            UiNativePendingDeltaSettlement::new(first_undo, UiNativePresentationEffects::default()),
        ))
        .is_err()
    {
        panic!("a superseding delta must inherit its predecessor rollback lineage");
    }
    let UiNativePendingSurfaceSettlement::Delta(lineage) = successor else {
        panic!("the successor must retain a composed delta settlement");
    };
    lineage.rollback(&mut retained).unwrap();

    assert_eq!(retained.frame(), frame_zero);
    let identity = command(zero).identity();
    assert_eq!(retained.command(identity), Some(&command(zero)));
    assert_eq!(
        retained.order.ordered().collect::<Vec<_>>(),
        vec![UiMountedPaintOrderIdentity::for_command(identity)]
    );
}

#[test]
fn superseded_then_rejected_pending_deltas_restore_host_truth_and_resources() {
    assert_premature_commit_mutant_loses_rollback();
    let world = DrawListWorld::new();
    let frame_zero = UiMountedFrameIdentity::mint_unbound().unwrap();
    let zero = world.rect(
        frame_zero,
        world.first,
        0.0,
        UiMountedRgba8::new(10, 20, 30, 255),
    );
    let initial = world.initial(frame_zero, [zero]);
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let frame_one = UiMountedFrameIdentity::mint_unbound().unwrap();
    let one = world.rect(
        frame_one,
        world.first,
        0.0,
        UiMountedRgba8::new(40, 50, 60, 255),
    );
    let (_, first_undo) = retained
        .stage_delta(
            &replacement_delta(&world, frame_zero, zero, frame_one, one),
            &[],
        )
        .unwrap();
    let frame_two = UiMountedFrameIdentity::mint_unbound().unwrap();
    let two = world.rect(
        frame_two,
        world.first,
        0.0,
        UiMountedRgba8::new(70, 80, 90, 255),
    );
    let (_, second_undo) = retained
        .stage_delta(
            &replacement_delta(&world, frame_one, one, frame_two, two),
            &[],
        )
        .unwrap();

    let mut state = crate::native::UiNativeHostState::new();
    let predecessor_basis = physical_basis(&world, &initial);
    let successor_basis = predecessor_basis.test_successor();
    let binding = predecessor_basis.binding().diagnostic_value();
    state.retained_draw_lists.insert(binding, retained);
    let predecessor = pending_delta(
        &mut state,
        predecessor_basis,
        first_undo,
        UiNativePhysicalSignalStatus::Completed,
    );
    let successor = pending_delta(
        &mut state,
        successor_basis,
        second_undo,
        UiNativePhysicalSignalStatus::RejectedBeforeEffects,
    );
    state.pending_presentations.extend([predecessor, successor]);

    let due = state
        .physical_signal
        .next_due_tick()
        .expect("both pending deltas retain Signal-owned poll wakes");
    state.physical_signal.advance_clock_to(due).unwrap();
    assert!(state.progress_one_physical_signal_ready());
    assert_eq!(state.pending_presentations.len(), 1);
    assert_eq!(state.retained_draw_lists[&binding].frame(), frame_two);
    assert!(state.progress_one_physical_signal_ready());

    let restored = &state.retained_draw_lists[&binding];
    assert_eq!(restored.frame(), frame_zero);
    assert_eq!(
        restored.command(command(zero).identity()),
        Some(&command(zero))
    );
    assert!(state.pending_presentations.is_empty());
    assert!(state.lifecycle.recovery_required(binding));
    assert_eq!(
        state.lifecycle.effect_posture(),
        crate::native::UiNativeEffectPosture::PresentationIndeterminate
    );
    assert!(state.resources.current().is_zero());
    assert_eq!(state.physical_signal.observation().active_requests, 0);
}

fn assert_premature_commit_mutant_loses_rollback() {
    let world = DrawListWorld::new();
    let frame_zero = UiMountedFrameIdentity::mint_unbound().unwrap();
    let zero = world.rect(
        frame_zero,
        world.first,
        0.0,
        UiMountedRgba8::new(10, 20, 30, 255),
    );
    let initial = world.initial(frame_zero, [zero]);
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let frame_one = UiMountedFrameIdentity::mint_unbound().unwrap();
    let one = world.rect(
        frame_one,
        world.first,
        0.0,
        UiMountedRgba8::new(40, 50, 60, 255),
    );
    let (_, first_undo) = retained
        .stage_delta(
            &replacement_delta(&world, frame_zero, zero, frame_one, one),
            &[],
        )
        .unwrap();
    let frame_two = UiMountedFrameIdentity::mint_unbound().unwrap();
    let two = world.rect(
        frame_two,
        world.first,
        0.0,
        UiMountedRgba8::new(70, 80, 90, 255),
    );
    let (_, second_undo) = retained
        .stage_delta(
            &replacement_delta(&world, frame_one, one, frame_two, two),
            &[],
        )
        .unwrap();
    let mut mutant = UiNativePendingSurfaceSettlement::Delta(UiNativePendingDeltaSettlement::new(
        second_undo,
        UiNativePresentationEffects::default(),
    ));
    if mutant
        .inherit_predecessor(UiNativePendingSurfaceSettlement::Delta(
            UiNativePendingDeltaSettlement::new(first_undo, UiNativePresentationEffects::default()),
        ))
        .is_err()
    {
        panic!("the mutant must discard the same composed rollback lineage");
    }
    mutant.commit_superseded_predecessor();
    assert_eq!(retained.frame(), frame_two);
    assert_ne!(retained.frame(), frame_zero);
}

fn physical_basis(
    world: &DrawListWorld,
    initial: &UiMountedPresentationInitial,
) -> UiNativePhysicalPresentationBasis {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(protocol) => protocol,
        UiHostProtocolNegotiation::Incompatible(_) => panic!("current protocol must negotiate"),
    };
    let view =
        UiMountedFrameConsumptionView::from_inert_mechanics(UiMountedFrameConsumptionInput {
            authority: std::rc::Rc::new(()),
            host_session_identity: 1,
            protocol,
            capability_generation: world.requirement.capability_generation(),
            capability_profile_digest: world.requirement.capability_profile_digest(),
            attempt: UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            deadline: UiPresentationDeadline::at_tick(20),
            requirement: world.requirement,
            presentation_work: UiMountedPresentationWorkView::Initial(initial),
            appearance_work: None,
            qualified_text: &(),
            text_raster_work: None,
        });
    UiNativePhysicalPresentationBasis::from_view(&view)
}

fn pending_delta(
    state: &mut crate::native::UiNativeHostState,
    basis: UiNativePhysicalPresentationBasis,
    undo: UiNativeRetainedDeltaUndo,
    status: UiNativePhysicalSignalStatus,
) -> crate::native::UiNativePendingPresentation {
    let owners =
        reserve_presentation_owners(&mut state.resources, &mut state.physical_signal, basis)
            .unwrap_or_else(|_| panic!("the empty host must admit both pending delta attempts"));
    let result = settle_port_result(
        &mut state.resources,
        &mut state.physical_signal,
        owners,
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(TerminalPresentationProbe { status }),
        )),
    );
    let Err(UiNativePresentationFailure::Pending(pending)) = result else {
        panic!("an unsettled external delta must retain its physical obligation");
    };
    pending.with_settlement(UiNativePendingSurfaceSettlement::Delta(
        UiNativePendingDeltaSettlement::new(undo, UiNativePresentationEffects::default()),
    ))
}

fn replacement_delta(
    world: &DrawListWorld,
    predecessor: UiMountedFrameIdentity,
    previous: UiMountedFilledRectMechanic,
    successor: UiMountedFrameIdentity,
    replacement: UiMountedFilledRectMechanic,
) -> UiMountedPresentationDelta {
    let replacement = command(replacement);
    let replacement_order = UiMountedPaintOrderIdentity::for_command(replacement.identity());
    let replacement_bounds = replacement.bounds();
    UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor,
        successor,
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: vec![UiMountedPaintCommandChange::replacement(
            command(previous).identity(),
            replacement,
        )],
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[replacement_order]),
        damage: vec![
            UiMountedLogicalDamage::from_runtime_mounting(previous.bounds()),
            UiMountedLogicalDamage::from_runtime_mounting(replacement_bounds),
        ],
        auxiliary: None,
        production_cost: Default::default(),
    })
}
