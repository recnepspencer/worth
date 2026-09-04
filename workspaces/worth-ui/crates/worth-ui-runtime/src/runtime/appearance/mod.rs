mod inspection;
mod invalidation;
mod projection;
mod state;
#[allow(
    dead_code,
    reason = "Gate 0 freezes theme switching without making it live"
)]
mod theme;

#[cfg(test)]
mod host_completion_tests;

pub(crate) use inspection::{
    UiAppearanceInspectionAttemptBatch, UiAppearanceInspectionDenial,
    UiAppearanceInspectionProducer, UiAppearanceInspectionRecord,
};
pub(crate) use invalidation::{UiAppearanceConsumerSelection, UiAppearanceInvalidationBatch};
#[cfg(test)]
pub(crate) use projection::projection_test_inputs;
#[allow(
    unused_imports,
    reason = "Gate 1 retains sealed appearance projection re-exports for later mounting consumers"
)]
pub(crate) use projection::{
    UiAppearanceAttemptContext, UiAppearanceChangeOutcome, UiAppearanceChangeReceipt,
    UiAppearanceMountAffinity, UiAppearanceMountAffinityDenial, UiAppearanceProjection,
    UiAppearanceProjectionAttempt, UiAppearanceResolutionDenial, UiAppearanceResolver,
    UiAppearanceSupportPosture, UiBackdropAppearanceProjection, UiOverlayStackSnapshot,
};
#[cfg(test)]
pub(crate) use state::validate_presentation_for_test;
pub use state::UiAppearanceOwnerSnapshot;
#[allow(
    unused_imports,
    reason = "Gate 1 retains sealed appearance state re-exports for later interaction consumers"
)]
pub(crate) use state::{
    UiAppearanceCoherentBasis, UiAppearanceCoherentBasisDenial, UiAppearanceCoherentBasisInput,
    UiAppearanceNodeRoleBinding, UiAppearanceSelectionSelector, UiAppearanceStateAdapterDenial,
    UiAppearanceStateAxisDemand, UiAppearanceStateConsumer, UiAppearanceStateConsumerSelection,
    UiAppearanceStateConsumerSelectionCost, UiAppearanceStateVector, UiAppearanceStateVectorDenial,
    UiAppearanceTarget, UiFocusAppearanceState, UiHoverAppearanceState,
    UiOperabilityAppearanceState, UiPressedAppearanceState, UiSelectionAppearanceState,
    UiValidationAppearanceState,
};
pub(crate) use theme::{
    prepare_theme_generation_rebinding, UiActiveThemeBinding, UiAppearanceThemeState,
    UiPreparedThemeBindingAdmission, UiPreparedThemeGenerationRebinding, UiPreparedThemeSwitch,
    UiThemeCapabilityAdmission, UiThemeCapabilityReceipt, UiThemeCapabilityReceiptDenial,
    UiThemeInitialBindingDenial, UiThemeResolutionDenial, UiThemeResolutionView,
    UiThemeSwitchDenial, UiThemeSwitchOrigin, UiThemeSwitchOriginAdmissionDenial,
    UiThemeSwitchOriginFamily, UiThemeSwitchRequest,
};
