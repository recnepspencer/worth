//! Appearance lifecycle facade: the admitted theme definitions a session can
//! bind, and the switch requests, receipts and denials that bind one.

pub use super::entry::{
    UiNativeThemeSwitchDenial, UiProgrammaticThemeSwitchPreparationDenial,
    UiThemeSwitchPreparationDenial,
};
pub use crate::capability::{
    FrozenAppearanceThemeCapabilities, FrozenAppearanceThemeCapabilitiesDenial, UiThemeDefinition,
    UiThemeDefinitionDenial, UiThemeDefinitionIdentity, UiThemeSlotCatalog,
    UiThemeSlotCatalogDenial, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
    UiThemeSlotSuccessorCompatibility,
};
pub use crate::runtime::appearance::{
    UiActiveThemeBinding, UiThemeCapabilityReceipt, UiThemeCapabilityReceiptDenial,
    UiThemeResolutionDenial, UiThemeSwitchDenial, UiThemeSwitchOrigin,
    UiThemeSwitchOriginAdmissionDenial, UiThemeSwitchOriginFamily, UiThemeSwitchOutcome,
    UiThemeSwitchRequest, UiThemeSwitchSelectionDenial,
};
