use super::UiInteractionRuntimeState;
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostObservationSequence, UiHostPointerIdentity,
    UiHostSurfacePosition, UiMountedInstanceIdentity, UiMountedNodeReceiptIdentity,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

#[test]
fn shutdown_reports_an_exact_pointer_presence_census() {
    let mut state = UiInteractionRuntimeState::new(
        true,
        false,
        super::super::pointer_presence::UiPointerPresenceCapacity::for_test(2),
    );
    let session = crate::runtime::tests::active_application_session_test_support::
        source_backed_component_session();
    let generation = session.active_generation_identity();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let presentation = UiHostObservationPresentationBasis::new(
        worth_ui_host_contract::UiHostSurfaceIdentity::mint_unbound().unwrap(),
        worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
        binding,
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1),
    );
    state
        .pointer_presence
        .as_mut()
        .expect("the test enables pointer presence")
        .record_pointer_target(
            UiHostPointerIdentity::new(19),
            super::super::pointer_presence::UiPrimaryPointerKind::Mouse,
            UiHostObservationSequence::new(1),
            UiHostSurfacePosition::viewport_logical(10, 10),
            presentation,
            Some((
                UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
                binding,
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
                UiMountedNodeReceiptIdentity::mint_unbound().unwrap(),
            )),
            &generation,
        )
        .unwrap();
    assert_eq!(state.pointer_presence.as_ref().unwrap().pointer_count(), 1);
    assert_eq!(state.pointer_presence.as_ref().unwrap().primary_count(), 1);

    let report = state.shutdown();
    let final_state = report.final_state().expect("shutdown retains final state");
    assert_eq!(final_state.pointer_presence_records(), 0);
    assert_eq!(final_state.active_gestures(), 0);
    assert_eq!(state.pointer_presence.as_ref().unwrap().pointer_count(), 0);
    assert_eq!(state.pointer_presence.as_ref().unwrap().primary_count(), 0);
    assert!(
        state
            .pointer_presence
            .as_ref()
            .unwrap()
            .appearance_snapshot()
            .postures()
            .is_empty()
    );
    let _ = session.shutdown();
}
