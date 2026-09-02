mod state;
#[allow(
    dead_code,
    reason = "Gate 0 freezes theme switching without making it live"
)]
mod theme;

#[cfg(test)]
mod host_completion_tests;

#[cfg(test)]
pub(crate) use state::validate_presentation_for_test;
pub use state::UiAppearanceOwnerSnapshot;
pub(crate) use state::{
    UiAppearanceCoherentBasis, UiAppearanceCoherentBasisDenial, UiAppearanceCoherentBasisInput,
    UiAppearanceSelectionSelector, UiAppearanceStateAdapterDenial, UiAppearanceStateAxisDemand,
    UiAppearanceStateConsumer, UiAppearanceStateConsumerSelection,
    UiAppearanceStateConsumerSelectionCost, UiAppearanceStateVector, UiAppearanceStateVectorDenial,
    UiFocusAppearanceState, UiHoverAppearanceState, UiOperabilityAppearanceState,
    UiPressedAppearanceState, UiSelectionAppearanceState, UiValidationAppearanceState,
};
pub(crate) use theme::{
    UiActiveThemeBinding, UiAppearanceThemeState, UiPreparedThemeSwitch, UiThemeCapabilityReceipt,
    UiThemeInitialBindingDenial, UiThemeSwitchDenial, UiThemeSwitchOrigin,
    UiThemeSwitchOriginAdmissionDenial, UiThemeSwitchOriginFamily, UiThemeSwitchRequest,
};
