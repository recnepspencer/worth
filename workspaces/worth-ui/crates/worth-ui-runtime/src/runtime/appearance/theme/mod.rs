mod active_binding;
mod capability;
mod prepared_switch;
mod resolution_view;
mod state;
mod switch_request;

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;

use crate::capability::{UiThemeDefinition, UiThemeDefinitionIdentity};
pub(crate) use active_binding::UiActiveThemeBinding;
#[allow(
    unused_imports,
    reason = "Gate 0 retains the non-current theme admission boundary"
)]
pub(crate) use capability::{
    prepare_theme_generation_rebinding, UiPreparedThemeBindingAdmission,
    UiPreparedThemeGenerationRebinding, UiThemeCapabilityAdmission, UiThemeCapabilityReceipt,
    UiThemeCapabilityReceiptDenial,
};
pub(crate) use prepared_switch::UiPreparedThemeSwitch;
pub(crate) use resolution_view::{
    UiResolvedThemeSlot, UiThemeResolutionDenial, UiThemeResolutionView,
};
pub(crate) use state::{UiAppearanceThemeState, UiThemeInitialBindingDenial, UiThemeSwitchDenial};
pub(crate) use switch_request::{
    UiThemeSwitchOrigin, UiThemeSwitchOriginAdmissionDenial, UiThemeSwitchOriginFamily,
    UiThemeSwitchRequest,
};
