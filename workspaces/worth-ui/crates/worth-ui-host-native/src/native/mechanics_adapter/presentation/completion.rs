use worth_ui_host_contract::{
    UiHostSurfacePresentationMode, UiHostSurfacePresentationOutcome, UiMountedFrameConsumptionView,
    UiMountedSurfacePresentationCompletion,
};

pub(super) fn completed(
    state: &mut crate::native::UiNativeHostState,
    key: u64,
    view: &UiMountedFrameConsumptionView<'_>,
    cost: worth_ui_host_contract::UiHostPresentationCostReport,
    painted: bool,
    effects: crate::native::presentation::UiNativePresentationEffects,
) -> UiHostSurfacePresentationOutcome {
    let Some(epoch) =
        super::epoch::presentation_epoch(state, key, view.attempt().diagnostic_value(), painted)
    else {
        return super::failure::malformed();
    };
    let outcome =
        UiHostSurfacePresentationOutcome::Presented(UiMountedSurfacePresentationCompletion::new(
            UiHostSurfacePresentationMode::NativeDisplay,
            epoch,
            effects.completion(),
            cost,
        ));
    let _input_settlement = state.lifecycle.record_completed_presentation(
        view.protocol(),
        view.host_session_identity(),
        worth_ui_host_contract::UiHostObservationPresentationBasis::new(
            view.requirement().host_surface(),
            view.frame(),
            view.binding(),
            epoch,
        ),
    );
    crate::native::capture::record_completed_view(state, view, epoch);
    #[cfg(feature = "certification-support")]
    state.apply_completed_qualified_derived_state_loss(key);
    outcome
}
