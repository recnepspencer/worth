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

pub(crate) use inspection::UiAppearanceInspectionProducer;
pub(crate) use invalidation::UiAppearanceConsumerSelection;
pub(crate) use projection::{
    UiAppearanceChangeOutcome, UiAppearanceChangeReceipt, UiAppearanceProjection,
    UiAppearanceResolutionDenial, UiAppearanceResolver, UiBackdropAppearanceProjection,
    UiOverlayStackSnapshot,
};
pub use state::UiAppearanceOwnerSnapshot;
pub(crate) use state::UiAppearanceStateAxisDemand;
pub(crate) use theme::{
    UiAppearanceThemeState, UiPreparedThemeSwitch, UiThemeCapabilityReceipt,
    UiThemeInitialBindingDenial, UiThemeSwitchDenial, UiThemeSwitchOrigin,
    UiThemeSwitchOriginAdmissionDenial, UiThemeSwitchOriginFamily, UiThemeSwitchRequest,
};
