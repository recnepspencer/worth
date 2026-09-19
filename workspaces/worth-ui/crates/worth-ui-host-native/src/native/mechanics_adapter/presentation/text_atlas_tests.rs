use std::rc::Rc;

use worth_ui_host_contract::{
    UiHostProtocolContract, UiHostProtocolNegotiation, UiHostSurfaceIdentity,
    UiHostSurfacePresentationMode, UiHostSurfaceStopReason, UiMountedContentGeneration,
    UiMountedFrameConsumptionInput, UiMountedFrameConsumptionView, UiMountedFrameIdentity,
    UiMountedPresentationAttemptIdentity, UiMountedPresentationProductionCost,
    UiMountedPresentationUnchanged, UiMountedPresentationUnchangedInput,
    UiMountedPresentationWorkView, UiMountedSurfaceBindingRequirement, UiPresentationDeadline,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    WorthUiHostCapabilityObservationGeneration,
};

#[test]
pub(crate) fn production_presentation_stop_routes_supersession_into_signal_recovery() {
    let mut state = crate::native::UiNativeHostState::new();
    let pending = crate::native::mechanics_adapter::seed_pending_atlas_for_event_loop(&mut state);
    let view = inert_view();
    let token = view.issue_completion_token();
    super::text_atlas::retain_pending(
        &mut state,
        &view,
        &token,
        (pending, Box::new([])),
        crate::native::host_state::UiNativePendingTextContinuation::AtlasReady,
    );
    let outcome = super::text_atlas::stop(&mut state, token, UiHostSurfaceStopReason::Superseded);
    assert_eq!(
        outcome,
        worth_ui_host_contract::UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun
    );
    let counters = state.physical_signal.observation().counters;
    assert_eq!(counters.supersessions, 1);
    assert_eq!(state.pending_text_presentations.len(), 0);
    assert_eq!(
        state.progress_text_atlas_physical(pending),
        crate::native::host_state::UiNativeTextAtlasPhysicalProgress::Terminal
    );
    assert!(state.close().is_zero());
}

pub(crate) fn inert_view() -> UiMountedFrameConsumptionView<'static> {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let host_surface = UiHostSurfaceIdentity::mint_unbound().unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let requirement = UiMountedSurfaceBindingRequirement::new(
        surface,
        host_surface,
        binding,
        WorthUiHostCapabilityObservationGeneration::new(1),
        1,
        UiHostSurfacePresentationMode::NativeDisplay,
    );
    let unchanged = Box::leak(Box::new(
        UiMountedPresentationUnchanged::from_inert_mechanics(UiMountedPresentationUnchangedInput {
            predecessor: UiMountedFrameIdentity::mint_unbound().unwrap(),
            successor: UiMountedFrameIdentity::mint_unbound().unwrap(),
            surface,
            binding,
            content: UiMountedContentGeneration::mint_unbound().unwrap(),
            baseline: requirement.baseline(),
            production_cost: UiMountedPresentationProductionCost::default(),
        }),
    ));
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        unreachable!("the current protocol is self-compatible")
    };
    let view =
        UiMountedFrameConsumptionView::from_inert_mechanics(UiMountedFrameConsumptionInput {
            authority: Rc::new(()),
            host_session_identity: 1,
            protocol,
            capability_generation: requirement.capability_generation(),
            capability_profile_digest: requirement.capability_profile_digest(),
            attempt: UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            deadline: UiPresentationDeadline::at_tick(10),
            requirement,
            presentation_work: UiMountedPresentationWorkView::Unchanged(unchanged),
            appearance_work: None,
            qualified_text: &(),
            text_raster_work: None,
        });
    view
}
