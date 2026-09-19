mod change;
mod outcome;
pub(crate) use change::UiThemeSwitchChange;
pub use change::UiThemeSwitchSelectionDenial;
pub use outcome::UiThemeSwitchOutcome;
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
pub use active_binding::UiActiveThemeBinding;
pub(crate) use capability::{
    prepare_theme_generation_rebinding, UiPreparedThemeBindingAdmission,
    UiPreparedThemeGenerationRebinding, UiThemeCapabilityAdmission,
};
pub use capability::{UiThemeCapabilityReceipt, UiThemeCapabilityReceiptDenial};
pub(crate) use prepared_switch::UiPreparedThemeSwitch;
pub use resolution_view::UiThemeResolutionDenial;
pub(crate) use resolution_view::{UiResolvedThemeSlot, UiThemeResolutionView};
pub use state::UiThemeSwitchDenial;
pub(crate) use state::{UiAppearanceThemeState, UiThemeInitialBindingDenial};
pub use switch_request::{
    UiThemeSwitchOrigin, UiThemeSwitchOriginAdmissionDenial, UiThemeSwitchOriginFamily,
    UiThemeSwitchRequest,
};
