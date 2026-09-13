mod inspection;
mod invalidation;
mod projection;
mod state;
mod theme;

#[cfg(test)]
mod host_completion_tests;

pub use inspection::UiAppearanceInspectionGenerationSuccessionDenial;
pub(crate) use inspection::{
    UiAppearanceInspectionAttemptBatch, UiAppearanceInspectionDenial,
    UiAppearanceInspectionProducer, UiAppearanceInspectionRecord,
    UiPreparedAppearanceInspectionGenerationSuccession,
};
pub(crate) use invalidation::UiAppearanceInvalidationBatch;
mod invalidation_input;
pub(crate) use invalidation_input::UiAppearanceInvalidationInput;
#[cfg(test)]
pub(crate) use projection::projection_test_inputs;
#[cfg(test)]
pub(crate) use projection::projection_test_inputs_from_session;
pub(crate) use projection::{
    UiAppearanceAttemptContext, UiAppearanceChangeReceipt, UiAppearanceMountAffinity,
    UiAppearanceProjection, UiAppearanceProjectionAttempt, UiAppearanceResolver,
    UiAppearanceSupportPosture, UiBackdropAppearanceProjection, UiOverlayStackSnapshot,
};
#[cfg(test)]
pub(crate) use state::validate_presentation_for_test;
pub use state::UiAppearanceOwnerSnapshot;
pub(crate) use state::UiPreparedRetainedAppearanceOwnerSuccession;
pub(crate) use state::{
    UiAppearanceCoherentBasis, UiAppearanceCoherentBasisDenial, UiAppearanceCoherentBasisInput,
    UiAppearanceNodeRoleBinding, UiAppearanceSelectionSelector, UiAppearanceStateAdapterDenial,
    UiAppearanceStateAxisDemand, UiAppearanceStateConsumer, UiAppearanceStateVector,
    UiAppearanceStateVectorDenial, UiAppearanceTarget, UiBackdropAppearanceStateVector,
};
pub(crate) use theme::{
    prepare_theme_generation_rebinding, UiAppearanceThemeState, UiPreparedThemeBindingAdmission,
    UiPreparedThemeGenerationRebinding, UiPreparedThemeSwitch, UiThemeCapabilityAdmission,
    UiThemeInitialBindingDenial, UiThemeResolutionView, UiThemeSwitchChange,
};
pub use theme::{
    UiActiveThemeBinding, UiThemeCapabilityReceipt, UiThemeCapabilityReceiptDenial,
    UiThemeSwitchDenial, UiThemeSwitchOrigin, UiThemeSwitchOriginAdmissionDenial,
    UiThemeSwitchOriginFamily, UiThemeSwitchRequest,
};
pub use theme::{UiThemeResolutionDenial, UiThemeSwitchOutcome, UiThemeSwitchSelectionDenial};
